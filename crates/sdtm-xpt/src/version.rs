//! XPT version definitions and version-specific limits.
//!
//! The SAS Transport format has two main versions:
//! - **V5**: Original format (1989), required by FDA for regulatory submissions
//! - **V8**: Extended format (2012), supports longer names and labels
//!
//! This module defines version-specific constants and provides methods
//! for querying format limits.

use std::fmt;

/// XPT file format version.
///
/// The version determines various format constraints such as maximum
/// name lengths, label lengths, and header structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum XptVersion {
    /// SAS Transport Version 5 (original format).
    ///
    /// This is the FDA-required format for regulatory submissions.
    /// Constraints:
    /// - Variable names: max 8 characters
    /// - Variable labels: max 40 characters
    /// - Format/informat names: max 8 characters
    /// - Dataset names: max 8 characters
    /// - Dataset labels: max 40 characters
    #[default]
    V5,

    /// SAS Transport Version 8 (extended format).
    ///
    /// Introduced in 2012 with extended capabilities.
    /// Constraints:
    /// - Variable names: max 32 characters
    /// - Variable labels: max 256 characters
    /// - Format/informat names: max 32 characters
    /// - Dataset names: max 32 characters
    /// - Dataset labels: max 256 characters
    ///
    /// Note: V8 is not accepted for FDA submissions.
    V8,
}

impl XptVersion {
    // ============ Name Limits ============

    /// Maximum length for variable/column names.
    #[must_use]
    pub const fn variable_name_limit(&self) -> usize {
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

    /// Maximum length for format names.
    #[must_use]
    pub const fn format_name_limit(&self) -> usize {
        match self {
            Self::V5 => 8,
            Self::V8 => 32,
        }
    }

    /// Maximum length for informat names.
    #[must_use]
    pub const fn informat_name_limit(&self) -> usize {
        self.format_name_limit()
    }

    // ============ Label Limits ============

    /// Maximum length for variable/column labels.
    #[must_use]
    pub const fn variable_label_limit(&self) -> usize {
        match self {
            Self::V5 => 40,
            Self::V8 => 256,
        }
    }

    /// Maximum length for dataset labels.
    #[must_use]
    pub const fn dataset_label_limit(&self) -> usize {
        match self {
            Self::V5 => 40,
            Self::V8 => 256,
        }
    }

    // ============ NAMESTR Record ============

    /// Standard NAMESTR record length in bytes.
    ///
    /// V5 uses 140-byte NAMESTR records (or 136 for VAX/VMS).
    /// V8 uses the same base structure but with extended fields.
    #[must_use]
    pub const fn namestr_length(&self) -> usize {
        140
    }

    /// VAX/VMS NAMESTR record length (historical compatibility).
    #[must_use]
    pub const fn namestr_length_vax(&self) -> usize {
        136
    }

    // ============ Header Prefixes ============

    /// Library header prefix string.
    #[must_use]
    pub const fn library_header_prefix(&self) -> &'static str {
        match self {
            Self::V5 => "HEADER RECORD*******LIBRARY HEADER RECORD!!!!!!!",
            Self::V8 => "HEADER RECORD*******LIBV8 HEADER RECORD!!!!!!!000",
        }
    }

    /// Member header prefix string.
    #[must_use]
    pub const fn member_header_prefix(&self) -> &'static str {
        match self {
            Self::V5 => "HEADER RECORD*******MEMBER  HEADER RECORD!!!!!!!",
            Self::V8 => "HEADER RECORD*******MEMBV8 HEADER RECORD!!!!!!!000",
        }
    }

    /// Descriptor header prefix string.
    #[must_use]
    pub const fn dscrptr_header_prefix(&self) -> &'static str {
        match self {
            Self::V5 => "HEADER RECORD*******DSCRPTR HEADER RECORD!!!!!!!",
            Self::V8 => "HEADER RECORD*******DSCPTV8 HEADER RECORD!!!!!!!000",
        }
    }

    /// NAMESTR header prefix string.
    #[must_use]
    pub const fn namestr_header_prefix(&self) -> &'static str {
        match self {
            Self::V5 => "HEADER RECORD*******NAMESTR HEADER RECORD!!!!!!!",
            Self::V8 => "HEADER RECORD*******NAMSTV8 HEADER RECORD!!!!!!!",
        }
    }

    /// Observation header prefix string.
    #[must_use]
    pub const fn obs_header_prefix(&self) -> &'static str {
        match self {
            Self::V5 => "HEADER RECORD*******OBS     HEADER RECORD!!!!!!!",
            Self::V8 => "HEADER RECORD*******OBSV8   HEADER RECORD!!!!!!!",
        }
    }

    // ============ Feature Support ============

    /// Whether this version supports LABELV8/V9 sections for extended labels.
    #[must_use]
    pub const fn supports_label_section(&self) -> bool {
        matches!(self, Self::V8)
    }

    /// Whether this version is FDA-compliant for regulatory submissions.
    #[must_use]
    pub const fn is_fda_compliant(&self) -> bool {
        matches!(self, Self::V5)
    }

    // ============ Detection ============

    /// Detect version from library header prefix.
    ///
    /// Returns `None` if the header doesn't match either version.
    #[must_use]
    pub fn from_library_header(header: &[u8]) -> Option<Self> {
        if header.len() < 48 {
            return None;
        }

        let prefix = &header[..48];
        let prefix_str = std::str::from_utf8(prefix).ok()?;

        if prefix_str.starts_with("HEADER RECORD*******LIBV8") {
            Some(Self::V8)
        } else if prefix_str.starts_with("HEADER RECORD*******LIBRARY") {
            Some(Self::V5)
        } else {
            None
        }
    }

    /// Get all supported versions.
    #[must_use]
    pub const fn all() -> [Self; 2] {
        [Self::V5, Self::V8]
    }
}

impl fmt::Display for XptVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V5 => write!(f, "V5"),
            Self::V8 => write!(f, "V8"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_v5() {
        assert_eq!(XptVersion::default(), XptVersion::V5);
    }

    #[test]
    fn test_v5_limits() {
        let v = XptVersion::V5;
        assert_eq!(v.variable_name_limit(), 8);
        assert_eq!(v.variable_label_limit(), 40);
        assert_eq!(v.dataset_name_limit(), 8);
        assert_eq!(v.format_name_limit(), 8);
    }

    #[test]
    fn test_v8_limits() {
        let v = XptVersion::V8;
        assert_eq!(v.variable_name_limit(), 32);
        assert_eq!(v.variable_label_limit(), 256);
        assert_eq!(v.dataset_name_limit(), 32);
        assert_eq!(v.format_name_limit(), 32);
    }

    #[test]
    fn test_fda_compliance() {
        assert!(XptVersion::V5.is_fda_compliant());
        assert!(!XptVersion::V8.is_fda_compliant());
    }

    #[test]
    fn test_label_section_support() {
        assert!(!XptVersion::V5.supports_label_section());
        assert!(XptVersion::V8.supports_label_section());
    }

    #[test]
    fn test_version_detection_v5() {
        let header = b"HEADER RECORD*******LIBRARY HEADER RECORD!!!!!!!000000000000000000000000000000  ";
        assert_eq!(XptVersion::from_library_header(header), Some(XptVersion::V5));
    }

    #[test]
    fn test_version_detection_v8() {
        let header = b"HEADER RECORD*******LIBV8 HEADER RECORD!!!!!!!000000000000000000000000000000  ";
        assert_eq!(XptVersion::from_library_header(header), Some(XptVersion::V8));
    }

    #[test]
    fn test_version_detection_invalid() {
        let header = b"INVALID HEADER";
        assert_eq!(XptVersion::from_library_header(header), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", XptVersion::V5), "V5");
        assert_eq!(format!("{}", XptVersion::V8), "V8");
    }
}
