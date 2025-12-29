use std::path::Path;

use anyhow::{Context, Result};
use comfy_table::Table;

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
    let output_formats = format_outputs(&args.format);

    PipelineConfig {
        study_id,
        study_folder,
        output_dir,
        output_formats,
    }
}

fn format_outputs(formats: &[OutputFormatArg]) -> Vec<OutputFormat> {
    let mut output_formats = Vec::new();
    for format in formats {
        let output = match format {
            OutputFormatArg::Xpt => OutputFormat::Xpt,
            OutputFormatArg::Xml => OutputFormat::Xml,
            OutputFormatArg::Sas => OutputFormat::Sas,
        };
        if !output_formats.contains(&output) {
            output_formats.push(output);
        }
    }
    output_formats
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
