//! CDISC Transpiler CLI.

use clap::{ColorChoice, Parser};
use sdtm_cli::logging::{LogConfig, LogFormat, init_logging};
use std::io::{self, IsTerminal};
use tracing::level_filters::LevelFilter;

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
    cli.color.write_global();
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
    let mut config = LogConfig {
        level_filter: cli.verbosity.tracing_level_filter(),
        ..LogConfig::default()
    };
    config.use_env_filter = !(cli.verbosity.is_present() || cli.log_level.is_some());
    if let Some(level) = cli.log_level {
        config.level_filter = match level {
            LogLevelArg::Error => LevelFilter::ERROR,
            LogLevelArg::Warn => LevelFilter::WARN,
            LogLevelArg::Info => LevelFilter::INFO,
            LogLevelArg::Debug => LevelFilter::DEBUG,
            LogLevelArg::Trace => LevelFilter::TRACE,
        };
    }
    config.format = match cli.log_format {
        LogFormatArg::Pretty => LogFormat::Pretty,
        LogFormatArg::Compact => LogFormat::Compact,
        LogFormatArg::Json => LogFormat::Json,
    };
    config.log_file = cli.log_file.clone();
    config.with_ansi = match cli.color.color {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => cli.log_file.is_none() && io::stderr().is_terminal(),
    };
    config
}
