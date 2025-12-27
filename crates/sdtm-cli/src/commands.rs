use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result, anyhow};
use comfy_table::Table;
use polars::prelude::DataFrame;
use tracing::{debug, info, info_span, warn};

use sdtm_core::{
    DomainFrame, ProcessingOptions, StudyPipelineContext, build_relationship_frames,
    build_report_domains, dedupe_frames_by_identifiers, insert_frame, is_supporting_domain,
};
use sdtm_model::{MappingConfig, OutputFormat};
use sdtm_standards::{
    load_default_ct_registry, load_default_p21_rules, load_default_sdtm_ig_domains,
};
use sdtm_validate::gate_strict_outputs;

use crate::cli::{OutputFormatArg, StudyArgs};
use crate::pipeline::{
    IngestResult, OutputConfig, ProcessFileInput, extract_reference_starts, ingest, output,
    process_file, validate, verify_xpt_counts,
};
use crate::summary::apply_table_style;
use crate::types::{DomainDataCheck, DomainSummary, StudyResult};

pub fn run_domains() -> Result<()> {
    let mut domains = load_default_sdtm_ig_domains().context("load standards")?;
    domains.sort_by(|a, b| a.code.cmp(&b.code));
    let mut table = Table::new();
    table.set_header(vec!["Domain", "Description"]);
    apply_table_style(&mut table);
    for domain in domains {
        let description = domain
            .description
            .clone()
            .or(domain.label.clone())
            .unwrap_or_default();
        table.add_row(vec![domain.code, description]);
    }
    println!("{table}");
    Ok(())
}

pub fn run_study(args: &StudyArgs) -> Result<StudyResult> {
    let study_folder = &args.study_folder;
    let study_id = derive_study_id(study_folder);
    let study_span = info_span!("study", study_id = %study_id);
    let _study_guard = study_span.enter();
    let output_dir = args
        .output_dir
        .clone()
        .unwrap_or_else(|| study_folder.join("output"));
    let output_formats = format_outputs(args.format);
    let want_xpt = output_formats
        .iter()
        .any(|f| matches!(f, OutputFormat::Xpt));

    // =========================================================================
    // Stage 0: Initialize pipeline context
    // =========================================================================
    let standards = load_default_sdtm_ig_domains().context("load standards")?;
    let ct_registry = load_default_ct_registry().context("load ct registry")?;
    let p21_rules = load_default_p21_rules().context("load p21 rules")?;

    // Build processing options based on CLI flags
    // --strict enables all strict mode options
    // Individual flags can also be set independently
    let options = if args.strict {
        ProcessingOptions::strict()
    } else {
        ProcessingOptions {
            prefix_usubjid: !args.no_usubjid_prefix,
            assign_sequence: !args.no_auto_seq,
            warn_on_rewrite: true,
            allow_heuristic_inference: !args.no_heuristic_inference,
            allow_lenient_ct_matching: !args.no_lenient_ct,
            require_explicit_mapping: !args.require_explicit_mapping,
        }
    };
    let mut pipeline = StudyPipelineContext::new(&study_id)
        .with_standards(standards.clone())
        .with_ct_registry(ct_registry)
        .with_p21_rules(p21_rules)
        .with_options(options);

    // Build standard variables set for SUPPQUAL exclusion
    let mut standard_variables = BTreeSet::new();
    for domain in &pipeline.standards {
        for variable in &domain.variables {
            standard_variables.insert(variable.name.to_uppercase());
        }
    }

    // =========================================================================
    // Stage 1: Ingest - Discover files, load metadata, compute exclusions
    // =========================================================================
    let domain_codes = pipeline.domain_codes();
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
    } = ingest_span.in_scope(|| ingest(study_folder, &domain_codes, &standard_variables))?;
    let file_count: usize = discovered.values().map(|files| files.len()).sum();
    info!(
        study_id = %study_id,
        domain_count = discovered.len(),
        file_count,
        duration_ms = ingest_start.elapsed().as_millis(),
        "ingest complete"
    );

    let mut errors = ingest_errors;

    let suppqual_domain = pipeline
        .get_domain("SUPPQUAL")
        .ok_or_else(|| anyhow!("missing SUPPQUAL metadata"))?
        .clone();

    // =========================================================================
    // Stage 2-4: Map, Preprocess, Domain Rules - Process each domain file
    // =========================================================================
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

        // Log what we're processing - filename for single file, count for multiple
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

        let domain = match pipeline.get_domain(&domain_key).cloned() {
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
            // Log each file when processing multiple files for a domain
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

            // Use pipeline stage function to process each file
            let result = process_file(ProcessFileInput {
                path,
                domain: &domain,
                dataset_name: variant.as_str(),
                study_id: &study_id,
                study_metadata: &study_metadata,
                suppqual_domain: &suppqual_domain,
                suppqual_exclusions: &suppqual_exclusions,
                seq_tracker: domain_tracker,
                pipeline: &pipeline,
            });

            match result {
                Ok(processed) => {
                    *input_counts.entry(domain_key.clone()).or_insert(0) += processed.input_count;

                    if let Some(suppqual) = processed.suppqual {
                        suppqual_frames.push(suppqual);
                    }

                    domain_mappings.push(processed.mapping);

                    // Extract reference starts from DM
                    if domain_key == "DM" {
                        let dm_starts = extract_reference_starts(&processed.frame.data);
                        pipeline.add_reference_starts(dm_starts);
                    }

                    if multi_source {
                        if let Some(existing) = combined.as_mut() {
                            existing
                                .vstack_mut(&processed.frame.data)
                                .with_context(|| format!("merge {domain_code} frames"))?;
                        } else {
                            combined = Some(processed.frame.data);
                        }
                    } else if let Err(error) = insert_frame(&mut processed_frames, processed.frame)
                    {
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
                    domain_code: key.clone(),
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
        build_relationship_frames(&relationship_sources, &pipeline.standards, &study_id)
            .context("build relationship domains")?;
    for frame in relationship_frames {
        if let Err(error) = insert_frame(&mut frames, frame) {
            errors.push(format!("relationship merge: {error}"));
        }
    }

    // Deduplicate and sort frames
    let mut frame_list: Vec<DomainFrame> = frames.into_values().collect();
    dedupe_frames_by_identifiers(&mut frame_list, &pipeline.standards_map, &suppqual_domain)?;
    frame_list.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));
    let total_records: usize = frame_list.iter().map(|frame| frame.record_count()).sum();
    info!(
        study_id = %study_id,
        domain_count = frame_list.len(),
        record_count = total_records,
        duration_ms = process_start.elapsed().as_millis(),
        "domain processing complete"
    );

    // Build report domains for output
    let report_domains = build_report_domains(&pipeline.standards, &frame_list)?;
    let report_domain_map = sdtm_core::domain_map_by_code(&report_domains);

    // =========================================================================
    // Stage 5: Validate - Run P21 validation rules
    // =========================================================================
    let validation_result = validate(&frame_list, &pipeline, &output_dir, &study_id, args.dry_run)?;
    let mut report_map = validation_result.reports;
    errors.extend(validation_result.errors);
    let conformance_report = validation_result.report_path;

    // =========================================================================
    // Stage 5.5: Gate outputs - Block strict outputs if validation fails
    // =========================================================================
    let fail_on_conformance_errors = !args.no_fail_on_conformance_errors;
    let conformance_reports: Vec<_> = report_map.values().cloned().collect();
    let gating = gate_strict_outputs(
        &output_formats,
        fail_on_conformance_errors,
        &conformance_reports,
    );

    if gating.block_strict_outputs {
        warn!(
            study_id = %study_id,
            blocked_domains = ?gating.blocking_domains,
            "strict output blocked due to conformance errors"
        );
        errors.push(format!(
            "Output blocked: conformance errors in domains: {}. Use --no-fail-on-conformance-errors to override.",
            gating.blocking_domains.join(", ")
        ));
    }

    // =========================================================================
    // Stage 6: Output - Write XPT, Dataset-XML, Define-XML, SAS
    // =========================================================================
    // Filter formats based on gating decision
    let gated_formats: Vec<OutputFormat> = if gating.block_strict_outputs {
        // Block XPT output when conformance errors exist
        output_formats
            .iter()
            .filter(|f| !matches!(f, OutputFormat::Xpt))
            .cloned()
            .collect()
    } else {
        output_formats.clone()
    };

    let output_result = output(OutputConfig {
        output_dir: &output_dir,
        study_id: &study_id,
        report_domains: &report_domains,
        frames: &frame_list,
        mapping_configs: &mapping_configs,
        formats: &gated_formats,
        dry_run: args.dry_run,
        skip_define_xml: args.no_define_xml,
        skip_sas: args.no_sas,
    })?;
    let mut output_paths = output_result.paths;
    let define_xml = output_result.define_xml;
    errors.extend(output_result.errors);

    // =========================================================================
    // Post-processing: Verify XPT counts and build summaries
    // =========================================================================
    let mut data_checks = Vec::new();
    let want_xpt_and_not_blocked = want_xpt && !gating.block_strict_outputs;
    if want_xpt_and_not_blocked && !args.dry_run {
        let (xpt_counts, xpt_errors) = verify_xpt_counts(&output_paths, &input_counts);
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
        let code = frame.domain_code.to_uppercase();
        let domain = report_domain_map.get(&code);
        let description = domain
            .and_then(|d| d.description.clone().or(d.label.clone()))
            .unwrap_or_default();
        let outputs = output_paths.remove(&code).unwrap_or_default();
        let conformance = report_map.remove(&code);
        summaries.push(DomainSummary {
            domain_code: code,
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
                .map(|report| report.has_errors())
                .unwrap_or(false)
        });

    Ok(StudyResult {
        study_id,
        output_dir,
        domains: summaries,
        data_checks,
        errors,
        conformance_report,
        define_xml,
        has_errors,
    })
}

fn format_outputs(format: OutputFormatArg) -> Vec<OutputFormat> {
    match format {
        OutputFormatArg::Xpt => vec![OutputFormat::Xpt],
        OutputFormatArg::Xml => vec![OutputFormat::Xml],
        OutputFormatArg::Both => vec![OutputFormat::Xpt, OutputFormat::Xml],
    }
}

fn derive_study_id(study_folder: &Path) -> String {
    let name = study_folder
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("STUDY");
    let parts: Vec<&str> = name.split('_').collect();
    if parts.len() >= 2 {
        format!("{}_{}", parts[0], parts[1])
    } else {
        name.to_string()
    }
}
