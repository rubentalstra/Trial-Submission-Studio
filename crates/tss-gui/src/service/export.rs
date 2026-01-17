//! Export service - async export with progress streaming.
//!
//! Clean Iced-native implementation using `Task::perform` pattern.
//! All export logic is self-contained with no legacy dependencies.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;

use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};
use tss_standards::{SdtmDomain, TerminologyRegistry};
use tss_submit::export::types::DomainFrame;
use tss_submit::export::{
    DatasetXmlOptions, DefineXmlOptions, build_xpt_dataset_with_name,
    write_dataset_xml as write_dataset_xml_output, write_define_xml as write_define_xml_output,
};
use tss_submit::{NormalizationContext, execute_normalization};
use tss_submit::{Severity, ValidationReport};

use crate::state::{
    DomainState, ExportFormat, ExportResult, SdtmIgVersion, SuppColumnConfig, XptVersion,
};

// =============================================================================
// INPUT TYPES
// =============================================================================

/// Complete input for export operation.
///
/// This struct contains all data needed for export, allowing the service
/// to run independently without references to application state.
#[derive(Clone)]
pub struct ExportInput {
    /// Output directory (will create `datasets/` subfolder).
    pub output_dir: PathBuf,
    /// Export format (XPT or Dataset-XML).
    pub format: ExportFormat,
    /// XPT version (only used when format is XPT).
    pub xpt_version: XptVersion,
    /// SDTM-IG version for Dataset-XML and Define-XML.
    pub sdtm_ig_version: SdtmIgVersion,
    /// Domains to export with their data.
    pub domains: Vec<DomainExportData>,
    /// Study ID (extracted from data or default).
    pub study_id: String,
    /// Whether to bypass validation errors.
    ///
    /// If true, export proceeds even with validation errors (warnings added).
    /// If false, export fails if any domain has validation errors.
    pub bypass_validation: bool,
    /// CT registry for validation (optional).
    pub ct_registry: Option<TerminologyRegistry>,
    /// Variables marked as "not collected" per domain.
    pub not_collected: HashMap<String, BTreeSet<String>>,
}

/// Data for a single domain to export.
#[derive(Clone)]
pub struct DomainExportData {
    /// Domain code (e.g., "DM", "AE").
    pub code: String,
    /// Domain definition from CDISC standards.
    pub definition: SdtmDomain,
    /// Transformed DataFrame ready for export.
    pub data: DataFrame,
    /// SUPP data if applicable.
    pub supp_data: Option<DataFrame>,
}

// =============================================================================
// OUTPUT TYPES
// =============================================================================

/// Export error.
#[derive(Debug, Clone)]
pub struct ExportError {
    /// Error message.
    pub message: String,
    /// Domain that caused the error (if applicable).
    pub domain: Option<String>,
}

impl ExportError {
    /// Create a new error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            domain: None,
        }
    }

    /// Create an error for a specific domain.
    pub fn for_domain(domain: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            domain: Some(domain.into()),
        }
    }
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref domain) = self.domain {
            write!(f, "[{}] {}", domain, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

// =============================================================================
// MAIN EXPORT FUNCTION
// =============================================================================

/// Execute export asynchronously.
///
/// This function is designed for use with `Task::perform`:
///
/// ```ignore
/// Task::perform(
///     execute_export(input),
///     |result| Message::Export(ExportMessage::Complete(result)),
/// )
/// ```
pub async fn execute_export(input: ExportInput) -> ExportResult {
    // Run blocking export in a separate thread
    match tokio::task::spawn_blocking(move || execute_export_sync(input)).await {
        Ok(result) => result,
        Err(e) => ExportResult::Error {
            message: format!("Export task panicked: {}", e),
            domain: None,
        },
    }
}

/// Synchronous export implementation (runs on blocking thread).
fn execute_export_sync(input: ExportInput) -> ExportResult {
    let start = Instant::now();

    // Run validation on all domains first
    let validation_errors = validate_all_domains(&input);

    if !validation_errors.is_empty() {
        if input.bypass_validation {
            // Continue but add warnings
            tracing::warn!(
                "Export proceeding with {} validation error(s) (bypass enabled)",
                validation_errors.len()
            );
        } else {
            // Block export due to validation errors
            let error_summary = validation_errors
                .iter()
                .map(|(domain, count)| format!("{}: {} error(s)", domain, count))
                .collect::<Vec<_>>()
                .join(", ");

            return ExportResult::Error {
                message: format!(
                    "Validation failed. Fix errors or enable 'Bypass Validation' in Developer Settings. Domains with errors: {}",
                    error_summary
                ),
                domain: None,
            };
        }
    }

    // Create output directory structure: {output_dir}/datasets/
    let datasets_dir = input.output_dir.join("datasets");
    if let Err(e) = std::fs::create_dir_all(&datasets_dir) {
        return ExportResult::Error {
            message: format!("Failed to create output directory: {}", e),
            domain: None,
        };
    }

    let mut written_files: Vec<PathBuf> = Vec::new();
    let mut domain_frames: Vec<DomainFrame> = Vec::new();
    let mut supp_frames: Vec<DomainFrame> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Add validation bypass warnings
    for (domain, count) in &validation_errors {
        warnings.push(format!(
            "{}: Exported with {} validation error(s) (bypass enabled)",
            domain, count
        ));
    }

    // Process each domain
    for domain_data in &input.domains {
        // Build main domain frame
        let frame = DomainFrame::new(domain_data.code.clone(), domain_data.data.clone());

        // Write data file
        let filename = format!(
            "{}.{}",
            domain_data.code.to_lowercase(),
            input.format.extension()
        );
        let path = datasets_dir.join(&filename);

        let ig_version = input.sdtm_ig_version.as_str();
        if let Err(e) = write_data_file(
            &path,
            &frame,
            &domain_data.definition,
            &input.study_id,
            input.format,
            ig_version,
        ) {
            return ExportResult::Error {
                message: e.message,
                domain: Some(domain_data.code.clone()),
            };
        }

        written_files.push(path);
        domain_frames.push(frame);

        // Write SUPP if present
        if let Some(ref supp_df) = domain_data.supp_data {
            let supp_code = format!("SUPP{}", domain_data.code.to_uppercase());
            let supp_frame = DomainFrame::new(supp_code.clone(), supp_df.clone());

            let supp_filename = format!(
                "supp{}.{}",
                domain_data.code.to_lowercase(),
                input.format.extension()
            );
            let supp_path = datasets_dir.join(&supp_filename);

            // For SUPP, we need to get the SUPP domain definition
            // Pass the parent domain's label for proper SUPP labeling
            let parent_label = domain_data.definition.label.as_deref();
            if let Some(supp_def) = build_supp_domain_definition(&domain_data.code, parent_label) {
                if let Err(e) = write_data_file(
                    &supp_path,
                    &supp_frame,
                    &supp_def,
                    &input.study_id,
                    input.format,
                    ig_version,
                ) {
                    warnings.push(format!("SUPP{} export warning: {}", domain_data.code, e));
                } else {
                    written_files.push(supp_path);
                    supp_frames.push(supp_frame);
                }
            }
        }
    }

    // Write Define-XML (always required)
    let define_path = datasets_dir.join("define.xml");
    let ig_version = input.sdtm_ig_version.as_str();
    if let Err(e) = write_define_xml(
        &define_path,
        &input.study_id,
        &input.domains,
        &domain_frames,
        &supp_frames,
        ig_version,
    ) {
        return ExportResult::Error {
            message: format!("Failed to write Define-XML: {}", e),
            domain: None,
        };
    }
    written_files.push(define_path);

    ExportResult::Success {
        output_dir: input.output_dir,
        files: written_files,
        domains_exported: input.domains.len(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        warnings,
    }
}

// =============================================================================
// FILE WRITERS
// =============================================================================

/// Write a data file in the specified format.
fn write_data_file(
    path: &Path,
    frame: &DomainFrame,
    domain: &SdtmDomain,
    study_id: &str,
    format: ExportFormat,
    sdtm_ig_version: &str,
) -> Result<(), ExportError> {
    match format {
        ExportFormat::Xpt => write_xpt_file(path, frame, domain),
        ExportFormat::DatasetXml => {
            write_dataset_xml_file(path, frame, domain, study_id, sdtm_ig_version)
        }
    }
}

/// Write XPT file using tss-output crate.
fn write_xpt_file(
    path: &Path,
    frame: &DomainFrame,
    domain: &SdtmDomain,
) -> Result<(), ExportError> {
    // Use the tss-output crate's XPT builder
    let dataset_name = frame.dataset_name();
    let dataset = build_xpt_dataset_with_name(domain, frame, &dataset_name)
        .map_err(|e| ExportError::new(format!("Failed to build XPT dataset: {}", e)))?;

    // Write using xportrs
    use xportrs::Xpt;
    Xpt::writer(dataset)
        .finalize()
        .map_err(|e| ExportError::new(format!("Failed to validate XPT: {}", e)))?
        .write_path(path)
        .map_err(|e| ExportError::new(format!("Failed to write XPT: {}", e)))?;

    Ok(())
}

/// Write Dataset-XML file.
fn write_dataset_xml_file(
    path: &Path,
    frame: &DomainFrame,
    domain: &SdtmDomain,
    study_id: &str,
    sdtm_ig_version: &str,
) -> Result<(), ExportError> {
    let options = DatasetXmlOptions {
        dataset_name: Some(frame.dataset_name()),
        ..Default::default()
    };

    write_dataset_xml_output(
        path,
        domain,
        frame,
        study_id,
        sdtm_ig_version,
        Some(&options),
    )
    .map_err(|e| ExportError::new(format!("Failed to write Dataset-XML: {}", e)))?;

    Ok(())
}

/// Write Define-XML file.
fn write_define_xml(
    path: &Path,
    study_id: &str,
    domain_data: &[DomainExportData],
    domain_frames: &[DomainFrame],
    supp_frames: &[DomainFrame],
    sdtm_ig_version: &str,
) -> Result<(), ExportError> {
    // Collect all domains and frames
    let mut domains: Vec<SdtmDomain> = domain_data.iter().map(|d| d.definition.clone()).collect();
    let mut all_frames: Vec<DomainFrame> = domain_frames.to_vec();

    // Add SUPP domains
    for supp_frame in supp_frames {
        let parent_code = supp_frame
            .domain_code
            .strip_prefix("SUPP")
            .or_else(|| supp_frame.domain_code.strip_prefix("supp"))
            .unwrap_or(&supp_frame.domain_code)
            .to_uppercase();

        // Find parent domain's label from domain_data
        let parent_label = domain_data
            .iter()
            .find(|d| d.code.to_uppercase() == parent_code)
            .and_then(|d| d.definition.label.as_deref());

        if let Some(supp_domain) = build_supp_domain_definition(&parent_code, parent_label) {
            domains.push(supp_domain);
        }
        all_frames.push(supp_frame.clone());
    }

    let options = DefineXmlOptions::new(sdtm_ig_version, "Submission");

    write_define_xml_output(path, study_id, &domains, &all_frames, &options)
        .map_err(|e| ExportError::new(format!("Failed to write Define-XML: {}", e)))?;

    Ok(())
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Build SUPP domain definition from CDISC standards.
///
/// Loads the SUPPQUAL template from embedded standards and customizes it
/// for the specific parent domain.
///
/// # Arguments
/// * `parent_code` - The parent domain code (e.g., "DM", "AE")
/// * `parent_label` - The parent domain label (e.g., "Demographics", "Adverse Events")
fn build_supp_domain_definition(
    parent_code: &str,
    parent_label: Option<&str>,
) -> Option<SdtmDomain> {
    // Load SUPPQUAL template from standards
    let domains = tss_standards::sdtm_ig::load().ok()?;
    let suppqual = domains.iter().find(|d| d.name == "SUPPQUAL")?;

    // Clone and customize for this specific parent domain
    let mut supp_domain = suppqual.clone();
    supp_domain.name = format!("SUPP{}", parent_code.to_uppercase());

    // Use parent label if available, otherwise fall back to code
    // Per SDTMIG: "Supplemental Qualifiers for [domain name]"
    let label_name = parent_label.unwrap_or(parent_code);
    supp_domain.label = Some(format!("Supplemental Qualifiers for {}", label_name));

    supp_domain.dataset_name = Some(format!("SUPP{}", parent_code.to_uppercase()));

    Some(supp_domain)
}

// =============================================================================
// DOMAIN EXPORT DATA BUILDER
// =============================================================================

/// Build export data from a GUI domain.
///
/// Performs data transformation using the normalization pipeline and
/// builds SUPP DataFrame if the domain has included SUPP columns.
pub fn build_domain_export_data(
    code: &str,
    gui_domain: &DomainState,
    study_id: &str,
    terminology: Option<&TerminologyRegistry>,
) -> Result<DomainExportData, ExportError> {
    // Get CDISC domain definition from mapping state
    let cdisc_domain = gui_domain.mapping.domain().clone();

    // Build NormalizationContext from mapping state
    let mappings: BTreeMap<String, String> = gui_domain
        .mapping
        .all_accepted()
        .iter()
        .map(|(target, (source, _))| (target.clone(), source.clone()))
        .collect();

    let omitted = gui_domain.mapping.all_omitted().clone();

    let mut context = NormalizationContext::new(study_id, code)
        .with_mappings(mappings)
        .with_omitted(omitted);

    if let Some(registry) = terminology {
        context = context.with_ct_registry(Some(registry.clone()));
    }

    // Execute normalization pipeline to transform source data
    let transformed_data =
        execute_normalization(&gui_domain.source.data, &gui_domain.normalization, &context)
            .map_err(|e| ExportError::for_domain(code, format!("Normalization failed: {}", e)))?;

    // Build SUPP DataFrame if there are included SUPP columns
    let supp_data = build_supp_dataframe(code, gui_domain, study_id, &transformed_data)?;

    Ok(DomainExportData {
        code: code.to_string(),
        definition: cdisc_domain,
        data: transformed_data,
        supp_data,
    })
}

/// Build SUPP DataFrame from domain's supp_config.
fn build_supp_dataframe(
    domain_code: &str,
    gui_domain: &DomainState,
    study_id: &str,
    transformed_data: &DataFrame,
) -> Result<Option<DataFrame>, ExportError> {
    // Get included SUPP columns
    let included: Vec<_> = gui_domain
        .supp_config
        .iter()
        .filter(|(_, config)| config.should_include())
        .collect();

    if included.is_empty() {
        return Ok(None);
    }

    let source_df = &gui_domain.source.data;
    let row_count = source_df.height();

    // SUPP columns: STUDYID, RDOMAIN, USUBJID, IDVAR, IDVARVAL, QNAM, QLABEL, QVAL, QORIG, QEVAL
    let mut studyid_vec: Vec<String> = Vec::new();
    let mut rdomain_vec: Vec<String> = Vec::new();
    let mut usubjid_vec: Vec<String> = Vec::new();
    let mut idvar_vec: Vec<String> = Vec::new();
    let mut idvarval_vec: Vec<String> = Vec::new();
    let mut qnam_vec: Vec<String> = Vec::new();
    let mut qlabel_vec: Vec<String> = Vec::new();
    let mut qval_vec: Vec<String> = Vec::new();
    let mut qorig_vec: Vec<String> = Vec::new();
    let mut qeval_vec: Vec<String> = Vec::new();

    // Get USUBJID from transformed data
    let usubjid_col = transformed_data.column("USUBJID").ok();

    // Determine IDVAR (use SEQ if available, otherwise USUBJID)
    let seq_var = format!("{}SEQ", domain_code);
    let (idvar, idvar_col) = if let Ok(col) = transformed_data.column(&seq_var) {
        (seq_var, Some(col))
    } else {
        ("USUBJID".to_string(), usubjid_col)
    };

    // Build rows for each SUPP column
    for (source_col_name, config) in &included {
        let source_col = match source_df.column(source_col_name) {
            Ok(col) => col,
            Err(_) => continue,
        };

        for row_idx in 0..row_count {
            let usubjid_val = usubjid_col
                .and_then(|col| col.get(row_idx).ok())
                .map(|v| any_value_to_string(&v))
                .unwrap_or_default();

            if usubjid_val.trim().is_empty() {
                continue;
            }

            let qval = source_col
                .get(row_idx)
                .ok()
                .map(|v| any_value_to_string(&v))
                .unwrap_or_default();

            if qval.trim().is_empty() {
                continue;
            }

            let idvarval = idvar_col
                .and_then(|col| col.get(row_idx).ok())
                .map(|v| any_value_to_string(&v))
                .unwrap_or_default();

            studyid_vec.push(study_id.to_string());
            rdomain_vec.push(domain_code.to_uppercase());
            usubjid_vec.push(usubjid_val);
            idvar_vec.push(idvar.clone());
            idvarval_vec.push(idvarval);
            qnam_vec.push(config.qnam.clone());
            qlabel_vec.push(config.qlabel.clone());
            qval_vec.push(qval);
            qorig_vec.push(config.qorig.code().to_string());
            qeval_vec.push(config.qeval.clone().unwrap_or_default());
        }
    }

    if studyid_vec.is_empty() {
        return Ok(None);
    }

    // Build DataFrame
    let supp_df = DataFrame::new(vec![
        Series::new("STUDYID".into(), studyid_vec).into(),
        Series::new("RDOMAIN".into(), rdomain_vec).into(),
        Series::new("USUBJID".into(), usubjid_vec).into(),
        Series::new("IDVAR".into(), idvar_vec).into(),
        Series::new("IDVARVAL".into(), idvarval_vec).into(),
        Series::new("QNAM".into(), qnam_vec).into(),
        Series::new("QLABEL".into(), qlabel_vec).into(),
        Series::new("QVAL".into(), qval_vec).into(),
        Series::new("QORIG".into(), qorig_vec).into(),
        Series::new("QEVAL".into(), qeval_vec).into(),
    ])
    .map_err(|e| ExportError::for_domain(domain_code, format!("Failed to build SUPP: {}", e)))?;

    Ok(Some(supp_df))
}

/// Convert AnyValue to String.
fn any_value_to_string(value: &AnyValue) -> String {
    match value {
        AnyValue::Null => String::new(),
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        AnyValue::Int64(n) => n.to_string(),
        AnyValue::Int32(n) => n.to_string(),
        AnyValue::Float64(n) => {
            if n.fract() == 0.0 {
                format!("{:.0}", n)
            } else {
                n.to_string()
            }
        }
        AnyValue::Float32(n) => {
            if n.fract() == 0.0 {
                format!("{:.0}", n)
            } else {
                n.to_string()
            }
        }
        other => format!("{}", other),
    }
}

// =============================================================================
// VALIDATION
// =============================================================================

/// Validate all domains and return a map of domain codes to error counts.
///
/// Returns empty map if all domains pass validation.
fn validate_all_domains(input: &ExportInput) -> Vec<(String, usize)> {
    let mut errors = Vec::new();

    for domain_data in &input.domains {
        let not_collected = input
            .not_collected
            .get(&domain_data.code)
            .cloned()
            .unwrap_or_default();

        let report = tss_submit::validate_domain_with_not_collected(
            &domain_data.definition,
            &domain_data.data,
            input.ct_registry.as_ref(),
            &not_collected,
        );

        let error_count = count_validation_errors(&report);
        if error_count > 0 {
            errors.push((domain_data.code.clone(), error_count));
        }
    }

    errors
}

/// Count validation errors (not warnings) in a report.
fn count_validation_errors(report: &ValidationReport) -> usize {
    report
        .issues
        .iter()
        .filter(|issue| matches!(issue.severity(), Severity::Error | Severity::Reject))
        .count()
}

// =============================================================================
// SUPP HELPERS
// =============================================================================

/// Check if a domain has any included SUPP columns.
pub fn domain_has_supp(gui_domain: &DomainState) -> bool {
    gui_domain
        .supp_config
        .values()
        .any(SuppColumnConfig::should_include)
}

/// Get count of included SUPP columns for a domain.
pub fn domain_supp_count(gui_domain: &DomainState) -> usize {
    gui_domain
        .supp_config
        .values()
        .filter(|config| config.should_include())
        .count()
}
