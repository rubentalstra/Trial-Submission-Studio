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

use std::collections::BTreeMap;
use std::fmt as std_fmt;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing::level_filters::LevelFilter;
use tracing::{Event, Level};
use tracing_subscriber::{
    EnvFilter,
    fmt::{
        self, FmtContext, MakeWriter,
        format::{FormatEvent, FormatFields, Writer},
        time::{FormatTime, SystemTime},
    },
    layer::SubscriberExt,
    registry::LookupSpan,
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
    /// Log level filter (off, error, warn, info, debug, trace).
    pub level_filter: LevelFilter,
    /// Whether to allow RUST_LOG to override CLI verbosity.
    pub use_env_filter: bool,
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
            level_filter: LevelFilter::INFO,
            use_env_filter: true,
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
        let level_filter = match verbosity {
            0 => LevelFilter::INFO,
            1 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        };
        Self {
            level_filter,
            ..Default::default()
        }
    }

    /// Set log level directly.
    #[must_use]
    pub fn with_level_filter(mut self, level_filter: LevelFilter) -> Self {
        self.level_filter = level_filter;
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
    let filter = build_env_filter(config.level_filter, config.use_env_filter);

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
                .event_format(HumanFormatter::new(config.with_timestamps))
                .with_writer(writer)
                .with_ansi(config.with_ansi);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .init();
        }
    }
}

/// Formats tracing events into a concise, human-friendly log line.
#[derive(Debug)]
struct HumanFormatter {
    timer: SystemTime,
    with_timestamps: bool,
}

impl HumanFormatter {
    /// Create a formatter that optionally includes timestamps.
    fn new(with_timestamps: bool) -> Self {
        Self {
            timer: SystemTime,
            with_timestamps,
        }
    }
}

impl<S, N> FormatEvent<S, N> for HumanFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std_fmt::Result {
        if self.with_timestamps {
            self.timer.format_time(&mut writer)?;
            write!(writer, " ")?;
        }

        write_level(&mut writer, event.metadata().level())?;
        write!(writer, " ")?;

        let mut visitor = HumanFieldVisitor::default();
        event.record(&mut visitor);

        let message = visitor
            .take_message()
            .unwrap_or_else(|| event.metadata().name().to_string());
        let context = format_context(&mut visitor);
        if !context.is_empty() {
            write!(writer, "{context} ")?;
        }

        write!(writer, "{message}")?;

        let details = format_details(&mut visitor);
        if !details.is_empty() {
            write!(writer, " ({})", details.join(", "))?;
        }

        writeln!(writer)
    }
}

/// Collects tracing event fields for human-friendly formatting.
#[derive(Debug, Default)]
struct HumanFieldVisitor {
    fields: BTreeMap<String, String>,
    message: Option<String>,
}

impl HumanFieldVisitor {
    /// Store a field value, extracting the message when present.
    fn record_value(&mut self, field: &Field, value: String) {
        if field.name() == "message" {
            self.message = Some(value);
        } else {
            self.fields.insert(field.name().to_string(), value);
        }
    }

    /// Take the stored log message, if any.
    fn take_message(&mut self) -> Option<String> {
        self.message.take()
    }

    /// Remove a field by name, returning its value.
    fn take_field(&mut self, name: &str) -> Option<String> {
        self.fields.remove(name)
    }
}

impl Visit for HumanFieldVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.record_value(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record_value(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_value(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_value(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record_value(field, value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std_fmt::Debug) {
        self.record_value(field, format!("{value:?}"));
    }
}

/// Format core context (study/domain/dataset) into a compact prefix.
fn format_context(fields: &mut HumanFieldVisitor) -> String {
    let study_id = fields.take_field("study_id");
    let dataset_name = fields.take_field("dataset_name");
    let mut domain = fields
        .take_field("domain_code")
        .or_else(|| fields.take_field("domain"));

    if let Some(dataset_name) = dataset_name {
        match &domain {
            Some(domain_code) if dataset_name != *domain_code => {
                domain = Some(format!("{domain_code}/{dataset_name}"));
            }
            Some(_) => {}
            None => domain = Some(dataset_name),
        }
    }

    let mut parts = Vec::new();
    if let Some(study_id) = study_id {
        parts.push(study_id);
    }
    if let Some(domain) = domain {
        parts.push(domain);
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("[{}]", parts.join(" | "))
    }
}

/// Format detail fields into a readable suffix.
fn format_details(fields: &mut HumanFieldVisitor) -> Vec<String> {
    let mut details = Vec::new();

    let input_rows = fields.take_field("input_rows");
    let output_rows = fields.take_field("output_rows");
    let record_count = fields.take_field("record_count");
    let domain_count = fields.take_field("domain_count");
    let file_count = fields.take_field("file_count");
    let error_count = fields.take_field("error_count");
    let warning_count = fields.take_field("warning_count");
    let xpt_count = fields.take_field("xpt_count");
    let dataset_xml_count = fields.take_field("dataset_xml_count");
    let sas_count = fields.take_field("sas_count");
    let define_xml = fields.take_field("define_xml");
    let source_file = fields.take_field("source_file");
    let duration_ms = fields.take_field("duration_ms");

    if let (Some(input_rows), Some(output_rows)) = (input_rows.as_ref(), output_rows.as_ref()) {
        if input_rows == output_rows {
            details.push(format!("rows={input_rows}"));
        } else {
            details.push(format!("rows={input_rows}->{output_rows}"));
        }
    } else if let Some(output_rows) = output_rows.as_ref() {
        details.push(format!("rows={output_rows}"));
    } else if let Some(record_count) = record_count.as_ref() {
        details.push(format!("rows={record_count}"));
    }

    if let Some(domain_count) = domain_count {
        details.push(format!("domains={domain_count}"));
    }
    if let Some(file_count) = file_count {
        details.push(format!("files={file_count}"));
    }

    if let Some(error_count) = error_count {
        details.push(format!("errors={error_count}"));
    }
    if let Some(warning_count) = warning_count {
        details.push(format!("warnings={warning_count}"));
    }

    if let Some(xpt_count) = xpt_count {
        details.push(format!("xpt={xpt_count}"));
    }
    if let Some(dataset_xml_count) = dataset_xml_count {
        details.push(format!("xml={dataset_xml_count}"));
    }
    if let Some(sas_count) = sas_count {
        details.push(format!("sas={sas_count}"));
    }
    if let Some(define_xml) = define_xml {
        details.push(format!("define={define_xml}"));
    }

    if let Some(source_file) = source_file {
        details.push(format!("file={source_file}"));
    }
    if let Some(duration_ms) = duration_ms {
        details.push(format!("duration={}", format_duration(&duration_ms)));
    }

    details
}

/// Render durations in milliseconds as ms or s for readability.
fn format_duration(duration_ms: &str) -> String {
    let Ok(value) = duration_ms.parse::<u128>() else {
        return format!("{duration_ms}ms");
    };
    if value >= 1000 {
        let seconds = (value as f64) / 1000.0;
        format!("{seconds:.1}s")
    } else {
        format!("{value}ms")
    }
}

/// Write the log level with optional ANSI coloring.
fn write_level(writer: &mut Writer<'_>, level: &Level) -> std_fmt::Result {
    let label = format!("{level:<5}");
    if writer.has_ansi_escapes() {
        let color = match *level {
            Level::ERROR => "\x1b[31m",
            Level::WARN => "\x1b[33m",
            Level::INFO => "\x1b[32m",
            Level::DEBUG => "\x1b[34m",
            Level::TRACE => "\x1b[36m",
        };
        write!(writer, "{color}{label}\x1b[0m")
    } else {
        write!(writer, "{label}")
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

/// Build an `EnvFilter` from the given level, optionally honoring `RUST_LOG`.
fn build_env_filter(level_filter: LevelFilter, use_env_filter: bool) -> EnvFilter {
    // Allow RUST_LOG to override the configured level when enabled.
    let level_str = level_filter.to_string();
    let default_filter = || {
        // Set default filter for our crates at the specified level.
        // External crates stay at warn level to reduce noise.
        EnvFilter::new(format!(
            "{level},sdtm_cli={level},sdtm_core={level},sdtm_ingest={level},\
             sdtm_map={level},sdtm_model={level},sdtm_report={level},\
             sdtm_standards={level},sdtm_validate={level},sdtm_xpt={level}",
            level = level_str
        ))
    };

    if use_env_filter {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| default_filter())
    } else {
        default_filter()
    }
}
