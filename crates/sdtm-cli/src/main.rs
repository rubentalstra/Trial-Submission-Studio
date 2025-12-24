use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use comfy_table::Table;
use comfy_table::presets::ASCII_FULL;
use polars::prelude::DataFrame;

use sdtm_cli::logging::init_logging;
use sdtm_core::{
    DomainFrame, ProcessingContext, build_domain_frame, build_domain_frame_with_mapping,
    build_relationship_frames, build_suppqual, process_domain_with_context,
};
use sdtm_ingest::{build_column_hints, discover_domain_files, list_csv_files, read_csv_table};
use sdtm_map::MappingEngine;
use sdtm_model::{ConformanceReport, MappingConfig, MappingSuggestion, OutputFormat};
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

    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
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
    table.load_preset(ASCII_FULL);
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
    let mut domain_map = BTreeMap::new();
    for domain in &standards {
        domain_map.insert(domain.code.to_uppercase(), domain);
    }

    let csv_files = list_csv_files(study_folder).context("list csv files")?;
    let discovered = discover_domain_files(&csv_files, &domain_codes);

    let mut processed_frames: BTreeMap<String, DomainFrame> = BTreeMap::new();
    let mut suppqual_frames: Vec<DomainFrame> = Vec::new();
    let mut mapping_configs: BTreeMap<String, Vec<MappingConfig>> = BTreeMap::new();
    let mut errors = Vec::new();

    let ctx = ProcessingContext::new(&study_id).with_ct_registry(&ct_registry);
    let suppqual_domain = domain_map
        .get("SUPPQUAL")
        .ok_or_else(|| anyhow!("missing SUPPQUAL metadata"))?;

    for (domain_code, files) in discovered {
        let domain = match domain_map.get(&domain_code.to_uppercase()) {
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
            let hints = build_column_hints(&table);
            let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
            let result = engine.suggest(&table.headers);
            let mapping_config = engine.to_config(&study_id, result);
            domain_mappings.push(mapping_config.clone());

            let mut mapped =
                match build_domain_frame_with_mapping(&table, domain, Some(&mapping_config)) {
                    Ok(frame) => frame,
                    Err(error) => {
                        errors.push(format!("{}: {error}", path.display()));
                        continue;
                    }
                };
            if let Err(error) = process_domain_with_context(domain, &mut mapped.data, &ctx) {
                errors.push(format!("{}: {error}", path.display()));
                continue;
            }

            if let Some(existing) = &mut combined {
                if let Err(error) = existing.vstack_mut(&mapped.data) {
                    errors.push(format!("{}: {error}", path.display()));
                }
            } else {
                combined = Some(mapped.data.clone());
            }

            let used: BTreeSet<String> = mapping_config
                .mappings
                .iter()
                .map(|mapping| mapping.source_column.clone())
                .collect();
            let source = match build_domain_frame(&table, &domain_code) {
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
    let relationship_frames = build_relationship_frames(
        &frames.values().cloned().collect::<Vec<_>>(),
        &standards,
        &study_id,
    )
    .context("build relationship domains")?;
    for frame in relationship_frames {
        if let Err(error) = insert_frame(&mut frames, frame) {
            errors.push(format!("relationship merge: {error}"));
        }
    }

    let mut frame_list: Vec<DomainFrame> = frames.into_values().collect();
    frame_list.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));

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
        if let Err(error) = write_define_xml(&path, &study_id, &standards, &frame_list, &options) {
            errors.push(format!("define-xml: {error}"));
            None
        } else {
            Some(path)
        }
    };

    if !args.dry_run {
        if want_xpt {
            let options = XptWriterOptions::default();
            match write_xpt_outputs(&output_dir, &standards, &frame_list, &options) {
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
            match write_dataset_xml_outputs(&output_dir, &standards, &frame_list, &study_id, "3.4")
            {
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
                &standards,
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
        let domain = domain_map.get(&code);
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
    table.load_preset(ASCII_FULL);
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
    for summary in &result.domains {
        let (errors, warnings) = match &summary.conformance {
            Some(report) => (
                report.error_count().to_string(),
                report.warning_count().to_string(),
            ),
            None => ("-".to_string(), "-".to_string()),
        };
        table.add_row(vec![
            summary.domain_code.clone(),
            summary.description.clone(),
            summary.records.to_string(),
            flag(summary.outputs.xpt.as_ref()),
            flag(summary.outputs.dataset_xml.as_ref()),
            flag(summary.outputs.sas.as_ref()),
            errors,
            warnings,
        ]);
    }
    println!("{table}");
    if !result.errors.is_empty() {
        eprintln!("Errors:");
        for error in &result.errors {
            eprintln!("- {error}");
        }
    }
}

fn flag(path: Option<&PathBuf>) -> String {
    if path.is_some() {
        "yes".to_string()
    } else {
        "-".to_string()
    }
}
