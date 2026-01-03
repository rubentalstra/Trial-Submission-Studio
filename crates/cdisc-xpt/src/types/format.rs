//! Format and informat specifications.
//!
//! SAS formats control how values are displayed (output formats) and
//! read (input formats/informats).

use std::fmt;

/// Output format specification.
///
/// Formats control how values are displayed in output.
/// Common formats include:
/// - `BEST12.` - Best numeric format
/// - `DATE9.` - Date format (e.g., "01JAN2020")
/// - `$CHAR20.` - Character format
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormatSpec {
    /// Format name (e.g., "DATE9", "BEST12").
    ///
    /// Max 8 characters for V5, 32 for V8.
    pub name: Option<String>,

    /// Field width (total display width).
    pub width: u16,

    /// Number of decimal places (for numeric formats).
    pub decimals: u16,
}

impl FormatSpec {
    /// Create a new empty format spec.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            width: 0,
            decimals: 0,
        }
    }

    /// Create a format spec with name and width.
    #[must_use]
    pub fn with_name(name: impl Into<String>, width: u16) -> Self {
        Self {
            name: Some(name.into()),
            width,
            decimals: 0,
        }
    }

    /// Create a format spec with name, width, and decimals.
    #[must_use]
    pub fn with_decimals(name: impl Into<String>, width: u16, decimals: u16) -> Self {
        Self {
            name: Some(name.into()),
            width,
            decimals,
        }
    }

    /// Check if the format is empty (no name specified).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
    }

    /// Get the format name if present.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Create common date format (DATE9.).
    #[must_use]
    pub fn date9() -> Self {
        Self::with_name("DATE", 9)
    }

    /// Create common datetime format (DATETIME16.).
    #[must_use]
    pub fn datetime() -> Self {
        Self::with_name("DATETIME", 16)
    }

    /// Create best numeric format.
    #[must_use]
    pub fn best(width: u16) -> Self {
        Self::with_name("BEST", width)
    }

    /// Create character format.
    #[must_use]
    pub fn char(width: u16) -> Self {
        Self::with_name("$CHAR", width)
    }
}

impl fmt::Display for FormatSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) if self.decimals > 0 => {
                write!(f, "{}{}.{}", name, self.width, self.decimals)
            }
            Some(name) if self.width > 0 => {
                write!(f, "{}{}.", name, self.width)
            }
            Some(name) => write!(f, "{}.", name),
            None => Ok(()),
        }
    }
}

/// Input format (informat) specification.
///
/// Informats control how raw data is read and converted.
/// They mirror format specifications but are used for input.
pub type InformatSpec = FormatSpec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_spec_new() {
        let fmt = FormatSpec::new();
        assert!(fmt.is_empty());
        assert!(fmt.name.is_none());
    }

    #[test]
    fn test_format_spec_with_name() {
        let fmt = FormatSpec::with_name("DATE9", 9);
        assert!(!fmt.is_empty());
        assert_eq!(fmt.name(), Some("DATE9"));
        assert_eq!(fmt.width, 9);
        assert_eq!(fmt.decimals, 0);
    }

    #[test]
    fn test_format_spec_with_decimals() {
        let fmt = FormatSpec::with_decimals("BEST", 12, 2);
        assert_eq!(fmt.name(), Some("BEST"));
        assert_eq!(fmt.width, 12);
        assert_eq!(fmt.decimals, 2);
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", FormatSpec::new()), "");
        assert_eq!(format!("{}", FormatSpec::date9()), "DATE9.");
        assert_eq!(format!("{}", FormatSpec::with_name("BEST", 12)), "BEST12.");
        assert_eq!(
            format!("{}", FormatSpec::with_decimals("BEST", 12, 2)),
            "BEST12.2"
        );
    }

    #[test]
    fn test_common_formats() {
        let fmt = FormatSpec::date9();
        assert_eq!(fmt.name(), Some("DATE"));
        assert_eq!(fmt.width, 9);

        let fmt = FormatSpec::datetime();
        assert_eq!(fmt.name(), Some("DATETIME"));

        let fmt = FormatSpec::best(12);
        assert_eq!(fmt.name(), Some("BEST"));
        assert_eq!(fmt.width, 12);

        let fmt = FormatSpec::char(20);
        assert_eq!(fmt.name(), Some("$CHAR"));
        assert_eq!(fmt.width, 20);
    }
}
