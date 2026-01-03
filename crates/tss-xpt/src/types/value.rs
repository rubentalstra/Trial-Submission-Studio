//! XPT data values.
//!
//! Represents individual cell values in an XPT dataset.

use std::fmt;

use super::MissingValue;

/// A numeric value that can be a number or missing.
#[derive(Debug, Clone, PartialEq)]
pub enum NumericValue {
    /// A valid floating-point number.
    Value(f64),

    /// A missing value (one of 28 possible missing codes).
    Missing(MissingValue),
}

impl NumericValue {
    /// Create a new numeric value.
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self::Value(value)
    }

    /// Create a standard missing value.
    #[must_use]
    pub const fn missing() -> Self {
        Self::Missing(MissingValue::Standard)
    }

    /// Create a special missing value.
    #[must_use]
    pub const fn missing_special(letter: char) -> Self {
        Self::Missing(MissingValue::Special(letter))
    }

    /// Check if this is a missing value.
    #[must_use]
    pub const fn is_missing(&self) -> bool {
        matches!(self, Self::Missing(_))
    }

    /// Check if this is a valid (non-missing) value.
    #[must_use]
    pub const fn is_present(&self) -> bool {
        matches!(self, Self::Value(_))
    }

    /// Get the numeric value if present.
    #[must_use]
    pub const fn value(&self) -> Option<f64> {
        match self {
            Self::Value(v) => Some(*v),
            Self::Missing(_) => None,
        }
    }

    /// Get the missing value type if missing.
    #[must_use]
    pub const fn missing_type(&self) -> Option<MissingValue> {
        match self {
            Self::Missing(m) => Some(*m),
            Self::Value(_) => None,
        }
    }

    /// Convert to Option<f64>, treating missing as None.
    #[must_use]
    pub const fn as_option(&self) -> Option<f64> {
        self.value()
    }
}

impl Default for NumericValue {
    fn default() -> Self {
        Self::missing()
    }
}

impl From<f64> for NumericValue {
    fn from(value: f64) -> Self {
        Self::Value(value)
    }
}

impl From<Option<f64>> for NumericValue {
    fn from(value: Option<f64>) -> Self {
        match value {
            Some(v) => Self::Value(v),
            None => Self::Missing(MissingValue::Standard),
        }
    }
}

impl From<MissingValue> for NumericValue {
    fn from(missing: MissingValue) -> Self {
        Self::Missing(missing)
    }
}

impl fmt::Display for NumericValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(v) => write!(f, "{v}"),
            Self::Missing(m) => write!(f, "{m}"),
        }
    }
}

/// A value in an XPT dataset cell.
///
/// Can be either a numeric value (possibly missing) or a character string.
#[derive(Debug, Clone, PartialEq)]
pub enum XptValue {
    /// Numeric value (float or missing).
    Num(NumericValue),

    /// Character string value.
    Char(String),
}

impl XptValue {
    /// Create a numeric value from a float.
    #[must_use]
    pub fn numeric(value: f64) -> Self {
        Self::Num(NumericValue::Value(value))
    }

    /// Create a missing numeric value.
    #[must_use]
    pub fn numeric_missing() -> Self {
        Self::Num(NumericValue::Missing(MissingValue::Standard))
    }

    /// Create a missing numeric value with specific type.
    #[must_use]
    pub fn numeric_missing_with(missing: MissingValue) -> Self {
        Self::Num(NumericValue::Missing(missing))
    }

    /// Create a character value.
    #[must_use]
    pub fn character(value: impl Into<String>) -> Self {
        Self::Char(value.into())
    }

    /// Check if this is a numeric value.
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        matches!(self, Self::Num(_))
    }

    /// Check if this is a character value.
    #[must_use]
    pub const fn is_character(&self) -> bool {
        matches!(self, Self::Char(_))
    }

    /// Check if this is a missing numeric value.
    #[must_use]
    pub fn is_missing(&self) -> bool {
        match self {
            Self::Num(n) => n.is_missing(),
            Self::Char(_) => false,
        }
    }

    /// Get the numeric value if this is a non-missing number.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Num(n) => n.value(),
            Self::Char(_) => None,
        }
    }

    /// Get the character value if this is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Char(s) => Some(s),
            Self::Num(_) => None,
        }
    }

    /// Get the numeric value reference.
    #[must_use]
    pub const fn as_numeric(&self) -> Option<&NumericValue> {
        match self {
            Self::Num(n) => Some(n),
            Self::Char(_) => None,
        }
    }

    /// Convert numeric to Option<f64> for convenience.
    ///
    /// Returns None for missing values and character values.
    #[must_use]
    pub fn to_f64(&self) -> Option<f64> {
        self.as_f64()
    }

    /// Convert to string representation.
    ///
    /// For numerics, returns the formatted number or missing indicator.
    /// For characters, returns the string value.
    #[must_use]
    pub fn to_string_value(&self) -> String {
        match self {
            Self::Num(n) => n.to_string(),
            Self::Char(s) => s.clone(),
        }
    }
}

impl Default for XptValue {
    fn default() -> Self {
        Self::Num(NumericValue::default())
    }
}

impl From<f64> for XptValue {
    fn from(value: f64) -> Self {
        Self::Num(NumericValue::Value(value))
    }
}

impl From<Option<f64>> for XptValue {
    fn from(value: Option<f64>) -> Self {
        Self::Num(NumericValue::from(value))
    }
}

impl From<String> for XptValue {
    fn from(value: String) -> Self {
        Self::Char(value)
    }
}

impl From<&str> for XptValue {
    fn from(value: &str) -> Self {
        Self::Char(value.to_string())
    }
}

impl From<NumericValue> for XptValue {
    fn from(value: NumericValue) -> Self {
        Self::Num(value)
    }
}

impl fmt::Display for XptValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{n}"),
            Self::Char(s) => write!(f, "{s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_value() {
        let v = NumericValue::new(42.0);
        assert!(v.is_present());
        assert!(!v.is_missing());
        assert_eq!(v.value(), Some(42.0));
        assert_eq!(v.missing_type(), None);
    }

    #[test]
    fn test_numeric_missing() {
        let v = NumericValue::missing();
        assert!(!v.is_present());
        assert!(v.is_missing());
        assert_eq!(v.value(), None);
        assert_eq!(v.missing_type(), Some(MissingValue::Standard));
    }

    #[test]
    fn test_numeric_special_missing() {
        let v = NumericValue::missing_special('A');
        assert!(v.is_missing());
        assert_eq!(v.missing_type(), Some(MissingValue::Special('A')));
        assert_eq!(format!("{v}"), ".A");
    }

    #[test]
    fn test_numeric_from_option() {
        let v: NumericValue = Some(1.5).into();
        assert_eq!(v.value(), Some(1.5));

        let v: NumericValue = None.into();
        assert!(v.is_missing());
    }

    #[test]
    fn test_xpt_value_numeric() {
        let v = XptValue::numeric(3.14);
        assert!(v.is_numeric());
        assert!(!v.is_character());
        assert!(!v.is_missing());
        assert_eq!(v.as_f64(), Some(3.14));
        assert_eq!(v.as_str(), None);
    }

    #[test]
    fn test_xpt_value_missing() {
        let v = XptValue::numeric_missing();
        assert!(v.is_numeric());
        assert!(v.is_missing());
        assert_eq!(v.as_f64(), None);
    }

    #[test]
    fn test_xpt_value_character() {
        let v = XptValue::character("hello");
        assert!(!v.is_numeric());
        assert!(v.is_character());
        assert!(!v.is_missing());
        assert_eq!(v.as_str(), Some("hello"));
        assert_eq!(v.as_f64(), None);
    }

    #[test]
    fn test_xpt_value_from() {
        let v: XptValue = 42.0.into();
        assert!(v.is_numeric());

        let v: XptValue = "test".into();
        assert!(v.is_character());

        let v: XptValue = String::from("test").into();
        assert!(v.is_character());

        let v: XptValue = Some(1.0).into();
        assert!(!v.is_missing());

        let v: XptValue = None.into();
        assert!(v.is_missing());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", NumericValue::new(1.5)), "1.5");
        assert_eq!(format!("{}", NumericValue::missing()), ".");
        assert_eq!(format!("{}", XptValue::numeric(2.5)), "2.5");
        assert_eq!(format!("{}", XptValue::character("test")), "test");
    }

    #[test]
    fn test_to_string_value() {
        assert_eq!(XptValue::numeric(1.0).to_string_value(), "1");
        assert_eq!(XptValue::numeric_missing().to_string_value(), ".");
        assert_eq!(XptValue::character("test").to_string_value(), "test");
    }
}
