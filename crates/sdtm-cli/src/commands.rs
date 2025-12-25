use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use comfy_table::Table;
use polars::prelude::DataFrame;

use sdtm_core::{
    DomainFrame, ProcessingContext, build_domain_frame, build_domain_frame_with_mapping,
    build_lb_wide_frame, build_relationship_frames, build_report_domains, build_suppqual,
    build_vs_wide_frame, dedupe_frames_by_identifiers, fill_missing_test_fields, insert_frame,
    is_supporting_domain, process_domain_with_context_and_tracker,
};
use sdtm_ingest::{
    build_column_hints, discover_domain_files, list_csv_files, read_csv_schema, read_csv_table,
};
use sdtm_map::{MappingEngine, merge_mappings};
use sdtm_model::{ConformanceReport, MappingConfig, OutputFormat};
use sdtm_report::{
    DefineXmlOptions, SasProgramOptions, write_dataset_xml_outputs, write_define_xml,
    write_sas_outputs, write_xpt_outputs,
};
use sdtm_standards::{
    load_default_ct_registry, load_default_p21_rules, load_default_sdtm_ig_domains,
};
use sdtm_validate::{ValidationContext, validate_domains, write_conformance_report_json};
use sdtm_xpt::{read_xpt, XptWriterOptions};

use crate::cli::{OutputFormatArg, StudyArgs};
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
    let output_dir = args
        .output_dir
        .clone()
        .unwrap_or_else(|| study_folder.join("output"));
    let output_formats = format_outputs(args.format);
    let want_xpt = output_formats
        .iter()
        .any(|f| matches!(f, OutputFormat::Xpt));
    let want_xml = output_formats
        .iter()
        .any(|f| matches!(f, OutputFormat::Xml));

    let standards = load_default_sdtm_ig_domains().context("load standards")?;
    let ct_registry = load_default_ct_registry().context("load ct registry")?;
    let p21_rules = load_default_p21_rules().context("load p21 rules")?;
    let domain_codes: Vec<String> = standards.iter().map(|d| d.code.clone()).collect();
    let mut standards_map = BTreeMap::new();
    for domain in &standards {
        standards_map.insert(domain.code.to_uppercase(), domain);
    }

    let csv_files = list_csv_files(study_folder).context("list csv files")?;
    let discovered = discover_domain_files(&csv_files, &domain_codes);

    let mut processed_frames: BTreeMap<String, DomainFrame> = BTreeMap::new();
    let mut suppqual_frames: Vec<DomainFrame> = Vec::new();
    let mut mapping_configs: BTreeMap<String, Vec<MappingConfig>> = BTreeMap::new();
    let mut errors = Vec::new();
    let mut input_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut seq_trackers: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();

    let mut standard_variables = BTreeSet::new();
    for domain in &standards {
        for variable in &domain.variables {
            standard_variables.insert(variable.name.to_uppercase());
        }
    }
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
    let global_suppqual_exclusions = if total_files >= 3 {
        let threshold = ((total_files as f64) * 0.6).ceil() as usize;
        column_counts
            .into_iter()
            .filter(|(name, count)| *count >= threshold && !standard_variables.contains(name))
            .map(|(name, _)| name)
            .collect::<BTreeSet<String>>()
    } else {
        BTreeSet::new()
    };

    let ctx = ProcessingContext::new(&study_id).with_ct_registry(&ct_registry);
    let suppqual_domain = standards_map
        .get("SUPPQUAL")
        .ok_or_else(|| anyhow!("missing SUPPQUAL metadata"))?;

    for (domain_code, files) in &discovered {
        let multi_source = files.len() > 1;
        let domain_key = domain_code.to_uppercase();
        let domain = match standards_map.get(&domain_key) {
            Some(domain) => domain,
            None => {
                errors.push(format!("missing standards metadata for {domain_code}"));
                continue;
            }
        };
        let domain_tracker = seq_trackers.entry(domain_key.clone()).or_default();
        let mut combined: Option<DataFrame> = None;
        let mut domain_mappings = Vec::new();
        for (path, _variant) in files {
            let table = match read_csv_table(&path) {
                Ok(table) => table,
                Err(error) => {
                    errors.push(format!("{}: {error}", path.display()));
                    continue;
                }
            };
            *input_counts.entry(domain_key.clone()).or_insert(0) += table.rows.len();
            let hints = build_column_hints(&table);
            let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
            let mapping_result = engine.suggest(&table.headers);
            let mapping_config = engine.to_config(&study_id, mapping_result);

            let (mapping_config, mapped, used) = match domain_code.as_str() {
                "LB" => match build_lb_wide_frame(&table, domain, &study_id) {
                    Ok(Some((config, frame, used))) => (config, frame, used),
                    Ok(None) => {
                        let mapped = match build_domain_frame_with_mapping(
                            &table,
                            domain,
                            Some(&mapping_config),
                        ) {
                            Ok(frame) => frame,
                            Err(error) => {
                                errors.push(format!("{}: {error}", path.display()));
                                continue;
                            }
                        };
                        let used = mapping_config
                            .mappings
                            .iter()
                            .map(|mapping| mapping.source_column.clone())
                            .collect::<BTreeSet<String>>();
                        (mapping_config, mapped, used)
                    }
                    Err(error) => {
                        errors.push(format!("{}: {error}", path.display()));
                        continue;
                    }
                },
                "VS" => match build_vs_wide_frame(&table, domain, &study_id) {
                    Ok(Some((config, frame, used))) => (config, frame, used),
                    Ok(None) => {
                        let mapped = match build_domain_frame_with_mapping(
                            &table,
                            domain,
                            Some(&mapping_config),
                        ) {
                            Ok(frame) => frame,
                            Err(error) => {
                                errors.push(format!("{}: {error}", path.display()));
                                continue;
                            }
                        };
                        let used = mapping_config
                            .mappings
                            .iter()
                            .map(|mapping| mapping.source_column.clone())
                            .collect::<BTreeSet<String>>();
                        (mapping_config, mapped, used)
                    }
                    Err(error) => {
                        errors.push(format!("{}: {error}", path.display()));
                        continue;
                    }
                },
                _ => {
                    let mapped = match build_domain_frame_with_mapping(
                        &table,
                        domain,
                        Some(&mapping_config),
                    ) {
                        Ok(frame) => frame,
                        Err(error) => {
                            errors.push(format!("{}: {error}", path.display()));
                            continue;
                        }
                    };
                    let used = mapping_config
                        .mappings
                        .iter()
                        .map(|mapping| mapping.source_column.clone())
                        .collect::<BTreeSet<String>>();
                    (mapping_config, mapped, used)
                }
            };

            let source = match build_domain_frame(&table, domain_code) {
                Ok(frame) => frame,
                Err(error) => {
                    errors.push(format!("{}: {error}", path.display()));
                    continue;
                }
            };

            let mut mapped = mapped;
            if let Err(error) =
                fill_missing_test_fields(domain, &mapping_config, &table, &mut mapped.data, &ctx)
            {
                errors.push(format!("{}: {error}", path.display()));
            }
            if let Err(error) = process_domain_with_context_and_tracker(
                domain,
                &mut mapped.data,
                &ctx,
                Some(domain_tracker),
            ) {
                errors.push(format!("{}: {error}", path.display()));
            }
            match build_suppqual(
                domain,
                suppqual_domain,
                &source.data,
                Some(&mapped.data),
                &used,
                &study_id,
                Some(&global_suppqual_exclusions),
            ) {
                Ok(Some(result)) => {
                    suppqual_frames.push(DomainFrame {
                        domain_code: result.domain_code,
                        data: result.data,
                    });
                }
                Ok(None) => {}
                Err(error) => {
                    errors.push(format!("SUPPQUAL {}: {error}", domain_code));
                }
            }

            domain_mappings.push(mapping_config);

            if multi_source {
                if let Some(existing) = combined.as_mut() {
                    existing
                        .vstack_mut(&mapped.data)
                        .with_context(|| format!("merge {domain_code} frames"))?;
                } else {
                    combined = Some(mapped.data);
                }
            } else {
                if let Err(error) = insert_frame(
                    &mut processed_frames,
                    DomainFrame {
                        domain_code: domain_code.to_uppercase(),
                        data: mapped.data,
                    },
                ) {
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

    let mut frames = processed_frames;
    for frame in suppqual_frames {
        if let Err(error) = insert_frame(&mut frames, frame) {
            errors.push(format!("SUPPQUAL merge: {error}"));
        }
    }
    let relationship_sources: Vec<DomainFrame> = frames
        .values()
        .filter(|frame| !is_supporting_domain(&frame.domain_code))
        .cloned()
        .collect();
    let relationship_frames =
        build_relationship_frames(&relationship_sources, &standards, &study_id)
            .context("build relationship domains")?;
    for frame in relationship_frames {
        if let Err(error) = insert_frame(&mut frames, frame) {
            errors.push(format!("relationship merge: {error}"));
        }
    }

    let mut frame_list: Vec<DomainFrame> = frames.into_values().collect();
    dedupe_frames_by_identifiers(&mut frame_list, &standards_map, suppqual_domain)?;
    frame_list.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));

    let report_domains = build_report_domains(&standards, &frame_list)?;
    let report_domain_map = sdtm_core::domain_map_by_code(&report_domains);

    let validation_ctx = ValidationContext::new()
        .with_ct_registry(&ct_registry)
        .with_p21_rules(&p21_rules);
    let frame_refs: Vec<(&str, &DataFrame)> = frame_list
        .iter()
        .map(|frame| (frame.domain_code.as_str(), &frame.data))
        .collect();
    let reports = validate_domains(&standards, &frame_refs, &validation_ctx);
    let mut report_map = BTreeMap::new();
    for report in reports {
        report_map.insert(report.domain_code.to_uppercase(), report);
    }

    let conformance_report = if args.dry_run {
        None
    } else {
        let report_list: Vec<ConformanceReport> = report_map.values().cloned().collect();
        match write_conformance_report_json(&output_dir, &study_id, &report_list) {
            Ok(path) => Some(path),
            Err(error) => {
                errors.push(format!("conformance report: {error}"));
                None
            }
        }
    };

    let mut output_paths: BTreeMap<String, sdtm_model::OutputPaths> = BTreeMap::new();
    let define_xml = if args.dry_run {
        None
    } else {
        let options = DefineXmlOptions::new("3.4", "Submission");
        let path = output_dir.join("define.xml");
        if let Err(error) =
            write_define_xml(&path, &study_id, &report_domains, &frame_list, &options)
        {
            errors.push(format!("define-xml: {error}"));
            None
        } else {
            Some(path)
        }
    };

    if !args.dry_run {
        if want_xpt {
            let options = XptWriterOptions::default();
            match write_xpt_outputs(&output_dir, &report_domains, &frame_list, &options) {
                Ok(paths) => {
                    for path in paths {
                        let key = path
                            .file_stem()
                            .and_then(|v| v.to_str())
                            .unwrap_or("")
                            .to_uppercase();
                        output_paths.entry(key).or_default().xpt.get_or_insert(path);
                    }
                }
                Err(error) => errors.push(format!("xpt: {error}")),
            }
        }
        if want_xml {
            match write_dataset_xml_outputs(
                &output_dir,
                &report_domains,
                &frame_list,
                &study_id,
                "3.4",
            ) {
                Ok(paths) => {
                    for path in paths {
                        let key = path
                            .file_stem()
                            .and_then(|v| v.to_str())
                            .unwrap_or("")
                            .to_uppercase();
                        output_paths
                            .entry(key)
                            .or_default()
                            .dataset_xml
                            .get_or_insert(path);
                    }
                }
                Err(error) => errors.push(format!("dataset-xml: {error}")),
            }
        }

        let merged_mappings = merge_mappings(&mapping_configs, &study_id);
        if !merged_mappings.is_empty() {
            let mut sas_frames: Vec<DomainFrame> = frame_list
                .iter()
                .filter(|frame| merged_mappings.contains_key(&frame.domain_code.to_uppercase()))
                .cloned()
                .collect();
            sas_frames.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));
            let options = SasProgramOptions::default();
            match write_sas_outputs(
                &output_dir,
                &report_domains,
                &sas_frames,
                &merged_mappings,
                &options,
            ) {
                Ok(paths) => {
                    for path in paths {
                        let key = path
                            .file_stem()
                            .and_then(|v| v.to_str())
                            .unwrap_or("")
                            .to_uppercase();
                        output_paths.entry(key).or_default().sas.get_or_insert(path);
                    }
                }
                Err(error) => errors.push(format!("sas: {error}")),
            }
        }
    }

    let mut xpt_counts: BTreeMap<String, usize> = BTreeMap::new();
    if want_xpt && !args.dry_run {
        for (code, paths) in &output_paths {
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
    }

    let mut data_checks = Vec::new();
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
