use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use comfy_table::Table;
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};

use sdtm_cli::logging::init_logging;
use sdtm_core::{
    DomainFrame, ProcessingContext, build_domain_frame, build_domain_frame_with_mapping,
    build_relationship_frames, build_suppqual, infer_seq_column, process_domain_with_context,
    standard_columns,
};
use sdtm_ingest::{
    build_column_hints, discover_domain_files, list_csv_files, read_csv_schema, read_csv_table,
};
use sdtm_map::MappingEngine;
use sdtm_model::{ConformanceReport, Domain, MappingConfig, MappingSuggestion, OutputFormat};
use sdtm_report::{
    DefineXmlOptions, SasProgramOptions, write_dataset_xml_outputs, write_define_xml,
    write_sas_outputs, write_xpt_outputs,
};
use sdtm_standards::{
    load_default_ct_registry, load_default_p21_rules, load_default_sdtm_ig_domains,
};
use sdtm_validate::{ValidationContext, validate_domains, write_conformance_report_json};
use sdtm_xpt::XptWriterOptions;

mod ct_utils;
mod data_utils;
mod dedupe;
mod summary;
mod wide;

use crate::ct_utils::{
    completion_column, ct_column_match, is_yes_no_token, resolve_ct_for_variable,
    resolve_ct_value_from_hint,
};
use crate::data_utils::{
    any_to_string, column_hint_for_domain, fill_string_column, mapping_source_for_target,
    sanitize_test_code, table_column_values, table_label,
};
use crate::dedupe::dedupe_frames_by_identifiers;
use crate::summary::{apply_table_style, print_summary};
use crate::wide::{build_lb_wide_frame, build_vs_wide_frame};

#[derive(Parser)]
#[command(name = "cdisc-transpiler", version, about = "CDISC Transpiler CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Command {
    Study(StudyArgs),
    Domains,
}

#[derive(Parser)]
struct StudyArgs {
    #[arg(value_name = "STUDY_FOLDER")]
    study_folder: PathBuf,

    #[arg(long = "output-dir")]
    output_dir: Option<PathBuf>,

    #[arg(long = "format", value_enum, default_value = "both")]
    format: OutputFormatArg,

    #[arg(long = "dry-run", default_value_t = false)]
    dry_run: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormatArg {
    Xpt,
    Xml,
    Both,
}

fn main() {
    let cli = Cli::parse();
    init_logging(cli.verbose);
    let exit_code = match cli.command {
        Command::Study(args) => match run_study(&args) {
            Ok(result) => {
                print_summary(&result);
                if result.has_errors { 1 } else { 0 }
            }
            Err(error) => {
                eprintln!("error: {error}");
                1
            }
        },
        Command::Domains => match run_domains() {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("error: {error}");
                1
            }
        },
    };
    std::process::exit(exit_code);
}

struct StudyResult {
    study_id: String,
    output_dir: PathBuf,
    domains: Vec<DomainSummary>,
    errors: Vec<String>,
    conformance_report: Option<PathBuf>,
    define_xml: Option<PathBuf>,
    has_errors: bool,
}

struct DomainSummary {
    domain_code: String,
    description: String,
    records: usize,
    outputs: sdtm_model::OutputPaths,
    conformance: Option<ConformanceReport>,
}

fn run_domains() -> Result<()> {
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

fn run_study(args: &StudyArgs) -> Result<StudyResult> {
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
    let mut seq_trackers: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
    let mut errors = Vec::new();

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
        let domain = match standards_map.get(&domain_code.to_uppercase()) {
            Some(domain) => domain,
            None => {
                errors.push(format!("missing standards metadata for {domain_code}"));
                continue;
            }
        };
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
            let (mapping_config, mut mapped, used) = if domain.code.eq_ignore_ascii_case("LB") {
                match build_lb_wide_frame(&table, domain, &study_id) {
                    Ok(Some(result)) => result,
                    Ok(None) => {
                        let hints = build_column_hints(&table);
                        let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
                        let result = engine.suggest(&table.headers);
                        let mapping_config = engine.to_config(&study_id, result);
                        let used: BTreeSet<String> = mapping_config
                            .mappings
                            .iter()
                            .map(|mapping| mapping.source_column.clone())
                            .collect();
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
                        (mapping_config, mapped, used)
                    }
                    Err(error) => {
                        errors.push(format!("{}: {error}", path.display()));
                        continue;
                    }
                }
            } else if domain.code.eq_ignore_ascii_case("VS") {
                match build_vs_wide_frame(&table, domain, &study_id) {
                    Ok(Some(result)) => result,
                    Ok(None) => {
                        let hints = build_column_hints(&table);
                        let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
                        let result = engine.suggest(&table.headers);
                        let mapping_config = engine.to_config(&study_id, result);
                        let used: BTreeSet<String> = mapping_config
                            .mappings
                            .iter()
                            .map(|mapping| mapping.source_column.clone())
                            .collect();
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
                        (mapping_config, mapped, used)
                    }
                    Err(error) => {
                        errors.push(format!("{}: {error}", path.display()));
                        continue;
                    }
                }
            } else {
                let hints = build_column_hints(&table);
                let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
                let result = engine.suggest(&table.headers);
                let mapping_config = engine.to_config(&study_id, result);
                let used: BTreeSet<String> = mapping_config
                    .mappings
                    .iter()
                    .map(|mapping| mapping.source_column.clone())
                    .collect();
                let mapped =
                    match build_domain_frame_with_mapping(&table, domain, Some(&mapping_config)) {
                        Ok(frame) => frame,
                        Err(error) => {
                            errors.push(format!("{}: {error}", path.display()));
                            continue;
                        }
                    };
                (mapping_config, mapped, used)
            };
            if let Err(error) =
                fill_missing_test_fields(domain, &mapping_config, &table, &mut mapped.data, &ctx)
            {
                errors.push(format!("{}: {error}", path.display()));
                continue;
            }
            domain_mappings.push(mapping_config.clone());
            if let Err(error) = process_domain_with_context(domain, &mut mapped.data, &ctx) {
                errors.push(format!("{}: {error}", path.display()));
                continue;
            }
            if multi_source {
                let tracker = seq_trackers.entry(domain.code.to_uppercase()).or_default();
                if let Err(error) = apply_sequence_offsets(domain, &mut mapped.data, tracker) {
                    errors.push(format!("{}: {error}", path.display()));
                    continue;
                }
            }

            if let Some(existing) = &mut combined {
                if let Err(error) = existing.vstack_mut(&mapped.data) {
                    errors.push(format!("{}: {error}", path.display()));
                }
            } else {
                combined = Some(mapped.data.clone());
            }

            let source = match build_domain_frame(&table, domain_code) {
                Ok(frame) => frame,
                Err(error) => {
                    errors.push(format!("{}: {error}", path.display()));
                    continue;
                }
            };
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
    let report_domain_map = domain_map_by_code(&report_domains);

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

fn insert_frame(map: &mut BTreeMap<String, DomainFrame>, frame: DomainFrame) -> Result<()> {
    let key = frame.domain_code.to_uppercase();
    if let Some(existing) = map.get_mut(&key) {
        existing
            .data
            .vstack_mut(&frame.data)
            .with_context(|| format!("merge {key} frames"))?;
    } else {
        map.insert(
            key.clone(),
            DomainFrame {
                domain_code: key.clone(),
                data: frame.data,
            },
        );
    }
    Ok(())
}

fn fill_missing_test_fields(
    domain: &Domain,
    mapping: &MappingConfig,
    table: &sdtm_ingest::CsvTable,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    let code = domain.code.to_uppercase();
    if code == "QS" {
        let orres_source = mapping_source_for_target(mapping, "QSORRES")
            .or_else(|| mapping_source_for_target(mapping, "QSSTRESC"));
        let label_hint = orres_source
            .as_deref()
            .and_then(|col| column_hint_for_domain(table, domain, col))
            .or_else(|| column_hint_for_domain(table, domain, "QSPGARS"))
            .or_else(|| column_hint_for_domain(table, domain, "QSPGARSCD"));
        if let Some((label, allow_raw)) = label_hint {
            let test_code = sanitize_test_code(&label);
            fill_string_column(df, "QSTEST", &label)?;
            fill_string_column(df, "QSTESTCD", &test_code)?;
            if let Some(qscat) = resolve_ct_for_variable(ctx, domain, "QSCAT", &label, allow_raw) {
                fill_string_column(df, "QSCAT", &qscat)?;
            }
        }
    } else if code == "PE" {
        let orres_source = mapping_source_for_target(mapping, "PEORRES")
            .or_else(|| mapping_source_for_target(mapping, "PEORRESSP"));
        let label_hint = orres_source
            .as_deref()
            .and_then(|col| column_hint_for_domain(table, domain, col))
            .or_else(|| column_hint_for_domain(table, domain, "PEORRES"))
            .or_else(|| column_hint_for_domain(table, domain, "PEORRESSP"));
        if let Some((label, _allow_raw)) = label_hint {
            let test_code = sanitize_test_code(&label);
            fill_string_column(df, "PETEST", &label)?;
            fill_string_column(df, "PETESTCD", &test_code)?;
        }
    } else if code == "DS" {
        let mut decod_vals = if let Ok(series) = df.column("DSDECOD") {
            (0..df.height())
                .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                .collect::<Vec<_>>()
        } else {
            vec![String::new(); df.height()]
        };
        let mut term_vals = if let Ok(series) = df.column("DSTERM") {
            (0..df.height())
                .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                .collect::<Vec<_>>()
        } else {
            vec![String::new(); df.height()]
        };
        if let Some(ct) = ctx.resolve_ct(domain, "DSDECOD") {
            if let Some((_header, mapped, raw)) = ct_column_match(table, domain, ct) {
                for idx in 0..df.height().min(mapped.len()).min(raw.len()) {
                    if decod_vals[idx].trim().is_empty() {
                        if let Some(ct_value) = &mapped[idx] {
                            decod_vals[idx] = ct_value.clone();
                        }
                    }
                    if term_vals[idx].trim().is_empty() && !raw[idx].trim().is_empty() {
                        term_vals[idx] = raw[idx].trim().to_string();
                    }
                }
            }
        }
        if let Some((values, label)) = completion_column(table, domain) {
            for idx in 0..df.height().min(values.len()) {
                if decod_vals[idx].trim().is_empty() && !values[idx].trim().is_empty() {
                    decod_vals[idx] = values[idx].trim().to_string();
                }
                if term_vals[idx].trim().is_empty() && !label.trim().is_empty() {
                    term_vals[idx] = label.clone();
                }
            }
        }
        df.with_column(Series::new("DSDECOD".into(), decod_vals))?;
        df.with_column(Series::new("DSTERM".into(), term_vals))?;
    } else if code == "EX" {
        let mut extrt_vals = if let Ok(series) = df.column("EXTRT") {
            (0..df.height())
                .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                .collect::<Vec<_>>()
        } else {
            vec![String::new(); df.height()]
        };
        let mut standard_vars = BTreeSet::new();
        for variable in &domain.variables {
            standard_vars.insert(variable.name.to_uppercase());
        }
        let mut candidate_headers: Vec<String> = Vec::new();
        if let Some(preferred) = mapping_source_for_target(mapping, "EXTRT") {
            candidate_headers.push(preferred);
        }
        let keywords = ["TREAT", "DRUG", "THERAP", "INTERVENT"];
        for header in &table.headers {
            if standard_vars.contains(&header.to_uppercase()) {
                continue;
            }
            let label = table_label(table, header).unwrap_or_default();
            let mut hay = header.to_uppercase();
            if !label.is_empty() {
                hay.push(' ');
                hay.push_str(&label.to_uppercase());
            }
            if keywords.iter().any(|kw| hay.contains(kw)) {
                candidate_headers.push(header.clone());
            }
        }
        for fallback in ["EventName", "ActivityName"] {
            if table
                .headers
                .iter()
                .any(|header| header.eq_ignore_ascii_case(fallback))
            {
                candidate_headers.push(fallback.to_string());
            }
        }
        candidate_headers.sort();
        candidate_headers.dedup();
        let mut candidates: Vec<Vec<String>> = Vec::new();
        for header in candidate_headers {
            if let Some(values) = table_column_values(table, &header) {
                if values.iter().any(|value| !value.trim().is_empty()) {
                    candidates.push(values);
                }
            }
        }
        if !candidates.is_empty() {
            for idx in 0..df.height() {
                if !extrt_vals[idx].trim().is_empty() {
                    continue;
                }
                for values in &candidates {
                    let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
                    if !value.is_empty() {
                        extrt_vals[idx] = value.to_string();
                        break;
                    }
                }
            }
            df.with_column(Series::new("EXTRT".into(), extrt_vals))?;
        }
    } else if code == "DA" {
        let ctdatest = ctx.resolve_ct(domain, "DATEST");
        let ctdatestcd = ctx.resolve_ct(domain, "DATESTCD");
        let ct_units = ctx.resolve_ct(domain, "DAORRESU");
        let datest_extensible = ctdatest.map(|ct| ct.extensible).unwrap_or(false);
        let datestcd_extensible = ctdatestcd.map(|ct| ct.extensible).unwrap_or(false);
        let mut candidates: Vec<(Option<String>, Option<String>, Option<String>, Vec<String>)> =
            Vec::new();
        let mut candidate_headers: Vec<String> = Vec::new();
        if let Some(preferred) = mapping_source_for_target(mapping, "DAORRES") {
            candidate_headers.push(preferred);
        } else {
            for header in &table.headers {
                if header.to_uppercase().ends_with("_DAORRES") {
                    candidate_headers.push(header.clone());
                }
            }
        }
        let mut standard_vars = BTreeSet::new();
        for variable in &domain.variables {
            standard_vars.insert(variable.name.to_uppercase());
        }
        for header in &table.headers {
            let upper = header.to_uppercase();
            if !upper.starts_with("DA") {
                continue;
            }
            if upper.ends_with("CD") {
                continue;
            }
            if standard_vars.contains(&upper) {
                continue;
            }
            candidate_headers.push(header.clone());
        }
        candidate_headers.sort();
        candidate_headers.dedup();
        for header in candidate_headers {
            let upper = header.to_uppercase();
            let prefix = upper.strip_suffix("_DAORRES").unwrap_or(&upper);
            if let Some(values) = table_column_values(table, &header) {
                let label = table_label(table, &header);
                let hint = label.clone().unwrap_or_else(|| prefix.to_string());
                let mut test_code = ctdatestcd
                    .and_then(|ct| resolve_ct_value_from_hint(ct, prefix))
                    .or_else(|| {
                        label.as_deref().and_then(|text| {
                            ctdatestcd.and_then(|ct| resolve_ct_value_from_hint(ct, text))
                        })
                    })
                    .or_else(|| ctdatestcd.and_then(|ct| resolve_ct_value_from_hint(ct, &hint)));
                let mut test_name = ctdatest
                    .and_then(|ct| resolve_ct_value_from_hint(ct, prefix))
                    .or_else(|| {
                        label.as_deref().and_then(|text| {
                            ctdatest.and_then(|ct| resolve_ct_value_from_hint(ct, text))
                        })
                    })
                    .or_else(|| ctdatest.and_then(|ct| resolve_ct_value_from_hint(ct, &hint)));
                if test_name.is_none() {
                    if let (Some(ct), Some(code)) = (ctdatestcd, test_code.as_ref()) {
                        test_name = ct.preferred_terms.get(code).cloned();
                    }
                }
                if test_name.is_none() && datest_extensible {
                    test_name = label.clone().or_else(|| Some(prefix.to_string()));
                }
                if test_code.is_none() && datestcd_extensible {
                    let raw = label.clone().unwrap_or_else(|| prefix.to_string());
                    test_code = Some(sanitize_test_code(&raw));
                }
                let unit = ct_units
                    .and_then(|ct| resolve_ct_value_from_hint(ct, &hint))
                    .or_else(|| {
                        label.as_deref().and_then(|text| {
                            ct_units.and_then(|ct| resolve_ct_value_from_hint(ct, text))
                        })
                    })
                    .or_else(|| ct_units.and_then(|ct| resolve_ct_value_from_hint(ct, prefix)));
                candidates.push((test_name, test_code, unit, values));
            }
        }
        if !candidates.is_empty() {
            let mut daorres_vals = if let Ok(series) = df.column("DAORRES") {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut datest_vals = if let Ok(series) = df.column("DATEST") {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut datestcd_vals = if let Ok(series) = df.column("DATESTCD") {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut daorresu_vals = if let Ok(series) = df.column("DAORRESU") {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut dastresu_vals = if let Ok(series) = df.column("DASTRESU") {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            for idx in 0..df.height() {
                let needs_orres = daorres_vals[idx].trim().is_empty();
                let needs_test = datest_vals[idx].trim().is_empty();
                let needs_testcd = datestcd_vals[idx].trim().is_empty();
                let needs_orresu = daorresu_vals[idx].trim().is_empty();
                let needs_stresu = dastresu_vals[idx].trim().is_empty();
                if !needs_orres && !needs_test && !needs_testcd && !needs_orresu && !needs_stresu {
                    continue;
                }
                for (test_name, test_code, unit, values) in &candidates {
                    let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
                    if value.is_empty() {
                        continue;
                    }
                    if needs_test && test_name.is_none() {
                        continue;
                    }
                    if needs_testcd && test_code.is_none() {
                        continue;
                    }
                    if needs_orres {
                        daorres_vals[idx] = value.to_string();
                    }
                    if needs_test {
                        if let Some(name) = test_name {
                            datest_vals[idx] = name.clone();
                        }
                    }
                    if needs_testcd {
                        if let Some(code) = test_code {
                            datestcd_vals[idx] = code.clone();
                        }
                    }
                    if needs_orresu {
                        if let Some(unit) = unit {
                            daorresu_vals[idx] = unit.clone();
                        }
                    }
                    if needs_stresu {
                        if let Some(unit) = unit {
                            dastresu_vals[idx] = unit.clone();
                        }
                    }
                    break;
                }
            }
            df.with_column(Series::new("DAORRES".into(), daorres_vals))?;
            df.with_column(Series::new("DATEST".into(), datest_vals))?;
            df.with_column(Series::new("DATESTCD".into(), datestcd_vals))?;
            df.with_column(Series::new("DAORRESU".into(), daorresu_vals))?;
            df.with_column(Series::new("DASTRESU".into(), dastresu_vals))?;
        }
    } else if code == "IE" {
        let mut candidates: Vec<(String, Vec<String>, String)> = Vec::new();
        let ct_cat = ctx.resolve_ct(domain, "IECAT");
        for header in &table.headers {
            let upper = header.to_uppercase();
            if !upper.starts_with("IE") {
                continue;
            }
            let label = table_label(table, header).unwrap_or_else(|| header.clone());
            let category = ct_cat.and_then(|ct| resolve_ct_value_from_hint(ct, &label));
            if let Some(category) = category {
                if let Some(values) = table_column_values(table, header) {
                    candidates.push((label, values, category));
                }
            }
        }
        if !candidates.is_empty() {
            let mut ietest_vals = if let Ok(series) = df.column("IETEST") {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut ietestcd_vals = if let Ok(series) = df.column("IETESTCD") {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut iecat_vals = if let Ok(series) = df.column("IECAT") {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let orres_vals = if let Ok(series) = df.column("IEORRES") {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            for idx in 0..df.height() {
                let testcd_raw = ietestcd_vals[idx].trim();
                let orres_raw = orres_vals.get(idx).map(|val| val.trim()).unwrap_or("");
                let needs_test = ietest_vals[idx].trim().is_empty();
                let needs_testcd = testcd_raw.is_empty()
                    || is_yes_no_token(testcd_raw)
                    || (!orres_raw.is_empty() && testcd_raw.eq_ignore_ascii_case(orres_raw));
                let needs_cat = iecat_vals[idx].trim().is_empty();
                if !needs_test && !needs_cat && !needs_testcd {
                    continue;
                }
                for (label, values, category) in &candidates {
                    let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
                    if value.is_empty() {
                        continue;
                    }
                    if needs_test {
                        ietest_vals[idx] = label.clone();
                    }
                    if needs_testcd {
                        ietestcd_vals[idx] = sanitize_test_code(label);
                    }
                    if needs_cat {
                        iecat_vals[idx] = category.clone();
                    }
                    break;
                }
            }
            df.with_column(Series::new("IETEST".into(), ietest_vals))?;
            df.with_column(Series::new("IETESTCD".into(), ietestcd_vals))?;
            df.with_column(Series::new("IECAT".into(), iecat_vals))?;
        }
    }
    Ok(())
}

fn merge_mappings(
    configs: &BTreeMap<String, Vec<MappingConfig>>,
    study_id: &str,
) -> BTreeMap<String, MappingConfig> {
    let mut merged = BTreeMap::new();
    for (domain_code, entries) in configs {
        if entries.is_empty() {
            continue;
        }
        merged.insert(
            domain_code.to_uppercase(),
            merge_mapping_configs(domain_code, study_id, entries),
        );
    }
    merged
}

fn merge_mapping_configs(
    domain_code: &str,
    study_id: &str,
    configs: &[MappingConfig],
) -> MappingConfig {
    let mut best: BTreeMap<String, MappingSuggestion> = BTreeMap::new();
    let mut unmapped = BTreeSet::new();
    for config in configs {
        for suggestion in &config.mappings {
            let key = suggestion.target_variable.to_uppercase();
            match best.get(&key) {
                Some(existing) => {
                    if suggestion.confidence > existing.confidence
                        || (suggestion.confidence == existing.confidence
                            && suggestion.source_column < existing.source_column)
                    {
                        best.insert(key, suggestion.clone());
                    }
                }
                None => {
                    best.insert(key, suggestion.clone());
                }
            }
        }
        for column in &config.unmapped_columns {
            unmapped.insert(column.clone());
        }
    }
    MappingConfig {
        domain_code: domain_code.to_uppercase(),
        study_id: study_id.to_string(),
        mappings: best.into_values().collect(),
        unmapped_columns: unmapped.into_iter().collect(),
    }
}

fn domain_map_by_code(domains: &[Domain]) -> BTreeMap<String, &Domain> {
    let mut map = BTreeMap::new();
    for domain in domains {
        map.insert(domain.code.to_uppercase(), domain);
    }
    map
}

fn build_report_domains(standards: &[Domain], frames: &[DomainFrame]) -> Result<Vec<Domain>> {
    let mut domains = standards.to_vec();
    let mut known: BTreeSet<String> = standards
        .iter()
        .map(|domain| domain.code.to_uppercase())
        .collect();
    let suppqual = standards
        .iter()
        .find(|domain| domain.code.eq_ignore_ascii_case("SUPPQUAL"))
        .ok_or_else(|| anyhow!("missing SUPPQUAL metadata"))?;

    for frame in frames {
        let code = frame.domain_code.to_uppercase();
        if known.contains(&code) {
            continue;
        }
        if let Some(parent) = code.strip_prefix("SUPP") {
            if parent.is_empty() {
                continue;
            }
            let label = format!("Supplemental Qualifiers for {parent}");
            let mut domain = suppqual.clone();
            domain.code = code.clone();
            domain.dataset_name = Some(code.clone());
            domain.label = Some(label.clone());
            domain.description = Some(label);
            domains.push(domain);
            known.insert(code);
        }
    }
    domains.sort_by(|a, b| a.code.cmp(&b.code));
    Ok(domains)
}

fn is_supporting_domain(code: &str) -> bool {
    let upper = code.to_uppercase();
    upper.starts_with("SUPP") || matches!(upper.as_str(), "RELREC" | "RELSPEC" | "RELSUB")
}

fn apply_sequence_offsets(
    domain: &Domain,
    df: &mut DataFrame,
    tracker: &mut BTreeMap<String, i64>,
) -> Result<()> {
    let Some(seq_col) = infer_seq_column(domain) else {
        return Ok(());
    };
    let columns = standard_columns(domain);
    let Some(usubjid_col) = columns.usubjid else {
        return Ok(());
    };
    let usubjid_series = match df.column(&usubjid_col) {
        Ok(series) => series.clone(),
        Err(_) => return Ok(()),
    };
    let mut values: Vec<Option<i64>> = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let usubjid = any_to_string(usubjid_series.get(idx).unwrap_or(AnyValue::Null));
        let key = usubjid.trim();
        if key.is_empty() {
            values.push(None);
            continue;
        }
        let entry = tracker.entry(key.to_string()).or_insert(0);
        *entry += 1;
        values.push(Some(*entry));
    }
    let series = Series::new(seq_col.into(), values);
    df.with_column(series)?;
    Ok(())
}
