//! Export service - async export with progress streaming.
//!
//! Clean Iced-native implementation using `Task::perform` pattern.
//! All export logic is self-contained with no legacy dependencies.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use polars::prelude::{AnyValue, DataFrame, DataType};
use tss_model::{Domain, VariableType};
use tss_output::types::DomainFrame;
use tss_output::{
    DatasetXmlOptions, DefineXmlOptions, write_dataset_xml as write_dataset_xml_output,
    write_define_xml as write_define_xml_output,
};
use xportrs::{Column, ColumnData, Dataset, Xpt};

use crate::state::{ExportFormat, ExportResult, XptVersion};

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
    /// Domains to export with their data.
    pub domains: Vec<DomainExportData>,
    /// Study ID (extracted from data or default).
    pub study_id: String,
}

/// Data for a single domain to export.
#[derive(Clone)]
pub struct DomainExportData {
    /// Domain code (e.g., "DM", "AE").
    pub code: String,
    /// Domain definition from CDISC standards.
    pub definition: Domain,
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

        if let Err(e) = write_data_file(
            &path,
            &frame,
            &domain_data.definition,
            &input.study_id,
            input.format,
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
            if let Some(supp_def) = build_supp_domain_definition(&domain_data.code) {
                if let Err(e) = write_data_file(
                    &supp_path,
                    &supp_frame,
                    &supp_def,
                    &input.study_id,
                    input.format,
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
    if let Err(e) = write_define_xml(
        &define_path,
        &input.study_id,
        &input.domains,
        &domain_frames,
        &supp_frames,
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
    domain: &Domain,
    study_id: &str,
    format: ExportFormat,
) -> Result<(), ExportError> {
    match format {
        ExportFormat::Xpt => write_xpt_file(path, frame, domain),
        ExportFormat::DatasetXml => write_dataset_xml_file(path, frame, domain, study_id),
    }
}

/// Write XPT file.
fn write_xpt_file(path: &Path, frame: &DomainFrame, domain: &Domain) -> Result<(), ExportError> {
    let row_count = frame.data.height();

    // Build variable metadata from domain definition
    let variable_metadata: HashMap<String, VariableMetadata> = domain
        .variables
        .iter()
        .map(|v| {
            let name = v.name.to_uppercase();
            let label = v.label.clone();
            let format = match v.data_type {
                VariableType::Char => {
                    let len = v.length.unwrap_or(200).min(200);
                    Some(format!("${len}."))
                }
                VariableType::Num => Some("8.".to_string()),
            };
            (name, VariableMetadata { label, format })
        })
        .collect();

    // Build columns
    let mut columns = Vec::new();
    for col in frame.data.get_columns() {
        let name = col.name().to_uppercase();
        let dtype = col.dtype();

        let is_numeric = matches!(
            dtype,
            DataType::Float64
                | DataType::Float32
                | DataType::Int64
                | DataType::Int32
                | DataType::Int16
                | DataType::Int8
                | DataType::UInt64
                | DataType::UInt32
                | DataType::UInt16
                | DataType::UInt8
        );

        let column_data = if is_numeric {
            let mut values = Vec::with_capacity(row_count);
            for row_idx in 0..row_count {
                let value = col.get(row_idx).ok();
                let num = match value {
                    Some(AnyValue::Float64(n)) => Some(n),
                    Some(AnyValue::Float32(n)) => Some(n as f64),
                    Some(AnyValue::Int64(n)) => Some(n as f64),
                    Some(AnyValue::Int32(n)) => Some(n as f64),
                    Some(AnyValue::Int16(n)) => Some(n as f64),
                    Some(AnyValue::Int8(n)) => Some(n as f64),
                    Some(AnyValue::UInt64(n)) => Some(n as f64),
                    Some(AnyValue::UInt32(n)) => Some(n as f64),
                    Some(AnyValue::UInt16(n)) => Some(n as f64),
                    Some(AnyValue::UInt8(n)) => Some(n as f64),
                    _ => None,
                };
                values.push(num);
            }
            ColumnData::F64(values)
        } else {
            let mut values = Vec::with_capacity(row_count);
            for row_idx in 0..row_count {
                let value = col.get(row_idx).ok();
                let s = match value {
                    Some(AnyValue::String(s)) => Some(s.to_string()),
                    Some(AnyValue::StringOwned(s)) => Some(s.to_string()),
                    Some(AnyValue::Null) | None => None,
                    Some(other) => Some(format!("{}", other)),
                };
                values.push(s);
            }
            ColumnData::String(values)
        };

        let mut column = Column::new(&name, column_data);

        // Apply metadata
        if let Some(meta) = variable_metadata.get(&name) {
            if let Some(label) = &meta.label {
                column = column.with_label(label.as_str());
            }
            if let Some(format) = &meta.format {
                column = column
                    .with_format_str(format)
                    .expect("auto-generated SAS format should be valid");
            }
        }

        columns.push(column);
    }

    // Create dataset
    let dataset_name = frame.dataset_name().to_uppercase();
    let dataset = Dataset::with_label(dataset_name.as_str(), frame.domain_code.as_str(), columns)
        .map_err(|e| ExportError::new(format!("Failed to create XPT dataset: {}", e)))?;

    // Write file
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
    domain: &Domain,
    study_id: &str,
) -> Result<(), ExportError> {
    let options = DatasetXmlOptions {
        dataset_name: Some(frame.dataset_name()),
        ..Default::default()
    };

    // TODO: Make SDTM-IG version configurable
    let sdtm_ig_version = "3.4";

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
) -> Result<(), ExportError> {
    // Collect all domains and frames
    let mut domains: Vec<Domain> = domain_data.iter().map(|d| d.definition.clone()).collect();
    let mut all_frames: Vec<DomainFrame> = domain_frames.to_vec();

    // Add SUPP domains
    for supp_frame in supp_frames {
        let parent_code = supp_frame
            .domain_code
            .strip_prefix("SUPP")
            .or_else(|| supp_frame.domain_code.strip_prefix("supp"))
            .unwrap_or(&supp_frame.domain_code)
            .to_uppercase();

        if let Some(supp_domain) = build_supp_domain_definition(&parent_code) {
            domains.push(supp_domain);
        }
        all_frames.push(supp_frame.clone());
    }

    // TODO: Make SDTM-IG version configurable
    let options = DefineXmlOptions::new("3.4", "Submission");

    write_define_xml_output(path, study_id, &domains, &all_frames, &options)
        .map_err(|e| ExportError::new(format!("Failed to write Define-XML: {}", e)))?;

    Ok(())
}

// =============================================================================
// HELPER TYPES
// =============================================================================

/// Variable metadata for XPT export.
struct VariableMetadata {
    label: Option<String>,
    format: Option<String>,
}

/// Build SUPP domain definition from CDISC standards.
///
/// Loads the SUPPQUAL template from embedded standards and customizes it
/// for the specific parent domain.
fn build_supp_domain_definition(parent_code: &str) -> Option<Domain> {
    // Load SUPPQUAL template from standards
    let domains = tss_standards::sdtm_ig::load().ok()?;
    let suppqual = domains.iter().find(|d| d.name == "SUPPQUAL")?;

    // Clone and customize for this specific parent domain
    let mut supp_domain = suppqual.clone();
    supp_domain.name = format!("SUPP{}", parent_code.to_uppercase());
    supp_domain.label = Some(format!(
        "Supplemental Qualifiers for {}",
        parent_code.to_uppercase()
    ));
    supp_domain.dataset_name = Some(format!("SUPP{}", parent_code.to_uppercase()));

    Some(supp_domain)
}
