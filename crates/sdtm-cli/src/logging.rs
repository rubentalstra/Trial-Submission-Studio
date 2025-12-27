//! Logging infrastructure using `tracing` and `tracing-subscriber`.
//!
//! This module provides structured logging for the SDTM transpiler CLI.
//! All logging is routed through `tracing` spans for consistent observability.
//!
//! # Log Levels
//!
//! - `error`: Validation failures, fatal errors
//! - `warn`: Warnings, non-fatal issues
//! - `info`: Pipeline stage progress, summary counts
//! - `debug`: Detailed processing information
//! - `trace`: Row-level data (requires explicit `--log-data` flag for PHI safety)
//!
//! # Usage
//!
//! ```ignore
//! use sdtm_cli::logging::{init_logging, LogConfig};
//!
//! let config = LogConfig::from_verbosity(2);
//! init_logging(&config).expect("init logging");
//! ```

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::Level;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

static LOG_DATA_ENABLED: AtomicBool = AtomicBool::new(false);

/// Placeholder used when row-level logging is disabled.
pub const REDACTED_VALUE: &str = "[REDACTED]";

/// Returns true if row-level logging is explicitly enabled.
pub fn log_data_enabled() -> bool {
    LOG_DATA_ENABLED.load(Ordering::Relaxed)
}

/// Returns the input value when PHI logging is enabled, otherwise a redacted token.
pub fn redact_value(value: &str) -> &str {
    if log_data_enabled() {
        value
    } else {
        REDACTED_VALUE
    }
}

/// Configuration for logging behavior.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level filter (error, warn, info, debug, trace).
    pub level: Level,
    /// Whether to include timestamps in log output.
    pub with_timestamps: bool,
    /// Whether to include target (module path) in log output.
    pub with_target: bool,
    /// Whether to include span information in log output.
    pub with_spans: bool,
    /// Whether to use ANSI colors in output.
    pub with_ansi: bool,
    /// Output format: "pretty", "compact", or "json".
    pub format: LogFormat,
    /// Optional log file path. When set, logs are written to the file.
    pub log_file: Option<PathBuf>,
    /// Whether row-level (PHI/PII) values may be logged.
    pub log_data: bool,
}

/// Log output format.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable pretty format with colors.
    #[default]
    Pretty,
    /// Compact single-line format.
    Compact,
    /// JSON format for machine parsing.
    Json,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            with_timestamps: false,
            with_target: false,
            with_spans: true,
            with_ansi: true,
            format: LogFormat::default(),
            log_file: None,
            log_data: false,
        }
    }
}

impl LogConfig {
    /// Create a `LogConfig` from CLI verbosity count.
    ///
    /// - 0 (no `-v`): info level
    /// - 1 (`-v`): debug level
    /// - 2+ (`-vv`): trace level
    #[must_use]
    pub fn from_verbosity(verbosity: u8) -> Self {
        let level = match verbosity {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        };
        Self {
            level,
            ..Default::default()
        }
    }

    /// Set log level directly.
    #[must_use]
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Enable or disable timestamps.
    #[must_use]
    pub fn with_timestamps(mut self, enable: bool) -> Self {
        self.with_timestamps = enable;
        self
    }

    /// Enable or disable target (module path) in output.
    #[must_use]
    pub fn with_target(mut self, enable: bool) -> Self {
        self.with_target = enable;
        self
    }

    /// Enable or disable ANSI colors.
    #[must_use]
    pub fn with_ansi(mut self, enable: bool) -> Self {
        self.with_ansi = enable;
        self
    }

    /// Set output format.
    #[must_use]
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the log file path (writes to stderr when None).
    #[must_use]
    pub fn with_log_file(mut self, path: Option<PathBuf>) -> Self {
        self.log_file = path;
        self
    }

    /// Enable or disable row-level logging of sensitive values.
    #[must_use]
    pub fn with_log_data(mut self, enable: bool) -> Self {
        self.log_data = enable;
        self
    }
}

/// Initialize the global tracing subscriber with the given configuration.
///
/// This should be called once at application startup.
///
/// # Errors
///
/// Returns an error if the log file cannot be opened.
///
/// # Panics
///
/// Panics if called more than once or if subscriber initialization fails.
pub fn init_logging(config: &LogConfig) -> io::Result<()> {
    if let Some(path) = &config.log_file {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        init_logging_with_writer(config, SharedFileWriter::new(file));
    } else {
        init_logging_with_writer(config, io::stderr);
    }
    Ok(())
}

/// Initialize logging with a custom writer (useful for testing).
pub fn init_logging_with_writer<W>(config: &LogConfig, writer: W)
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    LOG_DATA_ENABLED.store(config.log_data, Ordering::Release);
    let filter = build_env_filter(config.level);

    match config.format {
        LogFormat::Json => {
            let layer = fmt::layer()
                .json()
                .with_writer(writer)
                .with_target(config.with_target)
                .with_span_events(if config.with_spans {
                    fmt::format::FmtSpan::CLOSE
                } else {
                    fmt::format::FmtSpan::NONE
                });

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .init();
        }
        LogFormat::Compact => {
            let layer = fmt::layer()
                .compact()
                .with_writer(writer)
                .with_ansi(config.with_ansi)
                .with_target(config.with_target);

            if config.with_timestamps {
                tracing_subscriber::registry()
                    .with(filter)
                    .with(layer)
                    .init();
            } else {
                tracing_subscriber::registry()
                    .with(filter)
                    .with(layer.without_time())
                    .init();
            }
        }
        LogFormat::Pretty => {
            let layer = fmt::layer()
                .with_writer(writer)
                .with_ansi(config.with_ansi)
                .with_target(config.with_target);

            if config.with_timestamps {
                tracing_subscriber::registry()
                    .with(filter)
                    .with(layer)
                    .init();
            } else {
                tracing_subscriber::registry()
                    .with(filter)
                    .with(layer.without_time())
                    .init();
            }
        }
    }
}

#[derive(Clone)]
struct SharedFileWriter {
    file: Arc<Mutex<std::fs::File>>,
}

impl SharedFileWriter {
    fn new(file: std::fs::File) -> Self {
        Self {
            file: Arc::new(Mutex::new(file)),
        }
    }
}

struct SharedFileGuard {
    file: Arc<Mutex<std::fs::File>>,
}

impl Write for SharedFileGuard {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self
            .file
            .lock()
            .map_err(|_| io::Error::other("log file lock poisoned"))?;
        guard.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut guard = self
            .file
            .lock()
            .map_err(|_| io::Error::other("log file lock poisoned"))?;
        guard.flush()
    }
}

impl<'a> MakeWriter<'a> for SharedFileWriter {
    type Writer = SharedFileGuard;

    fn make_writer(&'a self) -> Self::Writer {
        SharedFileGuard {
            file: Arc::clone(&self.file),
        }
    }
}

/// Build an `EnvFilter` from the given level, respecting `RUST_LOG` env var.
fn build_env_filter(level: Level) -> EnvFilter {
    // Allow RUST_LOG to override the configured level
    let level_str = level.as_str().to_lowercase();

    EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Set default filter for our crates at the specified level
        // External crates stay at warn level to reduce noise
        EnvFilter::new(format!(
            "{level},sdtm_cli={level},sdtm_core={level},sdtm_ingest={level},\
             sdtm_map={level},sdtm_model={level},sdtm_report={level},\
             sdtm_standards={level},sdtm_validate={level},sdtm_xpt={level}",
            level = level_str
        ))
    })
}
