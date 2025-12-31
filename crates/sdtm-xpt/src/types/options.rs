//! Reader and writer options.

use chrono::NaiveDateTime;

use super::MissingValue;

/// Options for reading XPT files.
#[derive(Debug, Clone)]
pub struct XptReaderOptions {
    /// Whether to validate the file structure strictly.
    ///
    /// When false, the reader will attempt to recover from minor format issues.
    pub strict: bool,

    /// Whether to trim trailing spaces from character values.
    pub trim_strings: bool,
}

impl Default for XptReaderOptions {
    fn default() -> Self {
        Self {
            strict: false,
            trim_strings: true,
        }
    }
}

impl XptReaderOptions {
    /// Create new reader options with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable strict validation mode.
    #[must_use]
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Disable string trimming.
    #[must_use]
    pub fn no_trim(mut self) -> Self {
        self.trim_strings = false;
        self
    }
}

/// Options for writing XPT files.
#[derive(Debug, Clone)]
pub struct XptWriterOptions {
    /// SAS version string (max 8 characters).
    ///
    /// Default: "9.4"
    pub sas_version: String,

    /// Operating system name (max 8 characters).
    ///
    /// Default: "RUST"
    pub os_name: String,

    /// Datetime when file was created.
    ///
    /// If None, uses current time.
    pub created: Option<NaiveDateTime>,

    /// Datetime when file was modified.
    ///
    /// If None, uses created time.
    pub modified: Option<NaiveDateTime>,

    /// Default missing value type for numeric null values.
    ///
    /// Default: Standard (.)
    pub default_missing: MissingValue,

    /// NAMESTR record length.
    ///
    /// Default: 140 (standard), use 136 for VAX/VMS compatibility.
    pub namestr_length: usize,
}

impl Default for XptWriterOptions {
    fn default() -> Self {
        Self {
            sas_version: "9.4".to_string(),
            os_name: "RUST".to_string(),
            created: None,
            modified: None,
            default_missing: MissingValue::Standard,
            namestr_length: 140,
        }
    }
}

impl XptWriterOptions {
    /// Create new writer options with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the SAS version string.
    #[must_use]
    pub fn with_sas_version(mut self, version: impl Into<String>) -> Self {
        self.sas_version = truncate_string(version.into(), 8);
        self
    }

    /// Set the operating system name.
    #[must_use]
    pub fn with_os_name(mut self, os: impl Into<String>) -> Self {
        self.os_name = truncate_string(os.into(), 8);
        self
    }

    /// Set the created datetime.
    #[must_use]
    pub fn with_created(mut self, datetime: NaiveDateTime) -> Self {
        self.created = Some(datetime);
        self
    }

    /// Set the modified datetime.
    #[must_use]
    pub fn with_modified(mut self, datetime: NaiveDateTime) -> Self {
        self.modified = Some(datetime);
        self
    }

    /// Set the default missing value type.
    #[must_use]
    pub fn with_default_missing(mut self, missing: MissingValue) -> Self {
        self.default_missing = missing;
        self
    }

    /// Use VAX/VMS compatible NAMESTR length (136 bytes).
    #[must_use]
    pub fn vax_compatible(mut self) -> Self {
        self.namestr_length = 136;
        self
    }

    /// Get the created datetime, using current time if not set.
    #[must_use]
    pub fn get_created(&self) -> NaiveDateTime {
        self.created
            .unwrap_or_else(|| chrono::Local::now().naive_local())
    }

    /// Get the modified datetime, using created time if not set.
    #[must_use]
    pub fn get_modified(&self) -> NaiveDateTime {
        self.modified.unwrap_or_else(|| self.get_created())
    }

    /// Get the formatted created datetime string.
    #[must_use]
    pub fn format_created(&self) -> String {
        format_xpt_datetime(self.get_created())
    }

    /// Get the formatted modified datetime string.
    #[must_use]
    pub fn format_modified(&self) -> String {
        format_xpt_datetime(self.get_modified())
    }
}

/// Truncate a string to maximum length.
fn truncate_string(s: String, max_len: usize) -> String {
    if s.len() <= max_len {
        s
    } else {
        s.chars().take(max_len).collect()
    }
}

/// Format a datetime as SAS format: ddMMMyy:hh:mm:ss
fn format_xpt_datetime(dt: NaiveDateTime) -> String {
    dt.format("%d%b%y:%H:%M:%S").to_string().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_reader_options_default() {
        let opts = XptReaderOptions::default();
        assert!(!opts.strict);
        assert!(opts.trim_strings);
    }

    #[test]
    fn test_reader_options_builder() {
        let opts = XptReaderOptions::new().strict().no_trim();
        assert!(opts.strict);
        assert!(!opts.trim_strings);
    }

    #[test]
    fn test_writer_options_default() {
        let opts = XptWriterOptions::default();
        assert_eq!(opts.sas_version, "9.4");
        assert_eq!(opts.os_name, "RUST");
        assert_eq!(opts.default_missing, MissingValue::Standard);
        assert_eq!(opts.namestr_length, 140);
    }

    #[test]
    fn test_writer_options_builder() {
        let dt = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap();

        let opts = XptWriterOptions::new()
            .with_sas_version("9.3")
            .with_os_name("LINUX")
            .with_created(dt)
            .with_default_missing(MissingValue::Special('A'))
            .vax_compatible();

        assert_eq!(opts.sas_version, "9.3");
        assert_eq!(opts.os_name, "LINUX");
        assert_eq!(opts.created, Some(dt));
        assert_eq!(opts.default_missing, MissingValue::Special('A'));
        assert_eq!(opts.namestr_length, 136);
    }

    #[test]
    fn test_format_datetime() {
        let dt = NaiveDate::from_ymd_opt(2024, 3, 15)
            .unwrap()
            .and_hms_opt(14, 30, 45)
            .unwrap();

        let formatted = format_xpt_datetime(dt);
        assert_eq!(formatted, "15MAR24:14:30:45");
    }

    #[test]
    fn test_version_truncation() {
        let opts = XptWriterOptions::new()
            .with_sas_version("verylongversion")
            .with_os_name("verylongosname");

        assert_eq!(opts.sas_version.len(), 8);
        assert_eq!(opts.os_name.len(), 8);
    }
}
