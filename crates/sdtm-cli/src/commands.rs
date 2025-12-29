use std::path::Path;

use anyhow::{Context, Result};
use comfy_table::Table;

use sdtm_core::pipeline_context::{
    CtMatchingMode, ProcessingOptions, SequenceAssignmentMode, UsubjidPrefixMode,
};
use sdtm_model::OutputFormat;
use sdtm_standards::load_default_sdtm_ig_domains;

use crate::cli::{OutputFormatArg, StudyArgs};
use crate::pipeline::{PipelineConfig, PipelineRunner};
use crate::summary::apply_table_style;
use crate::types::StudyResult;

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
    let config = build_pipeline_config(args);
    let runner = PipelineRunner::new(config)?;
    runner.run()
}

fn build_pipeline_config(args: &StudyArgs) -> PipelineConfig {
    let study_folder = args.study_folder.clone();
    let study_id = derive_study_id(&study_folder);
    let output_dir = args
        .output_dir
        .clone()
        .unwrap_or_else(|| study_folder.join("output"));
    let output_formats = format_outputs(args.format);
    let options = if args.strict {
        ProcessingOptions::strict()
    } else {
        ProcessingOptions {
            usubjid_prefix: if args.no_usubjid_prefix {
                UsubjidPrefixMode::Skip
            } else {
                UsubjidPrefixMode::Prefix
            },
            sequence_assignment: if args.no_auto_seq {
                SequenceAssignmentMode::Skip
            } else {
                SequenceAssignmentMode::Assign
            },
            warn_on_rewrite: true,
            ct_matching: if args.no_lenient_ct {
                CtMatchingMode::Strict
            } else {
                CtMatchingMode::Lenient
            },
        }
    };

    PipelineConfig {
        study_id,
        study_folder,
        output_dir,
        output_formats,
        dry_run: args.dry_run,
        fail_on_conformance_errors: !args.no_fail_on_conformance_errors,
        skip_define_xml: args.no_define_xml,
        skip_sas: args.no_sas,
        options,
    }
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
