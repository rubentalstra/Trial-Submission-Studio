use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ColumnConstraint, ContentArrangement, Table, Width,
};
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
use sdtm_model::{
    ConformanceReport, ControlledTerminology, Domain, IssueSeverity, MappingConfig,
    MappingSuggestion, OutputFormat,
};
use sdtm_report::{
    DefineXmlOptions, SasProgramOptions, write_dataset_xml_outputs, write_define_xml,
    write_sas_outputs, write_xpt_outputs,
};
use sdtm_standards::{
    load_default_ct_registry, load_default_p21_rules, load_default_sdtm_ig_domains,
};
use sdtm_validate::{ValidationContext, validate_domains, write_conformance_report_json};
use sdtm_xpt::XptWriterOptions;

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
    apply_table_style(&mut table);
    table.set_header(vec!["Domain", "Description"]);
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

fn table_label(table: &sdtm_ingest::CsvTable, column: &str) -> Option<String> {
    let labels = table.labels.as_ref()?;
    let idx = table
        .headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(column))?;
    let label = labels.get(idx)?.trim();
    if label.is_empty() {
        None
    } else {
        Some(label.to_string())
    }
}

fn column_hint_for_domain(
    table: &sdtm_ingest::CsvTable,
    domain: &Domain,
    column: &str,
) -> Option<(String, bool)> {
    let idx = table
        .headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(column))?;
    if let Some(labels) = table.labels.as_ref() {
        if let Some(label) = labels.get(idx) {
            let trimmed = label.trim();
            if !trimmed.is_empty() {
                return Some((trimmed.to_string(), true));
            }
        }
    }
    let header = table.headers.get(idx)?.clone();
    let is_standard = domain
        .variables
        .iter()
        .any(|var| var.name.eq_ignore_ascii_case(&header));
    if is_standard {
        None
    } else {
        Some((header, false))
    }
}

fn table_column_values(table: &sdtm_ingest::CsvTable, column: &str) -> Option<Vec<String>> {
    let idx = table
        .headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(column))?;
    let mut values = Vec::with_capacity(table.rows.len());
    for row in &table.rows {
        values.push(row.get(idx).cloned().unwrap_or_default());
    }
    Some(values)
}

fn mapping_source_for_target(mapping: &MappingConfig, target: &str) -> Option<String> {
    mapping
        .mappings
        .iter()
        .find(|entry| entry.target_variable.eq_ignore_ascii_case(target))
        .map(|entry| entry.source_column.clone())
}

fn fill_string_column(df: &mut DataFrame, name: &str, fill: &str) -> Result<()> {
    if fill.is_empty() {
        return Ok(());
    }
    let mut values = if let Ok(series) = df.column(name) {
        (0..df.height())
            .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
            .collect::<Vec<_>>()
    } else {
        vec![String::new(); df.height()]
    };
    for value in &mut values {
        if value.trim().is_empty() {
            *value = fill.to_string();
        }
    }
    let series = Series::new(name.into(), values);
    df.with_column(series)?;
    Ok(())
}

fn compact_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_uppercase())
        .collect()
}

fn resolve_ct_submission_value(ct: &ControlledTerminology, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let key = trimmed.to_uppercase();
    if let Some(mapped) = ct.synonyms.get(&key) {
        return Some(mapped.clone());
    }
    if ct.submission_values.iter().any(|val| val == trimmed) {
        return Some(trimmed.to_string());
    }
    for submission in &ct.submission_values {
        if compact_key(submission) == compact_key(trimmed) {
            return Some(submission.clone());
        }
    }
    for (submission, preferred) in &ct.preferred_terms {
        if compact_key(preferred) == compact_key(trimmed) {
            return Some(submission.clone());
        }
    }
    None
}

fn resolve_ct_value_from_hint(ct: &ControlledTerminology, hint: &str) -> Option<String> {
    if let Some(value) = resolve_ct_submission_value(ct, hint) {
        return Some(value);
    }
    let hint_compact = compact_key(hint);
    if hint_compact.len() < 3 {
        return None;
    }
    let mut matches: Vec<String> = Vec::new();
    for submission in &ct.submission_values {
        let compact = compact_key(submission);
        if compact.len() >= 3
            && (hint_compact.contains(&compact) || compact.contains(&hint_compact))
        {
            matches.push(submission.clone());
        }
    }
    for (submission, preferred) in &ct.preferred_terms {
        let compact = compact_key(preferred);
        if compact.len() >= 3
            && (hint_compact.contains(&compact) || compact.contains(&hint_compact))
        {
            matches.push(submission.clone());
        }
    }
    for (synonym, submission) in &ct.synonyms {
        let compact = compact_key(synonym);
        if compact.len() >= 3
            && (hint_compact.contains(&compact) || compact.contains(&hint_compact))
        {
            matches.push(submission.clone());
        }
    }
    matches.sort();
    matches.dedup();
    if matches.len() == 1 {
        Some(matches.remove(0))
    } else {
        let mut best_dist = usize::MAX;
        let mut best_val: Option<String> = None;
        let mut best_count = 0usize;
        for submission in &ct.submission_values {
            let dist = edit_distance(&hint_compact, &compact_key(submission));
            if dist < best_dist {
                best_dist = dist;
                best_val = Some(submission.clone());
                best_count = 1;
            } else if dist == best_dist {
                best_count += 1;
            }
        }
        if best_dist <= 1 && best_count == 1 {
            best_val
        } else {
            None
        }
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    let a_len = a.len();
    let b_len = b.len();
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }
    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0usize; b_len + 1];
    for (i, a_ch) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, b_ch) in b.chars().enumerate() {
            let cost = if a_ch == b_ch { 0 } else { 1 };
            let insert = curr[j] + 1;
            let delete = prev[j + 1] + 1;
            let replace = prev[j] + cost;
            curr[j + 1] = insert.min(delete).min(replace);
        }
        prev.clone_from_slice(&curr);
    }
    prev[b_len]
}

fn ct_column_match(
    table: &sdtm_ingest::CsvTable,
    domain: &Domain,
    ct: &ControlledTerminology,
) -> Option<(String, Vec<Option<String>>, Vec<String>)> {
    let mut standard_vars = BTreeSet::new();
    for variable in &domain.variables {
        standard_vars.insert(variable.name.to_uppercase());
    }
    let mut best: Option<(String, Vec<Option<String>>, Vec<String>, f64, usize)> = None;
    for header in &table.headers {
        if standard_vars.contains(&header.to_uppercase()) {
            continue;
        }
        let Some(values) = table_column_values(table, header) else {
            continue;
        };
        let mut mapped = Vec::with_capacity(values.len());
        let mut matches = 0usize;
        let mut non_empty = 0usize;
        for value in &values {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                mapped.push(None);
                continue;
            }
            non_empty += 1;
            if let Some(ct_value) = resolve_ct_submission_value(ct, trimmed) {
                matches += 1;
                mapped.push(Some(ct_value));
            } else {
                mapped.push(None);
            }
        }
        if non_empty == 0 || matches == 0 {
            continue;
        }
        let ratio = matches as f64 / non_empty as f64;
        if ratio < 0.6 {
            continue;
        }
        let replace = match &best {
            Some((_, _, _, best_ratio, best_matches)) => {
                ratio > *best_ratio || (ratio == *best_ratio && matches > *best_matches)
            }
            None => true,
        };
        if replace {
            best = Some((header.clone(), mapped, values, ratio, matches));
        }
    }
    best.map(|(header, mapped, values, _ratio, _matches)| (header, mapped, values))
}

fn resolve_ct_for_variable(
    ctx: &ProcessingContext,
    domain: &Domain,
    variable: &str,
    hint: &str,
    allow_raw: bool,
) -> Option<String> {
    let ct = ctx.resolve_ct(domain, variable)?;
    if let Some(value) = resolve_ct_value_from_hint(ct, hint) {
        return Some(value);
    }
    if allow_raw && ct.extensible {
        let trimmed = hint.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
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
            let test_code = sanitize_vstestcd(&label);
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
            let test_code = sanitize_vstestcd(&label);
            fill_string_column(df, "PETEST", &label)?;
            fill_string_column(df, "PETESTCD", &test_code)?;
        }
    } else if code == "DS" {
        if let Some(ct) = ctx.resolve_ct(domain, "DSDECOD") {
            if let Some((_header, mapped, raw)) = ct_column_match(table, domain, ct) {
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
                df.with_column(Series::new("DSDECOD".into(), decod_vals))?;
                df.with_column(Series::new("DSTERM".into(), term_vals))?;
            }
        }
    } else if code == "DA" {
        let ctdatest = ctx.resolve_ct(domain, "DATEST");
        let ctdatestcd = ctx.resolve_ct(domain, "DATESTCD");
        let mut candidates: Vec<(Option<String>, Option<String>, Vec<String>)> = Vec::new();
        for header in &table.headers {
            let upper = header.to_uppercase();
            if let Some(prefix) = upper.strip_suffix("_DAORRES") {
                if let Some(values) = table_column_values(table, header) {
                    let label = table_label(table, header);
                    let hint = label.clone().unwrap_or_else(|| prefix.to_string());
                    let test_code = ctdatestcd
                        .and_then(|ct| resolve_ct_value_from_hint(ct, &hint))
                        .or_else(|| {
                            ctdatestcd.and_then(|ct| resolve_ct_value_from_hint(ct, prefix))
                        });
                    let mut test_name = ctdatest
                        .and_then(|ct| resolve_ct_value_from_hint(ct, &hint))
                        .or_else(|| ctdatest.and_then(|ct| resolve_ct_value_from_hint(ct, prefix)));
                    if test_name.is_none() {
                        if let (Some(ct), Some(code)) = (ctdatestcd, test_code.as_ref()) {
                            test_name = ct.preferred_terms.get(code).cloned();
                        }
                    }
                    candidates.push((test_name, test_code, values));
                }
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
            for idx in 0..df.height() {
                let needs_orres = daorres_vals[idx].trim().is_empty();
                let needs_test = datest_vals[idx].trim().is_empty();
                let needs_testcd = datestcd_vals[idx].trim().is_empty();
                if !needs_orres && !needs_test && !needs_testcd {
                    continue;
                }
                for (test_name, test_code, values) in &candidates {
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
                    break;
                }
            }
            df.with_column(Series::new("DAORRES".into(), daorres_vals))?;
            df.with_column(Series::new("DATEST".into(), datest_vals))?;
            df.with_column(Series::new("DATESTCD".into(), datestcd_vals))?;
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
            for idx in 0..df.height() {
                let needs_test = ietest_vals[idx].trim().is_empty();
                let needs_testcd = ietestcd_vals[idx].trim().is_empty();
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
                        ietestcd_vals[idx] = sanitize_vstestcd(label);
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

fn print_summary(result: &StudyResult) {
    println!("Study: {}", result.study_id);
    println!("Output: {}", result.output_dir.display());
    if let Some(path) = &result.define_xml {
        println!("Define-XML: {}", path.display());
    }
    if let Some(path) = &result.conformance_report {
        println!("Conformance report: {}", path.display());
    }
    let mut table = Table::new();
    apply_table_style(&mut table);
    table.set_header(vec![
        "Domain",
        "Description",
        "Records",
        "XPT",
        "XML",
        "SAS",
        "Errors",
        "Warnings",
    ]);
    align_column(&mut table, 2, CellAlignment::Right);
    align_column(&mut table, 3, CellAlignment::Center);
    align_column(&mut table, 4, CellAlignment::Center);
    align_column(&mut table, 5, CellAlignment::Center);
    align_column(&mut table, 6, CellAlignment::Right);
    align_column(&mut table, 7, CellAlignment::Right);
    let ordered = ordered_summaries(&result.domains);
    let mut total_records = 0usize;
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;
    for summary in ordered {
        let (errors, warnings) = match &summary.conformance {
            Some(report) => (Some(report.error_count()), Some(report.warning_count())),
            None => (None, None),
        };
        total_records += summary.records;
        if let Some(count) = errors {
            total_errors += count;
        }
        if let Some(count) = warnings {
            total_warnings += count;
        }
        table.add_row(vec![
            Cell::new(summary.domain_code.clone()),
            Cell::new(summary.description.clone()),
            Cell::new(summary.records),
            output_cell(summary.outputs.xpt.as_ref()),
            output_cell(summary.outputs.dataset_xml.as_ref()),
            output_cell(summary.outputs.sas.as_ref()),
            count_cell(errors, Color::Red),
            count_cell(warnings, Color::Yellow),
        ]);
    }
    table.add_row(vec![
        Cell::new("TOTAL").add_attribute(Attribute::Bold),
        Cell::new("All domains").add_attribute(Attribute::Bold),
        Cell::new(total_records).add_attribute(Attribute::Bold),
        dim_cell("-"),
        dim_cell("-"),
        dim_cell("-"),
        count_cell(Some(total_errors), Color::Red).add_attribute(Attribute::Bold),
        count_cell(Some(total_warnings), Color::Yellow).add_attribute(Attribute::Bold),
    ]);
    println!("{table}");
    print_issue_table(result);
    if !result.errors.is_empty() {
        eprintln!("Errors:");
        for error in &result.errors {
            eprintln!("- {error}");
        }
    }
}

fn print_issue_table(result: &StudyResult) {
    let mut issues = Vec::new();
    let ordered = ordered_summaries(&result.domains);
    for summary in ordered {
        let report = match &summary.conformance {
            Some(report) => report,
            None => continue,
        };
        for issue in &report.issues {
            let (message, examples) = split_examples(&issue.message);
            issues.push((
                summary.domain_code.clone(),
                issue.severity,
                issue.variable.clone().unwrap_or_else(|| "-".to_string()),
                issue.code.clone(),
                issue
                    .count
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                issue.rule_id.clone().unwrap_or_else(|| "-".to_string()),
                issue.category.clone().unwrap_or_else(|| "-".to_string()),
                message,
                examples,
            ));
        }
    }
    if issues.is_empty() {
        println!();
        println!("Issues: none");
        return;
    }
    let mut table = Table::new();
    apply_table_style(&mut table);
    table.set_header(vec![
        "Domain", "Severity", "Variable", "Code", "Count", "Rule", "Category", "Message",
        "Examples",
    ]);
    align_column(&mut table, 4, CellAlignment::Right);
    if let Some(column) = table.column_mut(7) {
        column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(70)));
    }
    if let Some(column) = table.column_mut(8) {
        column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(40)));
    }
    for (domain, severity, variable, code, count, rule, category, message, examples) in issues {
        table.add_row(vec![
            Cell::new(domain),
            severity_cell(severity),
            Cell::new(variable),
            Cell::new(code),
            Cell::new(count),
            Cell::new(rule),
            Cell::new(category),
            Cell::new(message),
            example_cell(examples),
        ]);
    }
    println!();
    println!("Issues:");
    println!("{table}");
}

fn output_cell(path: Option<&PathBuf>) -> Cell {
    if path.is_some() {
        Cell::new("yes").fg(Color::Green)
    } else {
        dim_cell("-")
    }
}

fn count_cell(count: Option<usize>, color: Color) -> Cell {
    match count {
        Some(value) => {
            let cell = Cell::new(value);
            if value > 0 {
                cell.fg(color)
            } else {
                dim_cell("0")
            }
        }
        None => dim_cell("-"),
    }
}

fn apply_table_style(table: &mut Table) {
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
}

fn align_column(table: &mut Table, index: usize, alignment: CellAlignment) {
    if let Some(column) = table.column_mut(index) {
        column.set_cell_alignment(alignment);
    }
}

fn ordered_summaries<'a>(summaries: &'a [DomainSummary]) -> Vec<&'a DomainSummary> {
    let mut ordered: Vec<&DomainSummary> = summaries.iter().collect();
    ordered.sort_by(|a, b| summary_sort_key(&a.domain_code).cmp(&summary_sort_key(&b.domain_code)));
    ordered
}

fn summary_sort_key(code: &str) -> (String, u8, String) {
    let upper = code.to_uppercase();
    if let Some(parent) = upper.strip_prefix("SUPP") {
        return (parent.to_string(), 1, upper);
    }
    (upper.clone(), 0, upper)
}

fn severity_cell(severity: IssueSeverity) -> Cell {
    match severity {
        IssueSeverity::Reject => Cell::new("reject")
            .fg(Color::Red)
            .add_attribute(Attribute::Bold),
        IssueSeverity::Error => Cell::new("error").fg(Color::Red),
        IssueSeverity::Warning => Cell::new("warning").fg(Color::Yellow),
    }
}

fn split_examples(message: &str) -> (String, String) {
    match message.rsplit_once(" examples: ") {
        Some((head, tail)) => (head.to_string(), tail.to_string()),
        None => (message.to_string(), "-".to_string()),
    }
}

fn example_cell(value: String) -> Cell {
    if value == "-" {
        dim_cell(value)
    } else {
        Cell::new(value)
    }
}

fn dim_cell<T: ToString>(value: T) -> Cell {
    Cell::new(value).fg(Color::DarkGrey)
}

#[derive(Debug, Default, Clone)]
struct LbWideGroup {
    key: String,
    test_col: Option<usize>,
    testcd_col: Option<usize>,
    orres_col: Option<usize>,
    orresu_col: Option<usize>,
    orresu_alt_col: Option<usize>,
    ornr_range_col: Option<usize>,
    ornr_lower_col: Option<usize>,
    ornr_upper_col: Option<usize>,
    range_col: Option<usize>,
    clsig_col: Option<usize>,
    date_col: Option<usize>,
    time_col: Option<usize>,
    extra_cols: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
struct VsWideGroup {
    key: String,
    label: Option<String>,
    orres_col: Option<usize>,
    orresu_col: Option<usize>,
    pos_col: Option<usize>,
    extra_cols: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
struct VsWideShared {
    orresu_bp: Option<usize>,
    pos_bp: Option<usize>,
}

fn build_lb_wide_frame(
    table: &sdtm_ingest::CsvTable,
    domain: &Domain,
    study_id: &str,
) -> Result<Option<(MappingConfig, DomainFrame, BTreeSet<String>)>> {
    let (groups, wide_columns) = detect_lb_wide_groups(&table.headers);
    if groups.is_empty() {
        return Ok(None);
    }
    let base_table = filter_table_columns(table, &wide_columns, false);
    let hints = build_column_hints(&base_table);
    let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
    let result = engine.suggest(&base_table.headers);
    let mapping_config = engine.to_config(study_id, result);
    let base_frame = build_domain_frame_with_mapping(&base_table, domain, Some(&mapping_config))?;
    let date_idx = find_lb_date_column(&table.headers);
    let time_idx = find_lb_time_column(&table.headers);
    let (expanded, used_wide) =
        expand_lb_wide(table, &base_frame.data, domain, &groups, date_idx, time_idx)?;
    let mut used: BTreeSet<String> = mapping_config
        .mappings
        .iter()
        .map(|mapping| mapping.source_column.clone())
        .collect();
    used.extend(used_wide);
    Ok(Some((
        mapping_config,
        DomainFrame {
            domain_code: domain.code.clone(),
            data: expanded,
        },
        used,
    )))
}

fn build_vs_wide_frame(
    table: &sdtm_ingest::CsvTable,
    domain: &Domain,
    study_id: &str,
) -> Result<Option<(MappingConfig, DomainFrame, BTreeSet<String>)>> {
    let (groups, shared, wide_columns) =
        detect_vs_wide_groups(&table.headers, table.labels.as_deref());
    if groups.is_empty() {
        return Ok(None);
    }
    let base_table = filter_table_columns(table, &wide_columns, false);
    let hints = build_column_hints(&base_table);
    let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
    let result = engine.suggest(&base_table.headers);
    let mapping_config = engine.to_config(study_id, result);
    let base_frame = build_domain_frame_with_mapping(&base_table, domain, Some(&mapping_config))?;
    let date_idx = find_vs_date_column(&table.headers);
    let time_idx = find_vs_time_column(&table.headers);
    let (expanded, used_wide) = expand_vs_wide(
        table,
        &base_frame.data,
        domain,
        &groups,
        &shared,
        date_idx,
        time_idx,
    )?;
    let mut used: BTreeSet<String> = mapping_config
        .mappings
        .iter()
        .map(|mapping| mapping.source_column.clone())
        .collect();
    used.extend(used_wide);
    Ok(Some((
        mapping_config,
        DomainFrame {
            domain_code: domain.code.clone(),
            data: expanded,
        },
        used,
    )))
}

fn detect_lb_wide_groups(headers: &[String]) -> (BTreeMap<String, LbWideGroup>, BTreeSet<String>) {
    let mut groups: BTreeMap<String, LbWideGroup> = BTreeMap::new();
    let mut wide_columns = BTreeSet::new();
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        let mut matched = false;
        for prefix in [
            "TEST", "ORRES", "ORRESU", "ORRESUO", "ORNR", "RANGE", "CLSIG",
        ] {
            let prefix_tag = format!("{prefix}_");
            if !upper.starts_with(&prefix_tag) {
                continue;
            }
            matched = true;
            let rest = &upper[prefix_tag.len()..];
            let (mut key, attr) = if prefix == "ORNR" {
                if let Some(stripped) = rest.strip_suffix("_LOWER") {
                    (stripped.to_string(), Some("LOWER"))
                } else if let Some(stripped) = rest.strip_suffix("_UPPER") {
                    (stripped.to_string(), Some("UPPER"))
                } else {
                    (rest.to_string(), Some("RANGE"))
                }
            } else {
                (rest.to_string(), None)
            };
            let mut is_code = false;
            if key.len() > 2 && key.ends_with("CD") {
                key.truncate(key.len() - 2);
                is_code = true;
            }
            let entry = groups.entry(key.clone()).or_insert_with(|| LbWideGroup {
                key,
                ..LbWideGroup::default()
            });
            if is_code {
                entry.extra_cols.push(idx);
                break;
            }
            match prefix {
                "TEST" => entry.test_col = Some(idx),
                "ORRES" => entry.orres_col = Some(idx),
                "ORRESU" => entry.orresu_col = Some(idx),
                "ORRESUO" => entry.orresu_alt_col = Some(idx),
                "ORNR" => match attr {
                    Some("RANGE") => entry.ornr_range_col = Some(idx),
                    Some("LOWER") => entry.ornr_lower_col = Some(idx),
                    Some("UPPER") => entry.ornr_upper_col = Some(idx),
                    _ => {}
                },
                "RANGE" => entry.range_col = Some(idx),
                "CLSIG" => entry.clsig_col = Some(idx),
                _ => {}
            }
            break;
        }
        if matched {
            wide_columns.insert(upper);
        }
    }

    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if wide_columns.contains(&upper) || upper.contains('_') {
            continue;
        }
        if let Some(stripped) = upper.strip_suffix("CD") {
            if let Some((key, kind)) = parse_lb_suffix(stripped) {
                let entry = groups.entry(key.clone()).or_insert_with(|| LbWideGroup {
                    key,
                    ..LbWideGroup::default()
                });
                match kind {
                    LbSuffixKind::TestCd
                    | LbSuffixKind::Test
                    | LbSuffixKind::Orres
                    | LbSuffixKind::Orresu
                    | LbSuffixKind::OrresuAlt
                    | LbSuffixKind::OrnrRange
                    | LbSuffixKind::OrnrLower
                    | LbSuffixKind::OrnrUpper
                    | LbSuffixKind::Range
                    | LbSuffixKind::Clsig => {
                        entry.extra_cols.push(idx);
                        wide_columns.insert(upper);
                    }
                }
                continue;
            }
        }
        if let Some((key, kind)) = parse_lb_suffix(&upper) {
            let entry = groups.entry(key.clone()).or_insert_with(|| LbWideGroup {
                key,
                ..LbWideGroup::default()
            });
            match kind {
                LbSuffixKind::TestCd => entry.testcd_col = Some(idx),
                LbSuffixKind::Test => entry.test_col = Some(idx),
                LbSuffixKind::Orres => entry.orres_col = Some(idx),
                LbSuffixKind::Orresu => entry.orresu_col = Some(idx),
                LbSuffixKind::OrresuAlt => entry.orresu_alt_col = Some(idx),
                LbSuffixKind::OrnrRange => entry.ornr_range_col = Some(idx),
                LbSuffixKind::OrnrLower => entry.ornr_lower_col = Some(idx),
                LbSuffixKind::OrnrUpper => entry.ornr_upper_col = Some(idx),
                LbSuffixKind::Range => entry.range_col = Some(idx),
                LbSuffixKind::Clsig => entry.clsig_col = Some(idx),
            }
            wide_columns.insert(upper);
        }
    }

    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if wide_columns.contains(&upper) || upper.contains('_') {
            continue;
        }
        if let Some((key, is_time)) = parse_lb_time_suffix(&upper) {
            if let Some(entry) = groups.get_mut(&key) {
                if is_time {
                    entry.time_col = Some(idx);
                } else {
                    entry.date_col = Some(idx);
                }
                wide_columns.insert(upper);
            }
        }
    }
    (groups, wide_columns)
}

#[derive(Debug, Clone, Copy)]
enum LbSuffixKind {
    TestCd,
    Test,
    Orres,
    Orresu,
    OrresuAlt,
    OrnrRange,
    OrnrLower,
    OrnrUpper,
    Range,
    Clsig,
}

fn parse_lb_suffix(value: &str) -> Option<(String, LbSuffixKind)> {
    let patterns = [
        ("TESTCD", LbSuffixKind::TestCd),
        ("TEST", LbSuffixKind::Test),
        ("ORRESUO", LbSuffixKind::OrresuAlt),
        ("ORRESU", LbSuffixKind::Orresu),
        ("ORRES", LbSuffixKind::Orres),
        ("ORNRLOWER", LbSuffixKind::OrnrLower),
        ("ORNRUPPER", LbSuffixKind::OrnrUpper),
        ("ORNRLO", LbSuffixKind::OrnrLower),
        ("ORNRHI", LbSuffixKind::OrnrUpper),
        ("ORNR", LbSuffixKind::OrnrRange),
        ("CLSIG", LbSuffixKind::Clsig),
        ("RANGE", LbSuffixKind::Range),
    ];
    for (suffix, kind) in patterns {
        if value.len() > suffix.len() && value.ends_with(suffix) {
            let key = value[..value.len() - suffix.len()]
                .trim_end_matches('_')
                .to_string();
            if !key.is_empty() {
                return Some((key, kind));
            }
        }
    }
    None
}

fn parse_lb_time_suffix(value: &str) -> Option<(String, bool)> {
    let patterns = [
        ("DATE", false),
        ("DAT", false),
        ("TIME", true),
        ("TIM", true),
    ];
    for (suffix, is_time) in patterns {
        if value.len() > suffix.len() && value.ends_with(suffix) {
            let key = value[..value.len() - suffix.len()]
                .trim_end_matches('_')
                .to_string();
            if !key.is_empty() {
                return Some((key, is_time));
            }
        }
    }
    None
}

fn detect_vs_wide_groups(
    headers: &[String],
    labels: Option<&[String]>,
) -> (
    BTreeMap<String, VsWideGroup>,
    VsWideShared,
    BTreeSet<String>,
) {
    let mut groups: BTreeMap<String, VsWideGroup> = BTreeMap::new();
    let mut wide_columns = BTreeSet::new();
    let mut shared = VsWideShared::default();
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if let Some(rest) = upper.strip_prefix("ORRES_") {
            let key = rest.to_string();
            let entry = groups.entry(key.clone()).or_insert_with(|| VsWideGroup {
                key,
                ..VsWideGroup::default()
            });
            entry.orres_col = Some(idx);
            if entry.label.is_none() {
                if let Some(labels) = labels {
                    if let Some(label) = labels.get(idx) {
                        let trimmed = label.trim();
                        if !trimmed.is_empty() {
                            entry.label = Some(trimmed.to_string());
                        }
                    }
                }
            }
            wide_columns.insert(upper);
            continue;
        }
        if let Some(rest) = upper.strip_prefix("ORRESU_") {
            let mut key = rest.to_string();
            if key.len() > 2 && key.ends_with("CD") {
                key.truncate(key.len() - 2);
                let entry = groups.entry(key.clone()).or_insert_with(|| VsWideGroup {
                    key,
                    ..VsWideGroup::default()
                });
                entry.extra_cols.push(idx);
                wide_columns.insert(upper);
                continue;
            }
            if key == "BP" {
                shared.orresu_bp = Some(idx);
            } else {
                let entry = groups.entry(key.clone()).or_insert_with(|| VsWideGroup {
                    key,
                    ..VsWideGroup::default()
                });
                entry.orresu_col = Some(idx);
            }
            wide_columns.insert(upper);
            continue;
        }
        if let Some(rest) = upper.strip_prefix("POS_") {
            let mut key = rest.to_string();
            if key.len() > 2 && key.ends_with("CD") {
                key.truncate(key.len() - 2);
                let entry = groups.entry(key.clone()).or_insert_with(|| VsWideGroup {
                    key,
                    ..VsWideGroup::default()
                });
                entry.extra_cols.push(idx);
                wide_columns.insert(upper);
                continue;
            }
            if key == "BP" {
                shared.pos_bp = Some(idx);
            } else {
                let entry = groups.entry(key.clone()).or_insert_with(|| VsWideGroup {
                    key,
                    ..VsWideGroup::default()
                });
                entry.pos_col = Some(idx);
            }
            wide_columns.insert(upper);
            continue;
        }
    }
    (groups, shared, wide_columns)
}

fn filter_table_columns(
    table: &sdtm_ingest::CsvTable,
    columns: &BTreeSet<String>,
    include: bool,
) -> sdtm_ingest::CsvTable {
    let mut indices = Vec::new();
    let mut headers = Vec::new();
    let mut labels = table.labels.as_ref().map(|_| Vec::new());
    for (idx, header) in table.headers.iter().enumerate() {
        let has = columns.contains(&header.to_uppercase());
        if has == include {
            indices.push(idx);
            headers.push(header.clone());
            if let Some(label_vec) = table.labels.as_ref() {
                if let Some(labels_mut) = labels.as_mut() {
                    labels_mut.push(label_vec.get(idx).cloned().unwrap_or_default());
                }
            }
        }
    }
    let mut rows = Vec::with_capacity(table.rows.len());
    for row in &table.rows {
        let mut next = Vec::with_capacity(indices.len());
        for &idx in &indices {
            next.push(row.get(idx).cloned().unwrap_or_default());
        }
        rows.push(next);
    }
    sdtm_ingest::CsvTable {
        headers,
        rows,
        labels,
    }
}

fn find_vs_date_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("DAT") || upper.ends_with("DATE"))
            && upper.contains("VS")
            && !upper.contains("EVENT")
        {
            return Some(idx);
        }
    }
    find_lb_date_column(headers)
}

fn find_vs_time_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("TIM") || upper.ends_with("TIME"))
            && upper.contains("VS")
            && !upper.contains("EVENT")
        {
            return Some(idx);
        }
    }
    find_lb_time_column(headers)
}

fn find_lb_date_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("DAT") || upper.ends_with("DATE")) && !upper.contains("EVENT") {
            return Some(idx);
        }
    }
    None
}

fn find_lb_time_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("TIM") || upper.ends_with("TIME")) && !upper.contains("EVENT") {
            return Some(idx);
        }
    }
    None
}

fn expand_vs_wide(
    table: &sdtm_ingest::CsvTable,
    base_df: &DataFrame,
    domain: &Domain,
    groups: &BTreeMap<String, VsWideGroup>,
    shared: &VsWideShared,
    date_idx: Option<usize>,
    time_idx: Option<usize>,
) -> Result<(DataFrame, BTreeSet<String>)> {
    let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for variable in &domain.variables {
        values.insert(variable.name.clone(), Vec::new());
    }
    let mut used = BTreeSet::new();
    for group in groups.values() {
        for idx in [group.orres_col, group.orresu_col, group.pos_col] {
            if let Some(idx) = idx {
                if let Some(name) = table.headers.get(idx) {
                    used.insert(name.clone());
                }
            }
        }
        for idx in &group.extra_cols {
            if let Some(name) = table.headers.get(*idx) {
                used.insert(name.clone());
            }
        }
    }
    for idx in [shared.orresu_bp, shared.pos_bp] {
        if let Some(idx) = idx {
            if let Some(name) = table.headers.get(idx) {
                used.insert(name.clone());
            }
        }
    }
    if let Some(idx) = date_idx {
        if let Some(name) = table.headers.get(idx) {
            used.insert(name.clone());
        }
    }
    if let Some(idx) = time_idx {
        if let Some(name) = table.headers.get(idx) {
            used.insert(name.clone());
        }
    }

    let mut total_rows = 0usize;
    for row_idx in 0..table.rows.len() {
        let date_value = date_idx
            .and_then(|idx| table.rows[row_idx].get(idx))
            .cloned()
            .unwrap_or_default();
        let time_value = time_idx
            .and_then(|idx| table.rows[row_idx].get(idx))
            .cloned()
            .unwrap_or_default();
        for group in groups.values() {
            let orres_value = group
                .orres_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let orresu_value = group
                .orresu_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let pos_value = group
                .pos_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let orresu_fallback = if group.key.ends_with("BP") || group.key.contains("BP") {
                shared
                    .orresu_bp
                    .and_then(|idx| table.rows[row_idx].get(idx))
                    .cloned()
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let pos_fallback = if group.key.ends_with("BP") || group.key.contains("BP") {
                shared
                    .pos_bp
                    .and_then(|idx| table.rows[row_idx].get(idx))
                    .cloned()
                    .unwrap_or_default()
            } else {
                String::new()
            };
            if orres_value.trim().is_empty()
                && orresu_value.trim().is_empty()
                && pos_value.trim().is_empty()
            {
                continue;
            }

            total_rows += 1;
            let test_code = sanitize_vstestcd(&group.key);
            let test_label = group.label.clone().unwrap_or_default();
            let mut base_values: BTreeMap<String, String> = BTreeMap::new();
            for variable in &domain.variables {
                let val = column_value_string(base_df, &variable.name, row_idx);
                base_values.insert(variable.name.clone(), val);
            }
            if let Some(value) = base_values.get_mut("VSTESTCD") {
                *value = test_code.clone();
            }
            if let Some(value) = base_values.get_mut("VSTEST") {
                if !test_label.is_empty() {
                    *value = test_label.clone();
                } else if !test_code.is_empty() {
                    *value = test_code.clone();
                }
            }
            if let Some(value) = base_values.get_mut("VSORRES") {
                *value = orres_value.clone();
            }
            if let Some(value) = base_values.get_mut("VSORRESU") {
                if !orresu_value.trim().is_empty() {
                    *value = orresu_value.clone();
                } else {
                    *value = orresu_fallback.clone();
                }
            }
            if let Some(value) = base_values.get_mut("VSPOS") {
                if !pos_value.trim().is_empty() {
                    *value = pos_value.clone();
                } else {
                    *value = pos_fallback.clone();
                }
            }
            if let Some(value) = base_values.get_mut("VSDTC") {
                if !date_value.trim().is_empty() {
                    if !time_value.trim().is_empty() && !date_value.contains('T') {
                        *value = format!("{}T{}", date_value.trim(), time_value.trim());
                    } else {
                        *value = date_value.clone();
                    }
                }
            }

            for (name, list) in values.iter_mut() {
                let value = base_values.get(name).cloned().unwrap_or_default();
                list.push(value);
            }
        }
    }
    if total_rows == 0 {
        return Ok((base_df.clone(), used));
    }
    let mut columns = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let vals = values.remove(&variable.name).unwrap_or_default();
        let column = match variable.data_type {
            sdtm_model::VariableType::Num => {
                let numeric: Vec<Option<f64>> = vals
                    .iter()
                    .map(|value| value.trim().parse::<f64>().ok())
                    .collect();
                polars::prelude::Series::new(variable.name.as_str().into(), numeric).into()
            }
            sdtm_model::VariableType::Char => {
                polars::prelude::Series::new(variable.name.as_str().into(), vals).into()
            }
        };
        columns.push(column);
    }
    let data = DataFrame::new(columns)?;
    Ok((data, used))
}

fn expand_lb_wide(
    table: &sdtm_ingest::CsvTable,
    base_df: &DataFrame,
    domain: &Domain,
    groups: &BTreeMap<String, LbWideGroup>,
    date_idx: Option<usize>,
    time_idx: Option<usize>,
) -> Result<(DataFrame, BTreeSet<String>)> {
    let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for variable in &domain.variables {
        values.insert(variable.name.clone(), Vec::new());
    }
    let mut used = BTreeSet::new();
    for group in groups.values() {
        for idx in [
            group.test_col,
            group.testcd_col,
            group.orres_col,
            group.orresu_col,
            group.orresu_alt_col,
            group.ornr_range_col,
            group.ornr_lower_col,
            group.ornr_upper_col,
            group.range_col,
            group.clsig_col,
            group.date_col,
            group.time_col,
        ] {
            if let Some(idx) = idx {
                if let Some(name) = table.headers.get(idx) {
                    used.insert(name.clone());
                }
            }
        }
        for idx in &group.extra_cols {
            if let Some(name) = table.headers.get(*idx) {
                used.insert(name.clone());
            }
        }
    }
    if let Some(idx) = date_idx {
        if let Some(name) = table.headers.get(idx) {
            used.insert(name.clone());
        }
    }
    if let Some(idx) = time_idx {
        if let Some(name) = table.headers.get(idx) {
            used.insert(name.clone());
        }
    }
    let mut total_rows = 0usize;
    for row_idx in 0..table.rows.len() {
        let base_date_value = date_idx
            .and_then(|idx| table.rows[row_idx].get(idx))
            .cloned()
            .unwrap_or_default();
        let base_time_value = time_idx
            .and_then(|idx| table.rows[row_idx].get(idx))
            .cloned()
            .unwrap_or_default();
        for group in groups.values() {
            let group_date_value = group
                .date_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let group_time_value = group
                .time_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let date_value = if !group_date_value.trim().is_empty() {
                group_date_value
            } else {
                base_date_value.clone()
            };
            let time_value = if !group_time_value.trim().is_empty() {
                group_time_value
            } else {
                base_time_value.clone()
            };
            let test_value = group
                .test_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let testcd_value = group
                .testcd_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let orres_value = group
                .orres_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let orresu_value = group
                .orresu_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let orresu_alt_value = group
                .orresu_alt_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let ornr_lower_value = group
                .ornr_lower_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let ornr_upper_value = group
                .ornr_upper_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let clsig_value = group
                .clsig_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();

            if test_value.trim().is_empty()
                && orres_value.trim().is_empty()
                && orresu_value.trim().is_empty()
                && ornr_lower_value.trim().is_empty()
                && ornr_upper_value.trim().is_empty()
                && clsig_value.trim().is_empty()
            {
                continue;
            }

            total_rows += 1;
            let test_code = if !testcd_value.trim().is_empty() {
                sanitize_lbtestcd(testcd_value.trim())
            } else {
                sanitize_lbtestcd(&group.key)
            };
            let mut base_values: BTreeMap<String, String> = BTreeMap::new();
            for variable in &domain.variables {
                let val = column_value_string(base_df, &variable.name, row_idx);
                base_values.insert(variable.name.clone(), val);
            }
            if let Some(value) = base_values.get_mut("LBTESTCD") {
                *value = test_code.clone();
            }
            if let Some(value) = base_values.get_mut("LBTEST") {
                if !test_value.trim().is_empty() {
                    *value = test_value.clone();
                } else {
                    *value = test_code.clone();
                }
            }
            if let Some(value) = base_values.get_mut("LBORRES") {
                *value = orres_value.clone();
            }
            if let Some(value) = base_values.get_mut("LBORRESU") {
                if !orresu_value.trim().is_empty() {
                    *value = orresu_value.clone();
                } else {
                    *value = orresu_alt_value.clone();
                }
            }
            if let Some(value) = base_values.get_mut("LBORNRLO") {
                *value = ornr_lower_value.clone();
            }
            if let Some(value) = base_values.get_mut("LBORNRHI") {
                *value = ornr_upper_value.clone();
            }
            if let Some(value) = base_values.get_mut("LBCLSIG") {
                *value = clsig_value.clone();
            }
            if let Some(value) = base_values.get_mut("LBDTC") {
                if !date_value.trim().is_empty() {
                    if !time_value.trim().is_empty() && !date_value.contains('T') {
                        *value = format!("{}T{}", date_value.trim(), time_value.trim());
                    } else {
                        *value = date_value.clone();
                    }
                }
            }

            for (name, list) in values.iter_mut() {
                let value = base_values.get(name).cloned().unwrap_or_default();
                list.push(value);
            }
        }
    }
    if total_rows == 0 {
        return Ok((base_df.clone(), used));
    }
    let mut columns = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let vals = values.remove(&variable.name).unwrap_or_default();
        let column = match variable.data_type {
            sdtm_model::VariableType::Num => {
                let numeric: Vec<Option<f64>> = vals
                    .iter()
                    .map(|value| value.trim().parse::<f64>().ok())
                    .collect();
                polars::prelude::Series::new(variable.name.as_str().into(), numeric).into()
            }
            sdtm_model::VariableType::Char => {
                polars::prelude::Series::new(variable.name.as_str().into(), vals).into()
            }
        };
        columns.push(column);
    }
    let data = DataFrame::new(columns)?;
    Ok((data, used))
}

fn sanitize_lbtestcd(raw: &str) -> String {
    let mut safe = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            safe.push(ch.to_ascii_uppercase());
        } else {
            safe.push('_');
        }
    }
    if safe.is_empty() {
        safe = "TEST".to_string();
    }
    safe.chars().take(8).collect()
}

fn sanitize_vstestcd(raw: &str) -> String {
    let mut safe = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            safe.push(ch.to_ascii_uppercase());
        } else {
            safe.push('_');
        }
    }
    if safe.is_empty() {
        safe = "TEST".to_string();
    }
    safe.chars().take(8).collect()
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

fn column_value_string(df: &DataFrame, name: &str, idx: usize) -> String {
    match df.column(name) {
        Ok(series) => any_to_string(series.get(idx).unwrap_or(AnyValue::Null)),
        Err(_) => String::new(),
    }
}

fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        AnyValue::Null => String::new(),
        _ => value.to_string(),
    }
}
