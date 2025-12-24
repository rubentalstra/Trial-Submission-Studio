#![deny(unsafe_code)]

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "sdtm")]
#[command(about = "Offline SDTM transpiler (Phase 0 bootstrap)")]
struct Cli {
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
                println!("standards verify: not implemented yet");
                Ok(())
            }
            StandardsCommand::Summary => {
                println!("standards summary: not implemented yet");
                Ok(())
            }
            StandardsCommand::Doctor { json } => {
                let out = serde_json::json!({
                    "schema": "sdtm.doctor.phase0",
                    "json": json,
                });
                if json == "-" {
                    println!("{}", serde_json::to_string_pretty(&out)?);
                } else {
                    std::fs::write(&json, serde_json::to_string_pretty(&out)?)?;
                    println!("wrote {}", json);
                }
                Ok(())
            }
        },
    }
}
