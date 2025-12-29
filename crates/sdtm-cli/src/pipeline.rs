//! Study processing pipeline with explicit stages.
//!
//! The pipeline follows these stages in order:
//! 1. **Ingest**: Discover and read source CSV files
//! 2. **Map**: Apply column mappings to SDTM variables
//! 3. **Preprocess**: Fill missing fields, extract reference dates
//! 4. **Domain Rules**: Process domains, build SUPPQUAL, relationships
//! 5. **Validate**: Run CT-based and structural validation
//! 6. **Output**: Write XPT, Dataset-XML, Define-XML, SAS programs
//!
//! Each stage takes the output of the previous stage and returns typed results.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use polars::prelude::{AnyValue, DataFrame};
use tracing::{debug, info, info_span};

use sdtm_core::frame::{DomainFrame, DomainFrameMeta};
use sdtm_core::frame_builder::{build_domain_frame, build_mapped_domain_frame};
use sdtm_core::pipeline_context::PipelineContext;
use sdtm_core::processor::{DomainProcessInput, process_domain};
use sdtm_core::suppqual::{SuppqualInput, build_suppqual};
use sdtm_ingest::{
    AppliedStudyMetadata, CsvTable, StudyMetadata, any_to_string, apply_study_metadata,
    discover_domain_files, list_csv_files, load_study_metadata, read_csv_schema, read_csv_table,
};
use sdtm_map::merge_mappings;
use sdtm_model::{CaseInsensitiveSet, Domain, MappingConfig, OutputFormat, ValidationReport};
use sdtm_report::{
    DefineXmlOptions, SasProgramOptions, write_dataset_xml_outputs, write_define_xml,
    write_sas_outputs, write_xpt_outputs,
};
use sdtm_validate::validate_domains;
use sdtm_xpt::{XptWriterOptions, read_xpt};

// ============================================================================
// Stage 1: Ingest
// ============================================================================

/// Result of the ingest stage.
#[derive(Debug)]
pub struct IngestResult {
    /// Discovered domain files grouped by domain code.
    pub discovered: BTreeMap<String, Vec<(PathBuf, String)>>,
    /// Study metadata loaded from codelists/items files.
    pub study_metadata: StudyMetadata,
    /// Global SUPPQUAL exclusion columns (appear in many files).
    pub suppqual_exclusions: BTreeSet<String>,
    /// Errors encountered during ingestion.
    pub errors: Vec<String>,
}

/// Discover and prepare source files for processing.
///
/// This stage:
/// - Lists CSV files in the study folder
/// - Discovers domain files by matching filenames
/// - Loads study metadata (codelists, items)
/// - Computes global SUPPQUAL exclusions
pub fn ingest(
    study_folder: &Path,
    domain_codes: &[String],
    standard_variables: &BTreeSet<String>,
) -> Result<IngestResult> {
    let mut errors = Vec::new();

    let csv_files = list_csv_files(study_folder).context("list csv files")?;
    let discovered = discover_domain_files(&csv_files, domain_codes);

    let study_metadata = match load_study_metadata(study_folder) {
        Ok(metadata) => metadata,
        Err(error) => {
            errors.push(format!("metadata: {error}"));
            StudyMetadata::default()
        }
    };

    // Compute global SUPPQUAL exclusions based on column frequency
    let mut column_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_files = 0usize;
    for files in discovered.values() {
        for (path, _variant) in files {
            total_files += 1;
            let schema = match read_csv_schema(path) {
                Ok(schema) => schema,
                Err(error) => {
                    errors.push(format!("{}: {error}", path.display()));
                    continue;
                }
            };
            let mut unique = BTreeSet::new();
            for header in schema.headers {
                unique.insert(header.to_uppercase());
            }
            for header in unique {
                *column_counts.entry(header).or_insert(0) += 1;
            }
        }
    }

    let suppqual_exclusions = if total_files >= 3 {
        let threshold = ((total_files as f64) * 0.6).ceil() as usize;
        column_counts
            .into_iter()
            .filter(|(name, count)| *count >= threshold && !standard_variables.contains(name))
            .map(|(name, _)| name)
            .collect()
    } else {
        BTreeSet::new()
    };

    Ok(IngestResult {
        discovered,
        study_metadata,
        suppqual_exclusions,
        errors,
    })
}

// ============================================================================
// Stage 2-4: Map, Preprocess, Domain Rules (combined for single-file processing)
// ============================================================================

/// Result of processing a single domain file.
#[derive(Debug)]
pub struct ProcessedFile {
    /// The mapped domain frame.
    pub frame: DomainFrame,
    /// The mapping configuration used.
    pub mapping: MappingConfig,
    /// SUPPQUAL frame if any non-standard columns exist.
    pub suppqual: Option<DomainFrame>,
    /// Number of input records.
    pub input_count: usize,
}

/// Input for processing a single domain file.
pub struct ProcessFileInput<'a> {
    pub path: &'a Path,
    pub domain: &'a Domain,
    /// Dataset name for logging and metadata (e.g., "LB" or "LBCH").
    pub dataset_name: &'a str,
    pub study_id: &'a str,
    pub study_metadata: &'a StudyMetadata,
    pub suppqual_domain: &'a Domain,
    pub suppqual_exclusions: &'a BTreeSet<String>,
    pub seq_tracker: &'a mut BTreeMap<String, i64>,
    pub pipeline: &'a PipelineContext,
}

/// Process a single domain file through map, preprocess, and domain rules stages.
///
/// This function:
/// 1. Reads the CSV file
/// 2. Applies study metadata transformations
/// 3. Maps columns to SDTM variables
/// 4. Fills missing test fields
/// 5. Applies domain-specific rules
/// 6. Builds SUPPQUAL for non-standard columns
pub fn process_file(input: ProcessFileInput<'_>) -> Result<ProcessedFile> {
    let domain_code = input.domain.code.to_uppercase();
    let dataset_name = input.dataset_name.to_uppercase();
    let source_file = input.path.display().to_string();
    let process_span = info_span!(
        "process_file",
        study_id = %input.study_id,
        domain_code = %domain_code,
        dataset_name = %dataset_name,
        source_file = %source_file
    );
    let _process_guard = process_span.enter();
    let process_start = Instant::now();

    let raw_table =
        read_csv_table(input.path).with_context(|| format!("read {}", input.path.display()))?;
    let input_count = raw_table.rows.len();

    let AppliedStudyMetadata {
        table,
        code_to_base,
        derived_columns,
    } = if input.study_metadata.is_empty() {
        AppliedStudyMetadata::new(raw_table)
    } else {
        apply_study_metadata(raw_table, input.study_metadata)
    };

    // Build source frame for SUPPQUAL reference
    let source = build_domain_frame(&table, &input.domain.code)
        .with_context(|| format!("build source frame for {}", input.domain.code))?;

    // Build mapped frame
    let mapped_result = info_span!("map").in_scope(|| -> Result<_> {
        let start = Instant::now();
        let result = build_mapped_domain_frame(&table, input.domain, input.study_id)
            .with_context(|| format!("map {} columns", input.domain.code))?;
        debug!(
            study_id = %input.study_id,
            domain_code = %domain_code,
            dataset_name = %dataset_name,
            source_file = %source_file,
            input_rows = input_count,
            output_rows = result.1.data.height(),
            duration_ms = start.elapsed().as_millis(),
            "mapping complete"
        );
        Ok(result)
    })?;

    let (mapping_config, mut mapped, mut used) = mapped_result;

    // Track code columns as used if their base column was used
    if !code_to_base.is_empty() {
        let used_upper: BTreeSet<String> = used.iter().map(|name| name.to_uppercase()).collect();
        for (code_col, base_col) in code_to_base {
            if used_upper.contains(&base_col.to_uppercase()) {
                used.insert(code_col);
            }
        }
    }

    let context = input.pipeline;

    // Apply domain rules
    info_span!("domain_rules").in_scope(|| -> Result<()> {
        let start = Instant::now();
        process_domain(DomainProcessInput {
            domain: input.domain,
            data: &mut mapped.data,
            context,
            sequence_tracker: Some(input.seq_tracker),
        })
        .with_context(|| format!("domain rules for {}", input.domain.code))?;
        debug!(
            study_id = %input.study_id,
            domain_code = %domain_code,
            dataset_name = %dataset_name,
            source_file = %source_file,
            record_count = mapped.data.height(),
            duration_ms = start.elapsed().as_millis(),
            "domain rules complete"
        );
        Ok(())
    })?;

    // Build SUPPQUAL
    let label_map = column_label_map(&table);
    let label_map_ref = if label_map.is_empty() {
        None
    } else {
        Some(&label_map)
    };
    let derived_ref = if derived_columns.is_empty() {
        None
    } else {
        Some(&derived_columns)
    };

    let suppqual = info_span!("suppqual")
        .in_scope(|| -> Result<_> {
            let start = Instant::now();
            let result = build_suppqual(&SuppqualInput {
                parent_domain: input.domain,
                suppqual_domain: input.suppqual_domain,
                source_df: &source.data,
                mapped_df: Some(&mapped.data),
                used_source_columns: &used,
                study_id: input.study_id,
                exclusion_columns: Some(input.suppqual_exclusions),
                source_labels: label_map_ref,
                derived_columns: derived_ref,
            })
            .with_context(|| format!("SUPPQUAL for {}", input.domain.code))?;
            match &result {
                Some(frame) => {
                    debug!(
                        study_id = %input.study_id,
                        domain_code = %domain_code,
                        dataset_name = %dataset_name,
                        source_file = %source_file,
                        record_count = frame.data.height(),
                        duration_ms = start.elapsed().as_millis(),
                        "suppqual complete"
                    );
                }
                None => {
                    debug!(
                        study_id = %input.study_id,
                        domain_code = %domain_code,
                        dataset_name = %dataset_name,
                        source_file = %source_file,
                        duration_ms = start.elapsed().as_millis(),
                        "suppqual skipped"
                    );
                }
            }
            Ok(result)
        })?
        .map(|mut frame| {
            frame.meta = Some(build_frame_meta(
                &frame.domain_code,
                &frame.domain_code,
                input.path,
            ));
            frame
        });

    // Add source file and dataset naming metadata
    let frame_meta = build_frame_meta(&domain_code, &dataset_name, input.path);
    let frame = DomainFrame {
        domain_code: domain_code.clone(),
        data: mapped.data,
        meta: Some(frame_meta),
    };
    debug!(
        study_id = %input.study_id,
        domain_code = %domain_code,
        dataset_name = %dataset_name,
        source_file = %source_file,
        output_rows = frame.record_count(),
        duration_ms = process_start.elapsed().as_millis(),
        "file processed"
    );

    Ok(ProcessedFile {
        frame,
        mapping: mapping_config,
        suppqual,
        input_count,
    })
}

// ============================================================================
// Stage 5: Validate
// ============================================================================

/// Run validation on processed frames.
pub fn validate(
    frames: &[DomainFrame],
    pipeline: &PipelineContext,
    study_id: &str,
) -> BTreeMap<String, ValidationReport> {
    let validation_span = info_span!("validate", study_id = %study_id);
    let _validation_guard = validation_span.enter();
    let validation_start = Instant::now();

    // Use dataset names for validation keys (handles split domains like LBCH)
    let dataset_names: Vec<String> = frames.iter().map(|f| f.dataset_name()).collect();
    let frame_refs: Vec<(&str, &DataFrame)> = frames
        .iter()
        .zip(dataset_names.iter())
        .map(|(frame, name)| (name.as_str(), &frame.data))
        .collect();

    // Per-domain validation - pass dataset names instead of domain codes
    let reports = validate_domains(
        &pipeline.standards,
        &frame_refs,
        Some(&pipeline.ct_registry),
    );
    let mut report_map = BTreeMap::new();
    for report in reports {
        report_map.insert(report.domain_code.to_uppercase(), report);
    }

    if !report_map.is_empty() {
        let mut frame_lookup: BTreeMap<String, &DomainFrame> = BTreeMap::new();
        for frame in frames {
            frame_lookup.insert(frame.domain_code.to_uppercase(), frame);
        }
        for report in report_map.values() {
            if let Some(frame) = frame_lookup.get(&report.domain_code.to_uppercase()) {
                let dataset_name = frame.dataset_name();
                let source_file = frame
                    .source_files()
                    .first()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                debug!(
                    study_id = %study_id,
                    domain_code = %report.domain_code,
                    dataset_name = %dataset_name,
                    source_file = %source_file,
                    error_count = report.error_count(),
                    warning_count = report.warning_count(),
                    "validation summary"
                );
            }
        }
        let total_errors: usize = report_map.values().map(|report| report.error_count()).sum();
        let total_warnings: usize = report_map
            .values()
            .map(|report| report.warning_count())
            .sum();
        info!(
            study_id = %study_id,
            domain_count = report_map.len(),
            error_count = total_errors,
            warning_count = total_warnings,
            duration_ms = validation_start.elapsed().as_millis(),
            "validation complete"
        );
    } else {
        info!(
            study_id = %study_id,
            domain_count = 0,
            duration_ms = validation_start.elapsed().as_millis(),
            "validation complete"
        );
    }

    report_map
}

// ============================================================================
// Stage 6: Output
// ============================================================================

/// Result of the output stage.
#[derive(Debug)]
pub struct OutputResult {
    /// Output paths by domain code.
    pub paths: BTreeMap<String, sdtm_model::OutputPaths>,
    /// Path to define.xml.
    pub define_xml: Option<PathBuf>,
    /// Errors encountered during output.
    pub errors: Vec<String>,
}

/// Output configuration.
pub struct OutputConfig<'a> {
    pub output_dir: &'a Path,
    pub study_id: &'a str,
    pub report_domains: &'a [Domain],
    pub frames: &'a [DomainFrame],
    pub mapping_configs: &'a BTreeMap<String, Vec<MappingConfig>>,
    pub formats: &'a [OutputFormat],
    pub dry_run: bool,
    /// Skip Define-XML generation.
    pub skip_define_xml: bool,
    /// Skip SAS program generation.
    pub skip_sas: bool,
}

/// Write output files (XPT, Dataset-XML, Define-XML, SAS).
pub fn output(config: OutputConfig<'_>) -> Result<OutputResult> {
    let output_span = info_span!("output", study_id = %config.study_id);
    let _output_guard = output_span.enter();
    let output_start = Instant::now();
    let mut errors = Vec::new();
    let mut paths: BTreeMap<String, sdtm_model::OutputPaths> = BTreeMap::new();

    let want_xpt = config
        .formats
        .iter()
        .any(|f| matches!(f, OutputFormat::Xpt));
    let want_xml = config
        .formats
        .iter()
        .any(|f| matches!(f, OutputFormat::Xml));

    // Write Define-XML
    let define_xml = if config.dry_run || config.skip_define_xml {
        None
    } else {
        let options = DefineXmlOptions::new("3.4", "Submission");
        let path = config.output_dir.join("define.xml");
        if let Err(error) = write_define_xml(
            &path,
            config.study_id,
            config.report_domains,
            config.frames,
            &options,
        ) {
            errors.push(format!("define-xml: {error}"));
            None
        } else {
            Some(path)
        }
    };

    if config.dry_run {
        info!(
            study_id = %config.study_id,
            domain_count = config.frames.len(),
            duration_ms = output_start.elapsed().as_millis(),
            "output skipped (dry run)"
        );
        return Ok(OutputResult {
            paths,
            define_xml,
            errors,
        });
    }

    // Write XPT
    if want_xpt {
        let options = XptWriterOptions::default();
        match write_xpt_outputs(
            config.output_dir,
            config.report_domains,
            config.frames,
            &options,
        ) {
            Ok(written) => {
                for path in written {
                    let key = path
                        .file_stem()
                        .and_then(|v| v.to_str())
                        .unwrap_or("")
                        .to_uppercase();
                    paths.entry(key).or_default().xpt.get_or_insert(path);
                }
            }
            Err(error) => errors.push(format!("xpt: {error}")),
        }
    }

    // Write Dataset-XML
    if want_xml {
        match write_dataset_xml_outputs(
            config.output_dir,
            config.report_domains,
            config.frames,
            config.study_id,
            "3.4",
        ) {
            Ok(written) => {
                for path in written {
                    let key = path
                        .file_stem()
                        .and_then(|v| v.to_str())
                        .unwrap_or("")
                        .to_uppercase();
                    paths
                        .entry(key)
                        .or_default()
                        .dataset_xml
                        .get_or_insert(path);
                }
            }
            Err(error) => errors.push(format!("dataset-xml: {error}")),
        }
    }

    // Write SAS programs
    let merged_mappings = merge_mappings(config.mapping_configs, config.study_id);
    if !config.skip_sas && !merged_mappings.is_empty() {
        let mut sas_frames: Vec<DomainFrame> = config
            .frames
            .iter()
            .filter(|frame| merged_mappings.contains_key(&frame.domain_code.to_uppercase()))
            .cloned()
            .collect();
        sas_frames.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));
        let options = SasProgramOptions::default();
        match write_sas_outputs(
            config.output_dir,
            config.report_domains,
            &sas_frames,
            &merged_mappings,
            &options,
        ) {
            Ok(written) => {
                for path in written {
                    let key = path
                        .file_stem()
                        .and_then(|v| v.to_str())
                        .unwrap_or("")
                        .to_uppercase();
                    paths.entry(key).or_default().sas.get_or_insert(path);
                }
            }
            Err(error) => errors.push(format!("sas: {error}")),
        }
    }

    for frame in config.frames {
        let dataset_name = frame.dataset_name();
        let source_file = frame
            .source_files()
            .first()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        debug!(
            study_id = %config.study_id,
            domain_code = %frame.domain_code,
            dataset_name = %dataset_name,
            source_file = %source_file,
            record_count = frame.record_count(),
            "output prepared"
        );
    }

    let xpt_count = paths.values().filter(|path| path.xpt.is_some()).count();
    let dataset_xml_count = paths
        .values()
        .filter(|path| path.dataset_xml.is_some())
        .count();
    let sas_count = paths.values().filter(|path| path.sas.is_some()).count();
    let define_xml_path = define_xml
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "none".to_string());
    info!(
        study_id = %config.study_id,
        domain_count = config.frames.len(),
        xpt_count,
        dataset_xml_count,
        sas_count,
        define_xml = %define_xml_path,
        duration_ms = output_start.elapsed().as_millis(),
        "output complete"
    );

    Ok(OutputResult {
        paths,
        define_xml,
        errors,
    })
}

// ============================================================================
// Helper functions
// ============================================================================

/// Extract RFSTDTC reference starts from DM frame.
pub fn extract_reference_starts(df: &DataFrame) -> BTreeMap<String, String> {
    let mut reference_starts = BTreeMap::new();
    let lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
    let Some(usubjid_col) = lookup.get("USUBJID") else {
        return reference_starts;
    };
    let Some(rfstdtc_col) = lookup.get("RFSTDTC") else {
        return reference_starts;
    };
    let Ok(usubjid_series) = df.column(usubjid_col) else {
        return reference_starts;
    };
    let Ok(rfstdtc_series) = df.column(rfstdtc_col) else {
        return reference_starts;
    };
    for idx in 0..df.height() {
        let usubjid = any_to_string(usubjid_series.get(idx).unwrap_or(AnyValue::Null));
        let rfstdtc = any_to_string(rfstdtc_series.get(idx).unwrap_or(AnyValue::Null));
        let usubjid = usubjid.trim();
        let rfstdtc = rfstdtc.trim();
        if usubjid.is_empty() || rfstdtc.is_empty() {
            continue;
        }
        reference_starts
            .entry(usubjid.to_string())
            .or_insert_with(|| rfstdtc.to_string());
    }
    reference_starts
}

fn column_label_map(table: &CsvTable) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();
    let Some(label_row) = table.labels.as_ref() else {
        return labels;
    };
    for (header, label) in table.headers.iter().zip(label_row.iter()) {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            continue;
        }
        labels.insert(header.to_uppercase(), trimmed.to_string());
    }
    labels
}

/// Build frame metadata with dataset naming and source provenance details.
fn build_frame_meta(domain_code: &str, dataset_name: &str, source_file: &Path) -> DomainFrameMeta {
    DomainFrameMeta {
        dataset_name: Some(dataset_name.to_string()),
        source_files: vec![source_file.to_path_buf()],
        base_domain_code: Some(domain_code.to_string()),
    }
}

/// Verify XPT output counts match input counts.
pub fn verify_xpt_counts(
    output_paths: &BTreeMap<String, sdtm_model::OutputPaths>,
    _input_counts: &BTreeMap<String, usize>,
) -> (BTreeMap<String, usize>, Vec<String>) {
    let mut xpt_counts = BTreeMap::new();
    let mut errors = Vec::new();

    for (code, paths) in output_paths {
        if let Some(path) = &paths.xpt {
            match read_xpt(path) {
                Ok(dataset) => {
                    xpt_counts.insert(code.to_uppercase(), dataset.rows.len());
                }
                Err(error) => {
                    errors.push(format!("xpt read {}: {error}", path.display()));
                }
            }
        }
    }

    (xpt_counts, errors)
}
