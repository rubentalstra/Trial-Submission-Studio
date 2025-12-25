use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "cdisc-transpiler", version, about = "CDISC Transpiler CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
    pub verbose: u8,
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
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormatArg {
    Xpt,
    Xml,
    Both,
}
