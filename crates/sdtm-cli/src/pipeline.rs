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

use anyhow::{Context, Result, anyhow};
use polars::prelude::{AnyValue, DataFrame};
use tracing::{debug, info, info_span};

use sdtm_core::frame_builder::build_mapped_domain_frame;
use sdtm_core::pipeline_context::PipelineContext;
use sdtm_core::processor::{DomainProcessInput, process_domain};
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
use sdtm_standards::{load_default_ct_registry, load_default_sdtm_ig_domains};
use sdtm_transform::domain_sets::{build_report_domains, domain_map_by_code, is_supporting_domain};
use sdtm_transform::frame::{DomainFrame, DomainFrameMeta};
use sdtm_transform::frame_builder::build_domain_frame;
use sdtm_transform::relationships::build_relationship_frames;
use sdtm_transform::suppqual::{SuppqualInput, build_suppqual};
use sdtm_validate::{gate_strict_outputs, validate_domains};
use sdtm_xpt::{XptWriterOptions, read_xpt};

use crate::types::{DomainDataCheck, DomainSummary, StudyResult};

/// Pipeline configuration derived from CLI arguments.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub study_id: String,
    pub study_folder: PathBuf,
    pub output_dir: PathBuf,
    pub output_formats: Vec<OutputFormat>,
}

/// Stateful study pipeline runner.
pub struct PipelineRunner {
    config: PipelineConfig,
    pipeline: PipelineContext,
    standard_variables: BTreeSet<String>,
}

impl PipelineRunner {
    pub fn new(config: PipelineConfig) -> Result<Self> {
        let standards = load_default_sdtm_ig_domains().context("load standards")?;
        let ct_registry = load_default_ct_registry().context("load ct registry")?;
        let pipeline = PipelineContext::new(&config.study_id)
            .with_standards(standards.clone())
            .with_ct_registry(ct_registry);

        let mut standard_variables = BTreeSet::new();
        for domain in &pipeline.standards {
            for variable in &domain.variables {
                standard_variables.insert(variable.name.to_uppercase());
            }
        }

        Ok(Self {
            config,
            pipeline,
            standard_variables,
        })
    }

    pub fn run(mut self) -> Result<StudyResult> {
        let study_folder = &self.config.study_folder;
        let study_id = self.config.study_id.clone();
        let output_dir = self.config.output_dir.clone();
        let want_xpt = self.config.output_formats.contains(&OutputFormat::Xpt);

        // =========================================================================
        // Stage 1: Ingest - Discover files, load metadata, compute exclusions
        // =========================================================================
        let ingest_span = info_span!(
            "ingest",
            study_id = %study_id,
            study_folder = %study_folder.display()
        );
        let ingest_start = Instant::now();
        let IngestResult {
            discovered,
            study_metadata,
            suppqual_exclusions,
            errors: ingest_errors,
        } = ingest_span.in_scope(|| self.ingest())?;
        let file_count: usize = discovered.values().map(std::vec::Vec::len).sum();
        info!(
            study_id = %study_id,
            domain_count = discovered.len(),
            file_count,
            duration_ms = ingest_start.elapsed().as_millis(),
            "ingest complete"
        );

        let mut errors = ingest_errors;

        let suppqual_domain = self
            .pipeline
            .get_domain("SUPPQUAL")
            .ok_or_else(|| anyhow!("missing SUPPQUAL metadata"))?
            .clone();

        // =========================================================================
        // Stage 2-4: Map, Preprocess, Domain Rules - Process each domain file
        // =========================================================================
        let ProcessResult {
            frames: frame_list,
            mapping_configs,
            input_counts,
            errors: process_errors,
        } = self.process_domains(
            &discovered,
            &study_metadata,
            &suppqual_exclusions,
            &suppqual_domain,
        )?;
        errors.extend(process_errors);

        // Build report domains for output
        let report_domains = build_report_domains(&self.pipeline.standards, &frame_list)?;
        let report_domain_map = domain_map_by_code(&report_domains);

        // =========================================================================
        // Stage 5: Validate - Conformance via CT + structural checks
        // =========================================================================
        let mut report_map = self.validate(&frame_list, &study_id);

        let output_formats = &self.config.output_formats;

        // =========================================================================
        // Stage 5.5: Gate outputs - Block strict outputs if validation fails
        // =========================================================================
        let conformance_reports: Vec<_> = report_map.values().cloned().collect();
        let gating = gate_strict_outputs(output_formats, true, &conformance_reports);
        let allow_xpt = !gating.blocks_output();

        if !allow_xpt {
            tracing::warn!(
                study_id = %study_id,
                blocked_domains = ?gating.blocking_domains,
                "strict output blocked due to conformance errors"
            );
            errors.push(format!(
                "XPT output blocked: conformance errors in domains: {}.",
                gating.blocking_domains.join(", ")
            ));
        }

        // =========================================================================
        // Stage 6: Output - Write XPT, Dataset-XML, Define-XML, SAS
        // =========================================================================
        let output_result = self.output(OutputInput {
            report_domains: &report_domains,
            frames: &frame_list,
            mapping_configs: &mapping_configs,
            formats: output_formats,
            allow_xpt,
        })?;
        let mut output_paths = output_result.paths;
        let define_xml = output_result.define_xml;
        errors.extend(output_result.errors);

        // =========================================================================
        // Post-processing: Verify XPT counts and build summaries
        // =========================================================================
        let mut data_checks = Vec::new();
        if want_xpt && allow_xpt {
            let (xpt_counts, xpt_errors) = verify_xpt_counts(&output_paths);
            errors.extend(xpt_errors);

            if !xpt_counts.is_empty() {
                let mut check_keys: BTreeSet<String> = BTreeSet::new();
                check_keys.extend(input_counts.keys().cloned());
                check_keys.extend(xpt_counts.keys().cloned());
                for key in check_keys {
                    data_checks.push(DomainDataCheck {
                        domain_code: key.clone(),
                        csv_rows: input_counts.get(&key).copied().unwrap_or(0),
                        xpt_rows: xpt_counts.get(&key).copied(),
                    });
                }
            }
        }

        // Build domain summaries
        let mut summaries = Vec::new();
        for frame in &frame_list {
            let dataset_name = frame.dataset_name();
            let base_code = frame.domain_code.to_uppercase();
            let domain = report_domain_map.get(&base_code);
            let description = domain
                .and_then(|d| d.description.clone().or(d.label.clone()))
                .unwrap_or_default();
            let outputs = output_paths.remove(&dataset_name).unwrap_or_default();
            let conformance = report_map.remove(&dataset_name);
            summaries.push(DomainSummary {
                domain_code: dataset_name,
                description,
                records: frame.record_count(),
                outputs,
                conformance,
            });
        }
        if !report_map.is_empty() {
            for (code, report) in report_map {
                let domain = report_domain_map.get(&code);
                let description = domain
                    .and_then(|d| d.description.clone().or(d.label.clone()))
                    .unwrap_or_default();
                summaries.push(DomainSummary {
                    domain_code: code,
                    description,
                    records: 0,
                    outputs: sdtm_model::OutputPaths::default(),
                    conformance: Some(report),
                });
            }
        }

        let has_errors = !errors.is_empty()
            || summaries.iter().any(|summary| {
                summary
                    .conformance
                    .as_ref()
                    .map(ValidationReport::has_errors)
                    .unwrap_or(false)
            });

        Ok(StudyResult {
            study_id,
            output_dir,
            domains: summaries,
            data_checks,
            errors,
            define_xml,
            has_errors,
        })
    }
}

// ============================================================================
// Stage 1: Ingest
// ============================================================================

/// Result of the ingest stage.
#[derive(Debug)]
struct IngestResult {
    /// Discovered domain files grouped by domain code.
    discovered: BTreeMap<String, Vec<(PathBuf, String)>>,
    /// Study metadata loaded from codelists/items files.
    study_metadata: StudyMetadata,
    /// Global SUPPQUAL exclusion columns (appear in many files).
    suppqual_exclusions: BTreeSet<String>,
    /// Errors encountered during ingestion.
    errors: Vec<String>,
}

impl PipelineRunner {
/// Discover and prepare source files for processing.
///
/// This stage:
/// - Lists CSV files in the study folder
/// - Discovers domain files by matching filenames
/// - Loads study metadata (codelists, items)
/// - Computes global SUPPQUAL exclusions
fn ingest(&self) -> Result<IngestResult> {
    let mut errors = Vec::new();

    let csv_files = list_csv_files(&self.config.study_folder).context("list csv files")?;
    let domain_codes = self.pipeline.domain_codes();
    let discovered = discover_domain_files(&csv_files, &domain_codes);

    let study_metadata = match load_study_metadata(&self.config.study_folder) {
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
            .filter(|(name, count)| {
                *count >= threshold && !self.standard_variables.contains(name)
            })
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

}

// ============================================================================
// Stage 2-4: Map, Preprocess, Domain Rules (combined for single-file processing)
// ============================================================================

/// Result of processing a single domain file.
#[derive(Debug)]
struct ProcessedFile {
    /// The mapped domain frame.
    pub frame: DomainFrame,
    /// The mapping configuration used.
    pub mapping: MappingConfig,
    /// SUPPQUAL frame if any non-standard columns exist.
    pub suppqual: Option<DomainFrame>,
    /// Number of input records.
    pub input_count: usize,
}

/// Result of processing all domain files.
struct ProcessResult {
    frames: Vec<DomainFrame>,
    mapping_configs: BTreeMap<String, Vec<MappingConfig>>,
    input_counts: BTreeMap<String, usize>,
    errors: Vec<String>,
}

/// Input for processing a single domain file.
struct ProcessFileInput<'a> {
    pub path: &'a Path,
    pub domain: &'a Domain,
    /// Dataset name for logging and metadata (e.g., "LB" or "LBCH").
    pub dataset_name: &'a str,
    pub study_metadata: &'a StudyMetadata,
    pub suppqual_domain: &'a Domain,
    pub suppqual_exclusions: &'a BTreeSet<String>,
    pub seq_tracker: &'a mut BTreeMap<String, i64>,
}

impl PipelineRunner {
/// Process a single domain file through map, preprocess, and domain rules stages.
///
/// This function:
/// 1. Reads the CSV file
/// 2. Applies study metadata transformations
/// 3. Maps columns to SDTM variables
/// 4. Fills missing test fields
/// 5. Applies domain-specific rules
/// 6. Builds SUPPQUAL for non-standard columns
fn process_file(&self, input: &mut ProcessFileInput<'_>) -> Result<ProcessedFile> {
    let study_id = self.config.study_id.as_str();
    let domain_code = input.domain.code.to_uppercase();
    let dataset_name = input.dataset_name.to_uppercase();
    let source_file = input.path.display().to_string();
    let process_span = info_span!(
        "process_file",
        study_id = %study_id,
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
        let result = build_mapped_domain_frame(&table, input.domain, study_id)
            .with_context(|| format!("map {} columns", input.domain.code))?;
        debug!(
            study_id = %study_id,
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

    let context = &self.pipeline;

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
            study_id = %study_id,
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
                study_id,
                exclusion_columns: Some(input.suppqual_exclusions),
                source_labels: label_map_ref,
                derived_columns: derived_ref,
            })
            .with_context(|| format!("SUPPQUAL for {}", input.domain.code))?;
            match &result {
                Some(frame) => {
                    debug!(
                        study_id = %study_id,
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
                        study_id = %study_id,
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
        study_id = %study_id,
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

fn process_domains(
    &mut self,
    discovered: &BTreeMap<String, Vec<(PathBuf, String)>>,
    study_metadata: &StudyMetadata,
    suppqual_exclusions: &BTreeSet<String>,
    suppqual_domain: &Domain,
) -> Result<ProcessResult> {
    let study_id = self.config.study_id.as_str();
    let mut errors = Vec::new();
    let mut processed_frames: BTreeMap<String, DomainFrame> = BTreeMap::new();
    let mut suppqual_frames: Vec<DomainFrame> = Vec::new();
    let mut mapping_configs: BTreeMap<String, Vec<MappingConfig>> = BTreeMap::new();
    let mut input_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut seq_trackers: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();

    // Order domains with DM first (needed for reference dates)
    let mut ordered_domains: Vec<String> = discovered.keys().cloned().collect();
    ordered_domains.sort_by(|left, right| {
        let left_dm = left.eq_ignore_ascii_case("DM");
        let right_dm = right.eq_ignore_ascii_case("DM");
        match (left_dm, right_dm) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => left.cmp(right),
        }
    });

    let process_start = Instant::now();
    for domain_code in ordered_domains {
        let Some(files) = discovered.get(&domain_code) else {
            continue;
        };
        let multi_source = files.len() > 1;
        let domain_key = domain_code.to_uppercase();

        if multi_source {
            info!(
                study_id = %study_id,
                domain_code = %domain_key,
                file_count = files.len(),
                "processing domain"
            );
        } else if let Some((path, _)) = files.first() {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            info!(
                study_id = %study_id,
                domain_code = %domain_key,
                source_filename = %filename,
                "processing domain"
            );
        }

        let domain = match self.pipeline.get_domain(&domain_key).cloned() {
            Some(domain) => domain,
            None => {
                errors.push(format!("missing standards metadata for {domain_code}"));
                continue;
            }
        };
        let domain_tracker = seq_trackers.entry(domain_key.clone()).or_default();
        let mut combined: Option<DataFrame> = None;
        let mut domain_mappings = Vec::new();

        for (path, variant) in files {
            if multi_source {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                debug!(
                    study_id = %study_id,
                    domain_code = %domain_key,
                    source_filename = %filename,
                    "processing file"
                );
            }

            let mut input = ProcessFileInput {
                path,
                domain: &domain,
                dataset_name: variant.as_str(),
                study_metadata,
                suppqual_domain,
                suppqual_exclusions,
                seq_tracker: domain_tracker,
            };
            let result = self.process_file(&mut input);

            match result {
                Ok(processed) => {
                    *input_counts.entry(domain_key.clone()).or_insert(0) += processed.input_count;

                    if let Some(suppqual) = processed.suppqual {
                        suppqual_frames.push(suppqual);
                    }

                    domain_mappings.push(processed.mapping);

                    if domain_key == "DM" {
                        let dm_starts = extract_reference_starts(&processed.frame.data);
                        self.pipeline.add_reference_starts(dm_starts);
                    }

                    if multi_source {
                        if let Some(existing) = combined.as_mut() {
                            existing
                                .vstack_mut(&processed.frame.data)
                                .with_context(|| format!("merge {domain_code} frames"))?;
                        } else {
                            combined = Some(processed.frame.data);
                        }
                    } else if let Err(error) = insert_frame(&mut processed_frames, processed.frame) {
                        errors.push(format!("{}: {error}", path.display()));
                    }
                }
                Err(error) => {
                    errors.push(format!("{}: {error}", path.display()));
                }
            }
        }

        if let Some(data) = combined {
            let key = domain.code.to_uppercase();
            processed_frames.insert(
                key.clone(),
                DomainFrame {
                    domain_code: key,
                    data,
                    meta: None,
                },
            );
        }
        if !domain_mappings.is_empty() {
            mapping_configs
                .entry(domain.code.to_uppercase())
                .or_default()
                .extend(domain_mappings);
        }
    }

    // Merge SUPPQUAL frames
    let mut frames = processed_frames;
    for frame in suppqual_frames {
        if let Err(error) = insert_frame(&mut frames, frame) {
            errors.push(format!("SUPPQUAL merge: {error}"));
        }
    }

    // Build relationship datasets (RELREC, RELSPEC, RELSUB)
    let relationship_sources: Vec<DomainFrame> = frames
        .values()
        .filter(|frame| !is_supporting_domain(&frame.domain_code))
        .cloned()
        .collect();
    let relationship_frames =
        build_relationship_frames(&relationship_sources, &self.pipeline.standards, study_id)
            .context("build relationship domains")?;
    for frame in relationship_frames {
        if let Err(error) = insert_frame(&mut frames, frame) {
            errors.push(format!("relationship merge: {error}"));
        }
    }

    // Sort frames
    let mut frame_list: Vec<DomainFrame> = frames.into_values().collect();
    frame_list.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));
    let total_records: usize = frame_list.iter().map(DomainFrame::record_count).sum();
    info!(
        study_id = %study_id,
        domain_count = frame_list.len(),
        record_count = total_records,
        duration_ms = process_start.elapsed().as_millis(),
        "domain processing complete"
    );

    Ok(ProcessResult {
        frames: frame_list,
        mapping_configs,
        input_counts,
        errors,
    })
}

}

// ============================================================================
// Stage 5: Validate
// ============================================================================

impl PipelineRunner {
/// Run validation on processed frames.
fn validate(&self, frames: &[DomainFrame], study_id: &str) -> BTreeMap<String, ValidationReport> {
    let validation_span = info_span!("validate", study_id = %study_id);
    let _validation_guard = validation_span.enter();
    let validation_start = Instant::now();

    // Use dataset names for validation keys (handles split domains like LBCH)
    let dataset_names: Vec<String> = frames
        .iter()
        .map(sdtm_transform::frame::DomainFrame::dataset_name)
        .collect();
    let frame_refs: Vec<(&str, &DataFrame)> = frames
        .iter()
        .zip(dataset_names.iter())
        .map(|(frame, name)| (name.as_str(), &frame.data))
        .collect();

    // Per-domain validation - pass dataset names instead of domain codes
    let reports = validate_domains(
        &self.pipeline.standards,
        &frame_refs,
        Some(&self.pipeline.ct_registry),
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
        let total_errors: usize = report_map
            .values()
            .map(sdtm_model::ValidationReport::error_count)
            .sum();
        let total_warnings: usize = report_map
            .values()
            .map(sdtm_model::ValidationReport::warning_count)
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

}

// ============================================================================
// Stage 6: Output
// ============================================================================

/// Result of the output stage.
#[derive(Debug)]
struct OutputResult {
    /// Output paths by domain code.
    paths: BTreeMap<String, sdtm_model::OutputPaths>,
    /// Path to define.xml.
    define_xml: Option<PathBuf>,
    /// Errors encountered during output.
    errors: Vec<String>,
}

struct OutputInput<'a> {
    report_domains: &'a [Domain],
    frames: &'a [DomainFrame],
    mapping_configs: &'a BTreeMap<String, Vec<MappingConfig>>,
    formats: &'a [OutputFormat],
    allow_xpt: bool,
}

impl PipelineRunner {
/// Write output files (XPT, Dataset-XML, Define-XML, SAS).
fn output(&self, input: OutputInput<'_>) -> Result<OutputResult> {
    let study_id = self.config.study_id.as_str();
    let output_span = info_span!("output", study_id = %study_id);
    let _output_guard = output_span.enter();
    let output_start = Instant::now();
    let mut errors = Vec::new();
    let mut paths: BTreeMap<String, sdtm_model::OutputPaths> = BTreeMap::new();

    let want_xpt = input.allow_xpt && input.formats.contains(&OutputFormat::Xpt);
    let want_xml = input.formats.contains(&OutputFormat::Xml);
    let want_sas = input.formats.contains(&OutputFormat::Sas);
    let want_define_xml = want_xpt || want_xml;

    // Write Define-XML
    let define_xml = if !want_define_xml {
        None
    } else {
        let options = DefineXmlOptions::new("3.4", "Submission");
        let path = self.config.output_dir.join("define.xml");
        if let Err(error) = write_define_xml(
            &path,
            study_id,
            input.report_domains,
            input.frames,
            &options,
        ) {
            errors.push(format!("define-xml: {error}"));
            None
        } else {
            Some(path)
        }
    };

    // Write XPT
    if want_xpt {
        let options = XptWriterOptions::default();
        match write_xpt_outputs(
            &self.config.output_dir,
            input.report_domains,
            input.frames,
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
            &self.config.output_dir,
            input.report_domains,
            input.frames,
            study_id,
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
    if want_sas {
        let merged_mappings = merge_mappings(input.mapping_configs, study_id);
        if !merged_mappings.is_empty() {
            let mut sas_frames: Vec<DomainFrame> = input
                .frames
                .iter()
                .filter(|frame| merged_mappings.contains_key(&frame.domain_code.to_uppercase()))
                .cloned()
                .collect();
            sas_frames.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));
            let options = SasProgramOptions::default();
            match write_sas_outputs(
                &self.config.output_dir,
                input.report_domains,
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
    }

    for frame in input.frames {
        let dataset_name = frame.dataset_name();
        let source_file = frame
            .source_files()
            .first()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        debug!(
            study_id = %study_id,
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
        study_id = %study_id,
        domain_count = input.frames.len(),
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
fn verify_xpt_counts(
    output_paths: &BTreeMap<String, sdtm_model::OutputPaths>,
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

fn insert_frame(map: &mut BTreeMap<String, DomainFrame>, frame: DomainFrame) -> Result<()> {
    let key = frame.domain_code.to_uppercase();
    if let Some(existing) = map.get_mut(&key) {
        let DomainFrame { data, meta, .. } = frame;
        existing
            .data
            .vstack_mut(&data)
            .with_context(|| format!("merge {key} frames"))?;
        if let Some(meta) = meta {
            for source in meta.source_files {
                existing.add_source_file(source);
            }
        }
    } else {
        map.insert(
            key.clone(),
            DomainFrame {
                domain_code: key,
                data: frame.data,
                meta: frame.meta,
            },
        );
    }
    Ok(())
}
