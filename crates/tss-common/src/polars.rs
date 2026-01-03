//! Polars AnyValue utility functions.
//!
//! This module provides helper functions for working with Polars `AnyValue` types,
//! including string conversions and numeric parsing.

use polars::prelude::*;

/// Converts a Polars `AnyValue` to a `String` representation.
///
/// Returns an empty string for `Null`, properly formats numeric types without
/// unnecessary trailing zeros.
///
/// # Examples
///
/// ```
/// use polars::prelude::AnyValue;
/// use tss_common::any_to_string;
///
/// assert_eq!(any_to_string(AnyValue::Null), "");
/// assert_eq!(any_to_string(AnyValue::Int32(42)), "42");
/// assert_eq!(any_to_string(AnyValue::String("hello")), "hello");
/// ```
pub fn any_to_string(value: AnyValue<'_>) -> String {
    match value {
        AnyValue::Null => String::new(),
        AnyValue::Int8(v) => v.to_string(),
        AnyValue::Int16(v) => v.to_string(),
        AnyValue::Int32(v) => v.to_string(),
        AnyValue::Int64(v) => v.to_string(),
        AnyValue::UInt8(v) => v.to_string(),
        AnyValue::UInt16(v) => v.to_string(),
        AnyValue::UInt32(v) => v.to_string(),
        AnyValue::UInt64(v) => v.to_string(),
        AnyValue::Float32(v) => format_numeric(f64::from(v)),
        AnyValue::Float64(v) => format_numeric(v),
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        AnyValue::Boolean(b) => if b { "Y" } else { "N" }.to_string(),
        // For any other type, use Display but strip outer quotes if present
        other => {
            let s = other.to_string();
            // Strip surrounding quotes that might come from formatting
            if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                s[1..s.len() - 1].to_string()
            } else {
                s
            }
        }
    }
}

/// Converts `AnyValue` to `String`, returning `None` if the result is empty.
///
/// Useful for XML/output generation where empty values should be omitted.
pub fn any_to_string_non_empty(value: AnyValue<'_>) -> Option<String> {
    let s = any_to_string(value);
    if s.trim().is_empty() { None } else { Some(s) }
}

/// Formats a floating-point number as a string without trailing zeros after decimal.
///
/// Only trims trailing zeros if the number contains a decimal point.
/// Integer-valued floats like 40.0 are formatted as "40", not "4".
///
/// # Examples
///
/// ```
/// use tss_common::format_numeric;
///
/// assert_eq!(format_numeric(1.0), "1");
/// assert_eq!(format_numeric(1.5), "1.5");
/// assert_eq!(format_numeric(1.50), "1.5");
/// assert_eq!(format_numeric(0.0), "0");
/// assert_eq!(format_numeric(40.0), "40");
/// assert_eq!(format_numeric(100.0), "100");
/// ```
pub fn format_numeric(v: f64) -> String {
    let s = format!("{v}");
    // Only trim trailing zeros if there's a decimal point
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        // No decimal point - return as-is (integer representation)
        s
    }
}

/// Converts an `AnyValue` to `f64`, returning `None` for non-numeric or null values.
///
/// Handles integer types, floating-point types, and string parsing.
pub fn any_to_f64(value: AnyValue<'_>) -> Option<f64> {
    match value {
        AnyValue::Null => None,
        AnyValue::Int8(v) => Some(f64::from(v)),
        AnyValue::Int16(v) => Some(f64::from(v)),
        AnyValue::Int32(v) => Some(f64::from(v)),
        AnyValue::Int64(v) => Some(v as f64),
        AnyValue::UInt8(v) => Some(f64::from(v)),
        AnyValue::UInt16(v) => Some(f64::from(v)),
        AnyValue::UInt32(v) => Some(f64::from(v)),
        AnyValue::UInt64(v) => Some(v as f64),
        AnyValue::Float32(v) => Some(f64::from(v)),
        AnyValue::Float64(v) => Some(v),
        AnyValue::String(s) => parse_f64(s),
        AnyValue::StringOwned(s) => parse_f64(&s),
        _ => None,
    }
}

/// Converts an `AnyValue` to `i64`, returning `None` for non-integer or null values.
///
/// Handles integer types, floating-point types (truncated), and string parsing.
pub fn any_to_i64(value: AnyValue<'_>) -> Option<i64> {
    match value {
        AnyValue::Null => None,
        AnyValue::Int8(v) => Some(i64::from(v)),
        AnyValue::Int16(v) => Some(i64::from(v)),
        AnyValue::Int32(v) => Some(i64::from(v)),
        AnyValue::Int64(v) => Some(v),
        AnyValue::UInt8(v) => Some(i64::from(v)),
        AnyValue::UInt16(v) => Some(i64::from(v)),
        AnyValue::UInt32(v) => Some(i64::from(v)),
        AnyValue::UInt64(v) => i64::try_from(v).ok(),
        AnyValue::Float32(v) => Some(v as i64),
        AnyValue::Float64(v) => Some(v as i64),
        AnyValue::String(s) => parse_i64(s),
        AnyValue::StringOwned(s) => parse_i64(&s),
        _ => None,
    }
}

/// Parses a string as `f64`, returning `None` for invalid or empty strings.
pub fn parse_f64(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<f64>().ok()
}

/// Parses a string as `i64`, returning `None` for invalid or empty strings.
pub fn parse_i64(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<i64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_any_to_string_null() {
        assert_eq!(any_to_string(AnyValue::Null), "");
    }

    #[test]
    fn test_any_to_string_integers() {
        assert_eq!(any_to_string(AnyValue::Int32(42)), "42");
        assert_eq!(any_to_string(AnyValue::Int64(-100)), "-100");
        assert_eq!(any_to_string(AnyValue::UInt32(0)), "0");
    }

    #[test]
    fn test_any_to_string_floats() {
        assert_eq!(any_to_string(AnyValue::Float64(1.5)), "1.5");
        assert_eq!(any_to_string(AnyValue::Float64(1.0)), "1");
        assert_eq!(any_to_string(AnyValue::Float64(1.50)), "1.5");
    }

    #[test]
    fn test_any_to_string_strings() {
        assert_eq!(any_to_string(AnyValue::String("hello")), "hello");
    }

    #[test]
    fn test_any_to_string_boolean() {
        assert_eq!(any_to_string(AnyValue::Boolean(true)), "Y");
        assert_eq!(any_to_string(AnyValue::Boolean(false)), "N");
    }

    #[test]
    fn test_any_to_string_non_empty() {
        assert_eq!(any_to_string_non_empty(AnyValue::Null), None);
        assert_eq!(any_to_string_non_empty(AnyValue::String("")), None);
        assert_eq!(any_to_string_non_empty(AnyValue::String("  ")), None);
        assert_eq!(
            any_to_string_non_empty(AnyValue::String("hello")),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_format_numeric() {
        assert_eq!(format_numeric(1.0), "1");
        assert_eq!(format_numeric(1.5), "1.5");
        assert_eq!(format_numeric(1.50), "1.5");
        assert_eq!(format_numeric(0.0), "0");
        // Ensure trailing zeros in integer part are NOT trimmed
        assert_eq!(format_numeric(40.0), "40");
        assert_eq!(format_numeric(100.0), "100");
        assert_eq!(format_numeric(1000.0), "1000");
        assert_eq!(format_numeric(10.5), "10.5");
        assert_eq!(format_numeric(40.50), "40.5");
    }

    #[test]
    fn test_any_to_f64() {
        assert_eq!(any_to_f64(AnyValue::Null), None);
        assert_eq!(any_to_f64(AnyValue::Int32(42)), Some(42.0));
        assert_eq!(any_to_f64(AnyValue::Float64(3.14)), Some(3.14));
        assert_eq!(any_to_f64(AnyValue::String("2.5")), Some(2.5));
        assert_eq!(any_to_f64(AnyValue::String("invalid")), None);
    }

    #[test]
    fn test_any_to_i64() {
        assert_eq!(any_to_i64(AnyValue::Null), None);
        assert_eq!(any_to_i64(AnyValue::Int32(42)), Some(42));
        assert_eq!(any_to_i64(AnyValue::Float64(3.9)), Some(3)); // truncated
        assert_eq!(any_to_i64(AnyValue::String("100")), Some(100));
        assert_eq!(any_to_i64(AnyValue::String("invalid")), None);
    }

    #[test]
    fn test_parse_f64() {
        assert_eq!(parse_f64(""), None);
        assert_eq!(parse_f64("  "), None);
        assert_eq!(parse_f64("3.14"), Some(3.14));
        assert_eq!(parse_f64("  3.14  "), Some(3.14));
        assert_eq!(parse_f64("invalid"), None);
    }

    #[test]
    fn test_parse_i64() {
        assert_eq!(parse_i64(""), None);
        assert_eq!(parse_i64("  "), None);
        assert_eq!(parse_i64("42"), Some(42));
        assert_eq!(parse_i64("  -100  "), Some(-100));
        assert_eq!(parse_i64("invalid"), None);
    }
}
