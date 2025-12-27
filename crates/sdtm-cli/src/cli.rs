use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "cdisc-transpiler", version, about = "CDISC Transpiler CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Reduce log output to errors only.
    #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue, global = true)]
    pub quiet: bool,

    /// Explicit log level (overrides verbosity/quiet).
    #[arg(long = "log-level", value_enum, global = true)]
    pub log_level: Option<LogLevelArg>,

    /// Log output format.
    #[arg(
        long = "log-format",
        value_enum,
        default_value = "pretty",
        global = true
    )]
    pub log_format: LogFormatArg,

    /// Write logs to the specified file instead of stderr.
    #[arg(long = "log-file", global = true)]
    pub log_file: Option<PathBuf>,

    /// Allow logging of row-level PHI/PII values.
    #[arg(long = "log-data", action = ArgAction::SetTrue, global = true)]
    pub log_data: bool,
}

#[derive(Subcommand)]
pub enum Command {
    Study(StudyArgs),
    Domains,
}

#[derive(Parser)]
pub struct StudyArgs {
    #[arg(value_name = "STUDY_FOLDER")]
    pub study_folder: PathBuf,

    #[arg(long = "output-dir")]
    pub output_dir: Option<PathBuf>,

    #[arg(long = "format", value_enum, default_value = "both")]
    pub format: OutputFormatArg,

    #[arg(long = "dry-run", default_value_t = false)]
    pub dry_run: bool,

    #[arg(long = "no-usubjid-prefix", default_value_t = false)]
    pub no_usubjid_prefix: bool,

    #[arg(long = "no-auto-seq", default_value_t = false)]
    pub no_auto_seq: bool,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormatArg {
    Xpt,
    Xml,
    Both,
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
