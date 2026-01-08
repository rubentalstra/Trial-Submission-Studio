//! XPT column (variable) definitions.
//!
//! Represents the metadata for a single variable in an XPT dataset,
//! corresponding to the NAMESTR record in the file format.

use crate::header::{normalize_name, truncate_str};
use std::fmt;

/// Variable data type in XPT format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum XptType {
    /// Numeric variable (1 in NAMESTR ntype field).
    ///
    /// Stored as IBM mainframe floating-point, typically 8 bytes.
    #[default]
    Num = 1,

    /// Character variable (2 in NAMESTR ntype field).
    ///
    /// Stored as fixed-width ASCII/EBCDIC text.
    Char = 2,
}

impl XptType {
    /// Create from NAMESTR ntype value.
    #[must_use]
    pub const fn from_ntype(ntype: i16) -> Option<Self> {
        match ntype {
            1 => Some(Self::Num),
            2 => Some(Self::Char),
            _ => None,
        }
    }

    /// Get the NAMESTR ntype value.
    #[must_use]
    pub const fn to_ntype(self) -> i16 {
        match self {
            Self::Num => 1,
            Self::Char => 2,
        }
    }

    /// Check if this is a numeric type.
    #[must_use]
    pub const fn is_numeric(self) -> bool {
        matches!(self, Self::Num)
    }

    /// Check if this is a character type.
    #[must_use]
    pub const fn is_character(self) -> bool {
        matches!(self, Self::Char)
    }
}

impl fmt::Display for XptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num => write!(f, "Num"),
            Self::Char => write!(f, "Char"),
        }
    }
}

/// Text justification for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Justification {
    /// Left-justified (0 in NAMESTR nfj field).
    #[default]
    Left = 0,

    /// Right-justified (1 in NAMESTR nfj field).
    Right = 1,
}

impl Justification {
    /// Create from NAMESTR nfj value.
    #[must_use]
    pub const fn from_nfj(nfj: i16) -> Self {
        match nfj {
            1 => Self::Right,
            _ => Self::Left,
        }
    }

    /// Get the NAMESTR nfj value.
    #[must_use]
    pub const fn to_nfj(self) -> i16 {
        self as i16
    }
}

/// Column (variable) definition in an XPT dataset.
///
/// Corresponds to a NAMESTR record in the XPT file format.
/// Contains all metadata for a single variable including name, type,
/// length, label, format, and informat specifications.
#[derive(Debug, Clone, PartialEq)]
pub struct XptColumn {
    /// Variable name (1-8 uppercase ASCII characters).
    pub name: String,

    /// Variable label (max 40 characters).
    pub label: Option<String>,

    /// Data type (numeric or character).
    pub data_type: XptType,

    /// Length in bytes within an observation.
    ///
    /// For numeric: typically 8 (can be 3-8).
    /// For character: 1-200 (extended format allows more).
    pub length: u16,

    // Format specification (output)
    /// Format name (max 8 characters, e.g., "DATE9", "BEST12").
    pub format: Option<String>,

    /// Format field width.
    pub format_length: u16,

    /// Format decimal places.
    pub format_decimals: u16,

    // Informat specification (input)
    /// Informat name (max 8 characters).
    pub informat: Option<String>,

    /// Informat field width.
    pub informat_length: u16,

    /// Informat decimal places.
    pub informat_decimals: u16,

    /// Text justification for display.
    pub justification: Justification,
}

impl XptColumn {
    /// Create a new numeric column with default settings.
    ///
    /// # Arguments
    /// * `name` - Variable name (1-8 characters, will be uppercased)
    ///
    /// # Returns
    /// A new `XptColumn` with numeric type and 8-byte length.
    #[must_use]
    pub fn numeric(name: impl Into<String>) -> Self {
        Self {
            name: normalize_name(&name.into()),
            label: None,
            data_type: XptType::Num,
            length: 8,
            format: None,
            format_length: 0,
            format_decimals: 0,
            informat: None,
            informat_length: 0,
            informat_decimals: 0,
            justification: Justification::Right,
        }
    }

    /// Create a new character column.
    ///
    /// # Arguments
    /// * `name` - Variable name (1-8 characters, will be uppercased)
    /// * `length` - Character length (1-200)
    ///
    /// # Returns
    /// A new `XptColumn` with character type and specified length.
    #[must_use]
    pub fn character(name: impl Into<String>, length: u16) -> Self {
        Self {
            name: normalize_name(&name.into()),
            label: None,
            data_type: XptType::Char,
            length,
            format: None,
            format_length: 0,
            format_decimals: 0,
            informat: None,
            informat_length: 0,
            informat_decimals: 0,
            justification: Justification::Left,
        }
    }

    /// Set the variable label.
    ///
    /// Note: Labels are NOT truncated here. Validation against version-specific
    /// limits (40 chars for V5, 256 chars for V8) happens at write time.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        let label_str = label.into();
        self.label = if label_str.is_empty() {
            None
        } else {
            Some(label_str)
        };
        self
    }

    /// Set the output format.
    ///
    /// # Arguments
    /// * `name` - Format name (e.g., "DATE9", "BEST12")
    /// * `length` - Field width
    /// * `decimals` - Decimal places
    #[must_use]
    pub fn with_format(mut self, name: impl Into<String>, length: u16, decimals: u16) -> Self {
        let name_str = name.into();
        self.format = if name_str.is_empty() {
            None
        } else {
            Some(truncate_str(&name_str.to_uppercase(), 8))
        };
        self.format_length = length;
        self.format_decimals = decimals;
        self
    }

    /// Set the input format (informat).
    ///
    /// # Arguments
    /// * `name` - Informat name
    /// * `length` - Field width
    /// * `decimals` - Decimal places
    #[must_use]
    pub fn with_informat(mut self, name: impl Into<String>, length: u16, decimals: u16) -> Self {
        let name_str = name.into();
        self.informat = if name_str.is_empty() {
            None
        } else {
            Some(truncate_str(&name_str.to_uppercase(), 8))
        };
        self.informat_length = length;
        self.informat_decimals = decimals;
        self
    }

    /// Set the justification.
    #[must_use]
    pub fn with_justification(mut self, justification: Justification) -> Self {
        self.justification = justification;
        self
    }

    /// Set the length in bytes.
    #[must_use]
    pub fn with_length(mut self, length: u16) -> Self {
        self.length = length;
        self
    }

    /// Check if this is a numeric column.
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        self.data_type.is_numeric()
    }

    /// Check if this is a character column.
    #[must_use]
    pub const fn is_character(&self) -> bool {
        self.data_type.is_character()
    }

    /// Get the effective label (label or name if no label).
    #[must_use]
    pub fn effective_label(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.name)
    }
}

impl Default for XptColumn {
    fn default() -> Self {
        Self::numeric("VAR")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xpt_type() {
        assert_eq!(XptType::from_ntype(1), Some(XptType::Num));
        assert_eq!(XptType::from_ntype(2), Some(XptType::Char));
        assert_eq!(XptType::from_ntype(0), None);
        assert_eq!(XptType::from_ntype(3), None);

        assert_eq!(XptType::Num.to_ntype(), 1);
        assert_eq!(XptType::Char.to_ntype(), 2);

        assert!(XptType::Num.is_numeric());
        assert!(!XptType::Num.is_character());
        assert!(!XptType::Char.is_numeric());
        assert!(XptType::Char.is_character());
    }

    #[test]
    fn test_justification() {
        assert_eq!(Justification::from_nfj(0), Justification::Left);
        assert_eq!(Justification::from_nfj(1), Justification::Right);
        assert_eq!(Justification::from_nfj(99), Justification::Left);

        assert_eq!(Justification::Left.to_nfj(), 0);
        assert_eq!(Justification::Right.to_nfj(), 1);
    }

    #[test]
    fn test_numeric_column() {
        let col = XptColumn::numeric("age");
        assert_eq!(col.name, "AGE");
        assert_eq!(col.data_type, XptType::Num);
        assert_eq!(col.length, 8);
        assert!(col.is_numeric());
        assert!(!col.is_character());
    }

    #[test]
    fn test_character_column() {
        let col = XptColumn::character("name", 20);
        assert_eq!(col.name, "NAME");
        assert_eq!(col.data_type, XptType::Char);
        assert_eq!(col.length, 20);
        assert!(!col.is_numeric());
        assert!(col.is_character());
    }

    #[test]
    fn test_with_label() {
        let col = XptColumn::numeric("age").with_label("Age in Years");
        assert_eq!(col.label, Some("Age in Years".to_string()));

        let col = XptColumn::numeric("age").with_label("");
        assert_eq!(col.label, None);
    }

    #[test]
    fn test_with_format() {
        let col = XptColumn::numeric("date").with_format("DATE9", 9, 0);
        assert_eq!(col.format, Some("DATE9".to_string()));
        assert_eq!(col.format_length, 9);
        assert_eq!(col.format_decimals, 0);
    }

    #[test]
    fn test_name_normalization() {
        // Trimming and uppercasing
        let col = XptColumn::numeric("  test  ");
        assert_eq!(col.name, "TEST");

        // Long names are NOT truncated (validation catches limit violations)
        // This supports V8 format which allows up to 32 characters
        let col = XptColumn::numeric("verylongname");
        assert_eq!(col.name, "VERYLONGNAME");
    }

    #[test]
    fn test_label_not_truncated() {
        // Labels are no longer truncated in with_label; validation handles limits
        let long_label = "A".repeat(100);
        let col = XptColumn::numeric("x").with_label(&long_label);
        assert_eq!(col.label.as_ref().map(String::len), Some(100));
        assert_eq!(col.label, Some(long_label));
    }

    #[test]
    fn test_effective_label() {
        let col = XptColumn::numeric("age");
        assert_eq!(col.effective_label(), "AGE");

        let col = XptColumn::numeric("age").with_label("Age in Years");
        assert_eq!(col.effective_label(), "Age in Years");
    }
}
