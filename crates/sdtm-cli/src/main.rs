use clap::Parser;
use sdtm_cli::logging::init_logging;

mod cli;
mod commands;
mod pipeline;
mod summary;
mod types;

use crate::cli::{Cli, Command};
use crate::commands::{run_domains, run_study};
use crate::summary::print_summary;

fn main() {
    let cli = Cli::parse();
    init_logging(cli.verbose);
    let exit_code = match cli.command {
        Command::Study(args) => match run_study(&args) {
            Ok(result) => {
                print_summary(&result);
                if result.has_errors { 1 } else { 0 }
            }
            Err(error) => {
                eprintln!("error: {error}");
                1
            }
        },
        Command::Domains => match run_domains() {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("error: {error}");
                1
            }
        },
    };
    std::process::exit(exit_code);
}
