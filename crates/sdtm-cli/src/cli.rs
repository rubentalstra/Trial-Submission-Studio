//! CLI argument definitions for the SDTM transpiler.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use colorchoice_clap::Color;

#[derive(Parser)]
#[command(
    name = "cdisc-transpiler",
    version,
    about = "CDISC SDTM Transpiler - Convert clinical data to SDTM format",
    long_about = "Convert clinical study data to CDISC SDTM format.\n\n\
                  Supports XPT (SAS Transport), Dataset-XML, Define-XML, and SAS program outputs.\n\
                  Validates data against SDTMIG v3.4 and Controlled Terminology."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Adjust log verbosity (-v for debug, -vv for trace, -q for errors only).
    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,

    /// Control ANSI color output (auto, always, never).
    #[command(flatten)]
    pub color: Color,

    /// Explicit log level (overrides -v/-q flags).
    #[arg(long = "log-level", value_enum, global = true)]
    pub log_level: Option<LogLevelArg>,

    /// Log output format (pretty for human, json for machine parsing).
    #[arg(
        long = "log-format",
        value_enum,
        default_value = "pretty",
        global = true
    )]
    pub log_format: LogFormatArg,

    /// Write logs to a file instead of stderr.
    #[arg(long = "log-file", value_name = "PATH", global = true)]
    pub log_file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Process a study folder and generate SDTM outputs.
    Study(StudyArgs),

    /// List all supported SDTM domains.
    Domains,
}

#[derive(Parser)]
pub struct StudyArgs {
    /// Path to the study data folder containing CSV files.
    #[arg(value_name = "STUDY_FOLDER")]
    pub study_folder: PathBuf,

    /// Output directory for generated files (default: <STUDY_FOLDER>/output).
    #[arg(long = "output-dir", value_name = "DIR")]
    pub output_dir: Option<PathBuf>,

    /// Output formats to generate (comma-separated, e.g. xpt,xml,sas).
    #[arg(long = "format", value_enum, value_delimiter = ',', default_value = "xpt,xml,sas")]
    pub format: Vec<OutputFormatArg>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormatArg {
    Xpt,
    Xml,
    Sas,
}

/// CLI log level choices.
#[derive(Clone, Copy, ValueEnum)]
pub enum LogLevelArg {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// CLI log format choices.
#[derive(Clone, Copy, ValueEnum)]
pub enum LogFormatArg {
    Pretty,
    Compact,
    Json,
}
