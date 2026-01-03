//! Reader and writer options.

use chrono::NaiveDateTime;

use super::MissingValue;
use crate::{
    XptVersion,
    header::{format_xpt_datetime, truncate_str},
};

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
