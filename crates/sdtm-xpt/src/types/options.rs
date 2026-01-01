//! Reader and writer options.

use chrono::NaiveDateTime;

use super::MissingValue;
use crate::header::truncate_str;

/// SAS Transport format version.
///
/// | Feature | V5 Limit | V8 Limit |
/// |---------|----------|----------|
/// | Variable name | 8 chars | 32 chars |
/// | Variable label | 40 chars | 256 chars |
/// | Format name | 8 chars | 32 chars |
/// | Dataset name | 8 chars | 32 chars |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XptVersion {
    /// V5/V6 format (default, maximum compatibility).
    #[default]
    V5,
    /// V8/V9 format (extended names and labels).
    V8,
}

impl XptVersion {
    /// Maximum length for variable names.
    #[must_use]
    pub const fn name_limit(self) -> usize {
        match self {
            Self::V5 => 8,
            Self::V8 => 32,
        }
    }

    /// Maximum length for variable labels.
    #[must_use]
    pub const fn label_limit(self) -> usize {
        match self {
            Self::V5 => 40,
            Self::V8 => 256,
        }
    }

    /// Maximum length for format names.
    #[must_use]
    pub const fn format_limit(self) -> usize {
        match self {
            Self::V5 => 8,
            Self::V8 => 32,
        }
    }

    /// Maximum length for dataset names.
    #[must_use]
    pub const fn dataset_name_limit(self) -> usize {
        match self {
            Self::V5 => 8,
            Self::V8 => 32,
        }
    }

    /// Whether this version supports long names (> 8 characters).
    #[must_use]
    pub const fn supports_long_names(self) -> bool {
        matches!(self, Self::V8)
    }

    /// Whether this version supports LABELV8/V9 sections.
    #[must_use]
    pub const fn supports_label_section(self) -> bool {
        matches!(self, Self::V8)
    }
}

impl std::fmt::Display for XptVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V5 => write!(f, "V5"),
            Self::V8 => write!(f, "V8"),
        }
    }
}

/// Options for reading XPT files.
#[derive(Debug, Clone)]
pub struct XptReaderOptions {
    /// Enable strict validation mode.
    pub strict: bool,
    /// Trim trailing spaces from character values (default: true).
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
    /// Create reader options with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable strict validation.
    #[must_use]
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }
}

/// Options for writing XPT files.
#[derive(Debug, Clone)]
pub struct XptWriterOptions {
    /// XPT format version (default: V5).
    pub version: XptVersion,
    /// SAS version string (max 8 chars, default: "9.4").
    pub sas_version: String,
    /// Operating system name (max 8 chars, default: "RUST").
    pub os_name: String,
    /// Created datetime (default: current time).
    pub created: Option<NaiveDateTime>,
    /// Modified datetime (default: created time).
    pub modified: Option<NaiveDateTime>,
    /// Default missing value for nulls (default: Standard ".").
    pub default_missing: MissingValue,
    /// NAMESTR length: 140 (standard) or 136 (VAX/VMS).
    pub namestr_length: usize,
}

impl Default for XptWriterOptions {
    fn default() -> Self {
        Self {
            version: XptVersion::V5,
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
    /// Create writer options with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the XPT format version.
    #[must_use]
    pub fn with_version(mut self, version: XptVersion) -> Self {
        self.version = version;
        self
    }

    /// Use V8 format.
    #[must_use]
    pub fn v8(mut self) -> Self {
        self.version = XptVersion::V8;
        self
    }

    /// Set the SAS version string.
    #[must_use]
    pub fn with_sas_version(mut self, version: impl Into<String>) -> Self {
        self.sas_version = truncate_str(&version.into(), 8);
        self
    }

    /// Set the operating system name.
    #[must_use]
    pub fn with_os_name(mut self, os: impl Into<String>) -> Self {
        self.os_name = truncate_str(&os.into(), 8);
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

    /// Get the created datetime (current time if not set).
    #[must_use]
    pub fn get_created(&self) -> NaiveDateTime {
        self.created
            .unwrap_or_else(|| chrono::Local::now().naive_local())
    }

    /// Get the modified datetime (created time if not set).
    #[must_use]
    pub fn get_modified(&self) -> NaiveDateTime {
        self.modified.unwrap_or_else(|| self.get_created())
    }

    /// Format created datetime for XPT header.
    #[must_use]
    pub fn format_created(&self) -> String {
        format_xpt_datetime(self.get_created())
    }

    /// Format modified datetime for XPT header.
    #[must_use]
    pub fn format_modified(&self) -> String {
        format_xpt_datetime(self.get_modified())
    }
}

/// Format datetime as SAS format: ddMMMyy:hh:mm:ss
fn format_xpt_datetime(dt: NaiveDateTime) -> String {
    dt.format("%d%b%y:%H:%M:%S").to_string().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_version_limits() {
        assert_eq!(XptVersion::V5.name_limit(), 8);
        assert_eq!(XptVersion::V5.label_limit(), 40);
        assert_eq!(XptVersion::V8.name_limit(), 32);
        assert_eq!(XptVersion::V8.label_limit(), 256);
    }

    #[test]
    fn test_writer_options() {
        let dt = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap();

        let opts = XptWriterOptions::new()
            .v8()
            .with_sas_version("9.3")
            .with_created(dt);

        assert_eq!(opts.version, XptVersion::V8);
        assert_eq!(opts.sas_version, "9.3");
        assert_eq!(opts.created, Some(dt));
    }

    #[test]
    fn test_format_datetime() {
        let dt = NaiveDate::from_ymd_opt(2024, 3, 15)
            .unwrap()
            .and_hms_opt(14, 30, 45)
            .unwrap();
        assert_eq!(format_xpt_datetime(dt), "15MAR24:14:30:45");
    }
}
