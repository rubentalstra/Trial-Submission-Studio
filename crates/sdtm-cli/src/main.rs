#![deny(unsafe_code)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "sdtm")]
#[command(about = "Offline SDTM transpiler (Phase 0 bootstrap)")]
struct Cli {
    /// Path to the offline standards directory.
    #[arg(long, default_value = "standards")]
    standards_dir: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Standards {
        #[command(subcommand)]
        command: StandardsCommand,
    },
    Run {
        #[command(subcommand)]
        command: RunCommand,
    },
    Validate {
        #[command(subcommand)]
        command: ValidateCommand,
    },
}

#[derive(Debug, Subcommand)]
enum StandardsCommand {
    Verify,
    Summary,
    Doctor {
        /// Write machine-readable JSON report to this path. Use '-' for stdout.
        #[arg(long, value_name = "PATH")]
        json: String,
    },
}

#[derive(Debug, Subcommand)]
enum ValidateCommand {
    Csv {
        /// SDTM domain code (e.g. DM, AE).
        #[arg(long)]
        domain: String,

        /// Input CSV file.
        #[arg(long, value_name = "PATH")]
        input: PathBuf,

        /// Stable source identifier used for deterministic RowId derivation.
        /// Defaults to the input path string.
        #[arg(long)]
        source_id: Option<String>,

        /// Write machine-readable JSON report to this path. Use '-' for stdout.
        #[arg(long, value_name = "PATH")]
        json: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum RunCommand {
    Csv {
        /// SDTM domain code (e.g. DM, AE).
        #[arg(long)]
        domain: String,

        /// Input CSV file.
        #[arg(long, value_name = "PATH")]
        input: PathBuf,

        /// Stable source identifier used for deterministic RowId derivation.
        /// Defaults to the input path string.
        #[arg(long)]
        source_id: Option<String>,

        /// Write machine-readable JSON report to this path. Use '-' for stdout.
        #[arg(long, value_name = "PATH")]
        json: Option<String>,
    },
}

#[derive(Debug, serde::Serialize)]
struct RunReport {
    domain: String,
    input: String,
    ingested_rows: usize,
    ingested_columns: usize,
    mapped_tables: usize,
    validation: sdtm_validate::ValidationReport,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Standards { command } => match command {
            StandardsCommand::Verify => {
                let (_registry, summary) =
                    sdtm_standards::StandardsRegistry::verify_and_load(&cli.standards_dir)
                        .map_err(|e| anyhow::anyhow!(e))?;

                println!(
                    "OK: verified {} files (SDTM domains={}, SDTMIG domains={}, CT codelists={}, conflicts={})",
                    summary.file_count,
                    summary.domain_count_sdtm,
                    summary.domain_count_sdtmig,
                    summary.codelist_count,
                    summary.conflict_count
                );
                Ok(())
            }
            StandardsCommand::Summary => {
                let (registry, summary) =
                    sdtm_standards::StandardsRegistry::verify_and_load(&cli.standards_dir)
                        .map_err(|e| anyhow::anyhow!(e))?;

                println!("Pins:");
                println!("  SDTM: {}", summary.manifest_pins.sdtm);
                println!("  SDTMIG: {}", summary.manifest_pins.sdtmig);
                println!("  CT: {}", summary.manifest_pins.ct);
                println!(
                    "  Conformance rules: {}",
                    summary.manifest_pins.conformance_rules
                );
                println!();
                println!("Counts:");
                println!("  files: {}", summary.file_count);
                println!("  sdtm domains: {}", summary.domain_count_sdtm);
                println!("  sdtmig domains: {}", summary.domain_count_sdtmig);
                println!("  sdtm variables: {}", summary.variable_count_sdtm);
                println!("  sdtmig variables: {}", summary.variable_count_sdtmig);
                println!("  ct codelists: {}", summary.codelist_count);
                println!();
                println!("Conflicts:");
                println!("  count: {}", registry.conflicts.len());
                Ok(())
            }
            StandardsCommand::Doctor { json } => {
                let (registry, summary) =
                    sdtm_standards::StandardsRegistry::verify_and_load(&cli.standards_dir)
                        .map_err(|e| anyhow::anyhow!(e))?;

                let report = sdtm_standards::DoctorReport::from_verify_summary(
                    &summary,
                    registry.manifest.policy.clone(),
                    registry.files.clone(),
                    registry.conflicts.clone(),
                );

                let out = serde_json::to_string_pretty(&report)?;
                if json == "-" {
                    println!("{}", out);
                } else {
                    std::fs::write(&json, out)?;
                    println!("wrote {}", json);
                }
                Ok(())
            }
        },
        Command::Run { command } => match command {
            RunCommand::Csv {
                domain,
                input,
                source_id,
                json,
            } => {
                let domain_code = sdtm_model::DomainCode::new(domain.clone())?;
                let source_id = source_id.unwrap_or_else(|| input.to_string_lossy().to_string());

                let table = sdtm_ingest::csv_ingest::ingest_csv_file(
                    domain_code,
                    &input,
                    sdtm_ingest::csv_ingest::CsvIngestOptions::new(source_id),
                )?;

                let ingested_rows = table.rows.len();
                let ingested_columns = table.columns.len();

                let mapper = sdtm_map::SimpleMapper::new();
                let mapped = sdtm_core::pipeline::Mapper::map(&mapper, table)?;

                let validator =
                    sdtm_validate::StandardsValidator::from_standards_dir(&cli.standards_dir)?;
                let validation = validator.validate_with_report(&mapped);

                let report = RunReport {
                    domain: domain.clone(),
                    input: input.to_string_lossy().to_string(),
                    ingested_rows,
                    ingested_columns,
                    mapped_tables: mapped.len(),
                    validation: validation.clone(),
                };

                if let Some(json) = json {
                    let out = serde_json::to_string_pretty(&report)?;
                    if json == "-" {
                        println!("{}", out);
                    } else {
                        std::fs::write(&json, out)?;
                        println!("wrote {}", json);
                    }
                } else {
                    println!(
                        "{}: pipeline (rows={}, cols={}, mapped_tables={}, errors={}, warnings={})",
                        domain,
                        report.ingested_rows,
                        report.ingested_columns,
                        report.mapped_tables,
                        report.validation.errors,
                        report.validation.warnings
                    );
                }

                if report.validation.errors > 0 {
                    return Err(anyhow::anyhow!("validation failed"));
                }
                Ok(())
            }
        },
        Command::Validate { command } => match command {
            ValidateCommand::Csv {
                domain,
                input,
                source_id,
                json,
            } => {
                let domain_code = sdtm_model::DomainCode::new(domain.clone())?;
                let source_id = source_id.unwrap_or_else(|| input.to_string_lossy().to_string());

                let table = sdtm_ingest::csv_ingest::ingest_csv_file(
                    domain_code,
                    &input,
                    sdtm_ingest::csv_ingest::CsvIngestOptions::new(source_id),
                )?;

                let validator =
                    sdtm_validate::StandardsValidator::from_standards_dir(&cli.standards_dir)?;
                let report = validator.validate_with_report(&[table]);

                if let Some(json) = json {
                    let out = serde_json::to_string_pretty(&report)?;
                    if json == "-" {
                        println!("{}", out);
                    } else {
                        std::fs::write(&json, out)?;
                        println!("wrote {}", json);
                    }
                } else {
                    println!(
                        "{}: conformance issues (errors={}, warnings={})",
                        domain, report.errors, report.warnings
                    );
                }

                if report.errors > 0 {
                    return Err(anyhow::anyhow!("validation failed"));
                }
                Ok(())
            }
        },
    }
}
