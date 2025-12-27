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
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing::level_filters::LevelFilter;
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
    EnvFilter,
    fmt::{
        self, FmtContext, MakeWriter,
        format::{FormatEvent, FormatFields, Writer},
        time::{FormatTime, SystemTime},
    },
    layer::SubscriberExt,
    layer::{Context, Layer},
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
                .with(FieldCaptureLayer)
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
                    .with(FieldCaptureLayer)
                    .with(layer)
                    .init();
            } else {
                tracing_subscriber::registry()
                    .with(filter)
                    .with(FieldCaptureLayer)
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
                .with(FieldCaptureLayer)
                .with(layer)
                .init();
        }
    }
}

/// Captures span fields so formatted logs can reuse structured context.
#[derive(Debug, Default)]
struct SpanFields {
    fields: BTreeMap<String, String>,
}

impl Visit for SpanFields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std_fmt::Debug) {
        self.fields
            .insert(field.name().to_string(), format!("{value:?}"));
    }
}

/// Stores span fields in extensions for later lookup by the formatter.
struct FieldCaptureLayer;

impl<S> Layer<S> for FieldCaptureLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut fields = SpanFields::default();
            attrs.record(&mut fields);
            span.extensions_mut().insert(fields);
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut extensions = span.extensions_mut();
            if let Some(fields) = extensions.get_mut::<SpanFields>() {
                values.record(fields);
            } else {
                let mut fields = SpanFields::default();
                values.record(&mut fields);
                extensions.insert(fields);
            }
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
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std_fmt::Result {
        if self.with_timestamps {
            self.timer.format_time(&mut writer)?;
            write!(writer, " ")?;
        }

        let level = event.metadata().level();
        let has_ansi = writer.has_ansi_escapes();

        let mut visitor = HumanFieldVisitor::default();
        let stage = enrich_from_spans(ctx, &mut visitor);
        event.record(&mut visitor);

        let message = visitor
            .take_message()
            .unwrap_or_else(|| event.metadata().name().to_string());

        // Check if this is a summary/milestone message
        let is_milestone = is_milestone_message(&message);

        // Write the level indicator (icon or text)
        write_level_indicator(&mut writer, level, has_ansi, is_milestone)?;
        write!(writer, " ")?;

        // Format context (domain/dataset)
        let context = format_context(&mut visitor);
        if !context.is_empty() {
            write_context(&mut writer, &context, has_ansi)?;
            write!(writer, " ")?;
        }

        // Format and write message
        let message = normalize_message(&message, stage);
        write_message(&mut writer, &message, level, has_ansi)?;

        // Format details
        let details = format_details(&mut visitor);
        if !details.is_empty() {
            write_details_section(&mut writer, &details, has_ansi)?;
        }

        writeln!(writer)
    }
}

/// Check if this is a milestone/summary message that deserves special formatting.
fn is_milestone_message(message: &str) -> bool {
    matches!(
        message,
        "ingest complete"
            | "domain processing complete"
            | "validation complete"
            | "output complete"
    )
}

/// Write level indicator with standard text labels.
fn write_level_indicator(
    writer: &mut Writer<'_>,
    level: &Level,
    has_ansi: bool,
    _is_milestone: bool,
) -> std_fmt::Result {
    let label = match *level {
        Level::ERROR => "ERROR",
        Level::WARN => "WARN ",
        Level::INFO => "INFO ",
        Level::DEBUG => "DEBUG",
        Level::TRACE => "TRACE",
    };

    if has_ansi {
        let color = match *level {
            Level::ERROR => "\x1b[1;31m", // Bold red
            Level::WARN => "\x1b[33m",    // Yellow
            Level::INFO => "\x1b[32m",    // Green
            Level::DEBUG => "\x1b[34m",   // Blue
            Level::TRACE => "\x1b[90m",   // Dim gray
        };
        write!(writer, "{color}{label}\x1b[0m")
    } else {
        write!(writer, "{label}")
    }
}

/// Write context (domain/dataset) with subtle styling.
fn write_context(writer: &mut Writer<'_>, context: &str, has_ansi: bool) -> std_fmt::Result {
    if has_ansi {
        write!(writer, "\x1b[1;36m{context}\x1b[0m") // Bold cyan
    } else {
        write!(writer, "{context}")
    }
}

/// Write message with appropriate styling based on level.
fn write_message(
    writer: &mut Writer<'_>,
    message: &str,
    level: &Level,
    has_ansi: bool,
) -> std_fmt::Result {
    if has_ansi && *level == Level::ERROR {
        write!(writer, "\x1b[1;31m{message}\x1b[0m") // Bold red for errors
    } else if has_ansi && *level == Level::WARN {
        write!(writer, "\x1b[33m{message}\x1b[0m") // Yellow for warnings
    } else {
        write!(writer, "{message}")
    }
}

/// Write details section with clean formatting.
fn write_details_section(
    writer: &mut Writer<'_>,
    details: &[String],
    has_ansi: bool,
) -> std_fmt::Result {
    if has_ansi {
        write!(writer, " \x1b[90m(")?; // Dim gray parenthesis
    } else {
        write!(writer, " (")?;
    }

    for (idx, detail) in details.iter().enumerate() {
        if idx > 0 {
            if has_ansi {
                write!(writer, "\x1b[90m, \x1b[0m")?;
            } else {
                write!(writer, ", ")?;
            }
        }
        write_detail_item(writer, detail, has_ansi)?;
    }

    if has_ansi {
        write!(writer, "\x1b[90m)\x1b[0m")
    } else {
        write!(writer, ")")
    }
}

/// Write a single detail item with styling.
fn write_detail_item(writer: &mut Writer<'_>, detail: &str, has_ansi: bool) -> std_fmt::Result {
    if let Some((label, value)) = detail.split_once('=') {
        if has_ansi {
            let value_color = value_color_for_label(label);
            write!(writer, "\x1b[90m{label}=\x1b[0m")?; // Dim label
            if let Some(color) = value_color {
                write!(writer, "{color}{value}\x1b[0m")
            } else {
                write!(writer, "{value}")
            }
        } else {
            write!(writer, "{label}={value}")
        }
    } else {
        write!(writer, "{detail}")
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

    /// Merge fields into the current visitor, allowing newer values to override.
    fn extend_fields(&mut self, fields: &BTreeMap<String, String>) {
        for (key, value) in fields {
            self.fields.insert(key.clone(), value.clone());
        }
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

/// Format core context (domain and dataset) into a compact prefix.
fn format_context(fields: &mut HumanFieldVisitor) -> String {
    let domain = fields
        .take_field("domain_code")
        .or_else(|| fields.take_field("domain"));
    let dataset = fields.take_field("dataset_name");

    match (domain, dataset) {
        (Some(domain), Some(dataset)) if dataset != domain => {
            format!("[{domain}/{dataset}]")
        }
        (Some(domain), _) => format!("[{domain}]"),
        (None, Some(dataset)) => format!("[{dataset}]"),
        (None, None) => String::new(),
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
    // Consume source_file from fields but don't display it per-line (shown at start of file processing)
    let _ = fields.take_field("source_file");
    // source_filename is the short filename shown in "Processing" messages
    let source_filename = fields.take_field("source_filename");
    let duration_ms = fields.take_field("duration_ms");
    let sequence = fields.take_field("sequence");

    // Source filename first - for "Processing X.csv" messages
    if let Some(filename) = source_filename {
        details.push(filename);
    }

    // Row counts - most important metric
    if let (Some(input_rows), Some(output_rows)) = (input_rows.as_ref(), output_rows.as_ref()) {
        if input_rows == output_rows {
            details.push(format!("{input_rows} rows"));
        } else {
            details.push(format!("{input_rows}→{output_rows} rows"));
        }
    } else if let Some(output_rows) = output_rows.as_ref() {
        details.push(format!("{output_rows} rows"));
    } else if let Some(record_count) = record_count.as_ref() {
        details.push(format!("{record_count} rows"));
    }

    // Counts
    if let Some(domain_count) = domain_count {
        details.push(format!("{domain_count} domains"));
    }
    if let Some(file_count) = file_count {
        details.push(format!("{file_count} files"));
    }

    // Validation results
    if let Some(error_count) = error_count {
        let count: u32 = error_count.parse().unwrap_or(0);
        if count > 0 {
            details.push(format!("{error_count} errors"));
        }
    }
    if let Some(warning_count) = warning_count {
        let count: u32 = warning_count.parse().unwrap_or(0);
        if count > 0 {
            details.push(format!("{warning_count} warnings"));
        }
    }

    // Output counts
    let mut outputs = Vec::new();
    if let Some(xpt_count) = xpt_count {
        outputs.push(format!("{xpt_count} XPT"));
    }
    if let Some(dataset_xml_count) = dataset_xml_count {
        outputs.push(format!("{dataset_xml_count} XML"));
    }
    if let Some(sas_count) = sas_count {
        outputs.push(format!("{sas_count} SAS"));
    }
    if !outputs.is_empty() {
        details.push(outputs.join(", "));
    }

    // Define-XML path
    if let Some(define_xml) = define_xml {
        details.push(format_path_tail(&define_xml, 2));
    }

    // Duration - always last
    if let Some(duration_ms) = duration_ms {
        details.push(format_duration(&duration_ms));
    }

    // Sequence info for warnings
    if let Some(sequence) = sequence {
        details.push(sequence);
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

/// Pick the nearest stage name from span scope, if available.
fn enrich_from_spans<S, N>(
    ctx: &FmtContext<'_, S, N>,
    visitor: &mut HumanFieldVisitor,
) -> Option<&'static str>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let mut stage = None;
    if let Some(scope) = ctx.event_scope() {
        for span in scope.from_root() {
            if let Some(fields) = span.extensions().get::<SpanFields>() {
                visitor.extend_fields(&fields.fields);
            }
            if let Some(stage_name) = stage_from_span(span.metadata().name()) {
                stage = Some(stage_name);
            }
        }
    }
    stage
}

/// Map span names to human-friendly stage labels.
fn stage_from_span(name: &str) -> Option<&'static str> {
    match name {
        "ingest" => Some("ingest"),
        "map" => Some("map"),
        "preprocess" => Some("preprocess"),
        "domain_rules" => Some("rules"),
        "suppqual" => Some("suppqual"),
        "validate" => Some("validate"),
        "output" => Some("output"),
        _ => None,
    }
}

/// Normalize known messages for cleaner, more readable output.
fn normalize_message(message: &str, stage: Option<&str>) -> String {
    // Clean up verbose internal messages
    match message {
        // Stage completion messages - make them clean summaries
        "ingest complete" => "Loaded study data".to_string(),
        "domain processing complete" => "Processed all domains".to_string(),
        "validation complete" => "Validation finished".to_string(),
        "output complete" => "Generated output files".to_string(),

        // Processing messages
        "mapping complete" => "Mapped columns".to_string(),
        "preprocess complete" => "Preprocessed data".to_string(),
        "domain rules complete" => "Applied domain rules".to_string(),
        "suppqual complete" => "Generated SUPPQUAL".to_string(),
        "suppqual skipped" => "No SUPPQUAL needed".to_string(),
        "file processed" => "Processed file".to_string(),
        "validation summary" => "Validation summary".to_string(),
        "output prepared" => "Prepared output".to_string(),
        "processing domain" => "Processing".to_string(),
        "processing file" => "Processing".to_string(),

        // Common warning messages - simplify
        "USUBJID values updated with study prefix" => "Added STUDYID prefix to USUBJID".to_string(),
        "Sequence values recalculated with tracker" => "Recalculated sequence numbers".to_string(),

        // Default handling
        _ => {
            // Remove redundant stage prefix if present
            if let Some(stage) = stage {
                let prefix = format!("{stage}: ");
                if message.starts_with(&prefix) {
                    return message[prefix.len()..].to_string();
                }
            }
            message.to_string()
        }
    }
}

/// Render a shortened path tail for log readability.
fn format_path_tail(path: &str, segments: usize) -> String {
    if segments == 0 {
        return path.to_string();
    }
    let components: Vec<_> = Path::new(path)
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .map(str::to_string)
        .collect();
    if components.is_empty() {
        return path.to_string();
    }
    let start = components.len().saturating_sub(segments);
    let sep = std::path::MAIN_SEPARATOR.to_string();
    components[start..].join(&sep)
}

/// Pick ANSI colors for key detail values.
fn value_color_for_label(label: &str) -> Option<&'static str> {
    // Check if this looks like an error/warning count
    if label.contains("error") {
        return Some("\x1b[1;31m"); // Bold red
    }
    if label.contains("warning") {
        return Some("\x1b[33m"); // Yellow
    }
    // Numbers get cyan
    if label.contains("rows")
        || label.contains("domain")
        || label.contains("file")
        || label.contains("XPT")
        || label.contains("XML")
        || label.contains("SAS")
    {
        return Some("\x1b[36m"); // Cyan
    }
    None
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
