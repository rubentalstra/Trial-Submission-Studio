//! Numeric conversion per SDTM requirements.
//!
//! Per SDTMIG, Num variables are 8-byte IEEE floating point (Float64).
//! This module handles parsing various numeric formats.

/// Parse a string value to numeric (f64).
///
/// Handles common numeric formats:
/// - Standard numbers: "123", "-45.67"
/// - Thousands separators: "1,234,567"
/// - Whitespace: "  123  "
/// - Scientific notation: "1.23e5"
///
/// Returns None if the value cannot be parsed as a number.
pub fn parse_numeric(value: &str) -> Option<f64> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Remove thousands separators and whitespace
    let cleaned = trimmed
        .replace(',', "")
        .replace(' ', "")
        .replace('\u{a0}', ""); // Non-breaking space

    // Handle special cases
    if cleaned.eq_ignore_ascii_case("nan") {
        return Some(f64::NAN);
    }
    if cleaned.eq_ignore_ascii_case("inf") || cleaned.eq_ignore_ascii_case("infinity") {
        return Some(f64::INFINITY);
    }
    if cleaned.eq_ignore_ascii_case("-inf") || cleaned.eq_ignore_ascii_case("-infinity") {
        return Some(f64::NEG_INFINITY);
    }

    // Try standard parse
    cleaned.parse().ok()
}

/// Convert a string to numeric, returning the value or None if unparseable.
///
/// This is the main transformation function for numeric variables.
/// Preserves the original value in case of parse failure for error handling.
pub fn transform_to_numeric(value: &str) -> Result<f64, &str> {
    parse_numeric(value).ok_or(value)
}

/// Check if a string represents a valid numeric value.
pub fn is_numeric(value: &str) -> bool {
    parse_numeric(value).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_integer() {
        assert_eq!(parse_numeric("123"), Some(123.0));
        assert_eq!(parse_numeric("-456"), Some(-456.0));
    }

    #[test]
    fn test_decimal() {
        assert_eq!(parse_numeric("123.45"), Some(123.45));
        assert_eq!(parse_numeric("-0.5"), Some(-0.5));
    }

    #[test]
    fn test_thousands_separator() {
        assert_eq!(parse_numeric("1,234,567"), Some(1234567.0));
        assert_eq!(parse_numeric("1,234.56"), Some(1234.56));
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(parse_numeric("  123  "), Some(123.0));
        assert_eq!(parse_numeric("  -45.67  "), Some(-45.67));
    }

    #[test]
    fn test_scientific_notation() {
        assert_eq!(parse_numeric("1.23e5"), Some(123000.0));
        assert_eq!(parse_numeric("1.5E-3"), Some(0.0015));
    }

    #[test]
    fn test_empty() {
        assert_eq!(parse_numeric(""), None);
        assert_eq!(parse_numeric("  "), None);
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse_numeric("abc"), None);
        assert_eq!(parse_numeric("12.34.56"), None);
        assert_eq!(parse_numeric("1,23"), Some(123.0)); // Comma as thousands sep
    }

    #[test]
    fn test_special_values() {
        assert!(parse_numeric("nan").unwrap().is_nan());
        assert_eq!(parse_numeric("inf"), Some(f64::INFINITY));
        assert_eq!(parse_numeric("-inf"), Some(f64::NEG_INFINITY));
    }

    #[test]
    fn test_transform_success() {
        assert_eq!(transform_to_numeric("123.45"), Ok(123.45));
    }

    #[test]
    fn test_transform_failure() {
        assert_eq!(transform_to_numeric("not a number"), Err("not a number"));
    }

    #[test]
    fn test_is_numeric() {
        assert!(is_numeric("123"));
        assert!(is_numeric("45.67"));
        assert!(!is_numeric("abc"));
        assert!(!is_numeric(""));
    }
}
