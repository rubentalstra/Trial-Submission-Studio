#![deny(unsafe_code)]

use clap::{Parser, Subcommand};
use std::collections::BTreeMap;
use std::path::PathBuf;

use sdtm_standards::StandardsRegistry;

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
        #![deny(unsafe_code)]

        fn main() -> anyhow::Result<()> {
            sdtm_cli::run()
        }
#[derive(Debug, Subcommand)]
