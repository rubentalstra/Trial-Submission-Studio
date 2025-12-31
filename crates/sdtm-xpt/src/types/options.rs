//! Reader and writer options.

use chrono::NaiveDateTime;

use super::MissingValue;

/// SAS Transport format version.
///
/// Determines the header prefixes and field length limits used when
/// reading and writing XPT files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XptVersion {
    /// Version 5/6 format (default, maximum compatibility).
    ///
    /// - Dataset names: max 8 characters
    /// - Variable names: max 8 characters
    /// - Labels: max 40 characters
    /// - Format names: max 8 characters
    #[default]
    V5,

    /// Version 8/9 format (extended names and labels).
    ///
    /// - Dataset names: max 32 characters
    /// - Variable names: max 32 characters
    /// - Labels: max 256 characters
    /// - Format names: max 32 characters
    V8,
}

impl XptVersion {
    // --- Limits (parameterized by version) ---

    /// Maximum length for variable names.
    #[must_use]
    pub const fn name_limit(&self) -> usize {
        match self {
            Self::V5 => 8,
            Self::V8 => 32,
        }
    }

    /// Maximum length for variable labels.
    #[must_use]
    pub const fn label_limit(&self) -> usize {
        match self {
            Self::V5 => 40,
            Self::V8 => 256,
        }
    }

    /// Maximum length for format names.
    #[must_use]
    pub const fn format_limit(&self) -> usize {
        match self {
            Self::V5 => 8,
            Self::V8 => 32,
        }
    }

    /// Maximum length for dataset names.
    #[must_use]
    pub const fn dataset_name_limit(&self) -> usize {
        match self {
            Self::V5 => 8,
            Self::V8 => 32,
        }
    }

    // --- Header prefixes (parameterized by version) ---

    /// Library header prefix.
    #[must_use]
    pub const fn library_prefix(&self) -> &'static str {
        match self {
            Self::V5 => "HEADER RECORD*******LIBRARY HEADER RECORD!!!!!!!",
            Self::V8 => "HEADER RECORD*******LIBV8   HEADER RECORD!!!!!!!",
        }
    }

    /// Member header prefix.
    #[must_use]
    pub const fn member_prefix(&self) -> &'static str {
        match self {
            Self::V5 => "HEADER RECORD*******MEMBER  HEADER RECORD!!!!!!!",
            Self::V8 => "HEADER RECORD*******MEMBV8  HEADER RECORD!!!!!!!",
        }
    }

    /// Descriptor header prefix.
    #[must_use]
    pub const fn dscrptr_prefix(&self) -> &'static str {
        match self {
            Self::V5 => "HEADER RECORD*******DSCRPTR HEADER RECORD!!!!!!!",
            Self::V8 => "HEADER RECORD*******DSCPTV8 HEADER RECORD!!!!!!!",
        }
    }

    /// NAMESTR header prefix.
    #[must_use]
    pub const fn namestr_prefix(&self) -> &'static str {
        match self {
            Self::V5 => "HEADER RECORD*******NAMESTR HEADER RECORD!!!!!!!",
            Self::V8 => "HEADER RECORD*******NAMSTV8 HEADER RECORD!!!!!!!",
        }
    }

    /// OBS header prefix.
    #[must_use]
    pub const fn obs_prefix(&self) -> &'static str {
        match self {
            Self::V5 => "HEADER RECORD*******OBS     HEADER RECORD!!!!!!!",
            Self::V8 => "HEADER RECORD*******OBSV8   HEADER RECORD!!!!!!!",
        }
    }

    /// LABELV8 header prefix (V8 only).
    #[must_use]
    pub const fn labelv8_prefix(&self) -> &'static str {
        "HEADER RECORD*******LABELV8 HEADER RECORD!!!!!!!"
    }

    /// LABELV9 header prefix (V8 only).
    #[must_use]
    pub const fn labelv9_prefix(&self) -> &'static str {
        "HEADER RECORD*******LABELV9 HEADER RECORD!!!!!!!"
    }

    // --- Feature checks ---

    /// Whether this version supports long names (> 8 characters).
    #[must_use]
    pub const fn supports_long_names(&self) -> bool {
        matches!(self, Self::V8)
    }

    /// Whether this version supports LABELV8/V9 sections.
    #[must_use]
    pub const fn supports_label_section(&self) -> bool {
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
    /// XPT format version to write.
    ///
    /// Default: V5 (maximum compatibility)
    pub version: XptVersion,

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
    /// Create new writer options with defaults.
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

    /// Use V8 format (extended names and labels).
    #[must_use]
    pub fn v8(mut self) -> Self {
        self.version = XptVersion::V8;
        self
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

    #[test]
    fn test_xpt_version_default() {
        assert_eq!(XptVersion::default(), XptVersion::V5);
    }

    #[test]
    fn test_xpt_version_limits() {
        // V5 limits
        assert_eq!(XptVersion::V5.name_limit(), 8);
        assert_eq!(XptVersion::V5.label_limit(), 40);
        assert_eq!(XptVersion::V5.format_limit(), 8);
        assert_eq!(XptVersion::V5.dataset_name_limit(), 8);

        // V8 limits
        assert_eq!(XptVersion::V8.name_limit(), 32);
        assert_eq!(XptVersion::V8.label_limit(), 256);
        assert_eq!(XptVersion::V8.format_limit(), 32);
        assert_eq!(XptVersion::V8.dataset_name_limit(), 32);
    }

    #[test]
    fn test_xpt_version_prefixes() {
        // V5 prefixes
        assert!(XptVersion::V5.library_prefix().contains("LIBRARY"));
        assert!(XptVersion::V5.member_prefix().contains("MEMBER"));
        assert!(XptVersion::V5.namestr_prefix().contains("NAMESTR"));

        // V8 prefixes
        assert!(XptVersion::V8.library_prefix().contains("LIBV8"));
        assert!(XptVersion::V8.member_prefix().contains("MEMBV8"));
        assert!(XptVersion::V8.namestr_prefix().contains("NAMSTV8"));
    }

    #[test]
    fn test_xpt_version_features() {
        assert!(!XptVersion::V5.supports_long_names());
        assert!(!XptVersion::V5.supports_label_section());

        assert!(XptVersion::V8.supports_long_names());
        assert!(XptVersion::V8.supports_label_section());
    }

    #[test]
    fn test_writer_options_version() {
        let opts = XptWriterOptions::default();
        assert_eq!(opts.version, XptVersion::V5);

        let opts = XptWriterOptions::new().v8();
        assert_eq!(opts.version, XptVersion::V8);

        let opts = XptWriterOptions::new().with_version(XptVersion::V8);
        assert_eq!(opts.version, XptVersion::V8);
    }

    #[test]
    fn test_xpt_version_display() {
        assert_eq!(format!("{}", XptVersion::V5), "V5");
        assert_eq!(format!("{}", XptVersion::V8), "V8");
    }
}
