use clap::Parser;
use sdtm_cli::logging::{LogConfig, LogFormat, init_logging};
use tracing::Level;

mod cli;
mod commands;
mod pipeline;
mod summary;
mod types;

use crate::cli::{Cli, Command, LogFormatArg, LogLevelArg};
use crate::commands::{run_domains, run_study};
use crate::summary::print_summary;

fn main() {
    let cli = Cli::parse();
    let log_config = log_config_from_cli(&cli);
    if let Err(error) = init_logging(&log_config) {
        eprintln!("error: failed to initialize logging: {error}");
        std::process::exit(1);
    }
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

/// Build logging configuration from CLI flags with consistent precedence.
fn log_config_from_cli(cli: &Cli) -> LogConfig {
    let mut config = LogConfig::from_verbosity(cli.verbose);
    if cli.quiet {
        config.level = Level::ERROR;
    }
    if let Some(level) = cli.log_level {
        config.level = match level {
            LogLevelArg::Error => Level::ERROR,
            LogLevelArg::Warn => Level::WARN,
            LogLevelArg::Info => Level::INFO,
            LogLevelArg::Debug => Level::DEBUG,
            LogLevelArg::Trace => Level::TRACE,
        };
    }
    config.format = match cli.log_format {
        LogFormatArg::Pretty => LogFormat::Pretty,
        LogFormatArg::Compact => LogFormat::Compact,
        LogFormatArg::Json => LogFormat::Json,
    };
    config.log_file = cli.log_file.clone();
    config.log_data = cli.log_data;
    if config.log_file.is_some() {
        config.with_ansi = false;
    }
    config
}
