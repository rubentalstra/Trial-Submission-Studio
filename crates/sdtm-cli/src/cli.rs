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
                  Supports XPT (SAS Transport), Dataset-XML, and Define-XML outputs.\n\
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

    /// Output format to generate.
    #[arg(long = "format", value_enum, default_value = "both")]
    pub format: OutputFormatArg,

    /// Validate and report without writing output files.
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Skip adding STUDYID prefix to USUBJID values.
    #[arg(long = "no-usubjid-prefix")]
    pub no_usubjid_prefix: bool,

    /// Skip automatic sequence number generation.
    #[arg(long = "no-auto-seq")]
    pub no_auto_seq: bool,

    /// Continue writing outputs even if conformance errors are detected.
    ///
    /// By default, the transpiler blocks XPT output generation when conformance
    /// errors are found. Use this flag to override that behavior and write
    /// outputs regardless of validation results.
    ///
    /// WARNING: Outputs generated with this flag may not be conformant.
    #[arg(long = "no-fail-on-conformance-errors")]
    pub no_fail_on_conformance_errors: bool,

    /// Skip Define-XML generation.
    #[arg(long = "no-define-xml")]
    pub no_define_xml: bool,

    /// Skip SAS program generation.
    #[arg(long = "no-sas")]
    pub no_sas: bool,

    /// Enable strict SDTMIG-conformant processing.
    ///
    /// Disables lenient CT matching while keeping documented SDTMIG
    /// derivations (USUBJID prefixing, sequence assignment).
    ///
    /// Use this mode for production submissions requiring strict conformance.
    #[arg(long = "strict")]
    pub strict: bool,

    /// Disable lenient CT matching.
    ///
    /// When set, only exact matches and defined synonyms are allowed for
    /// controlled terminology normalization.
    #[arg(long = "no-lenient-ct")]
    pub no_lenient_ct: bool,
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
