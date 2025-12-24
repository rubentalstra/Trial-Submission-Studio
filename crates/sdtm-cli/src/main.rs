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
    }
}
