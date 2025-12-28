//! Polars AnyValue utility functions.
//!
//! This module provides helper functions for working with Polars `AnyValue` types,
//! including string conversions and numeric parsing.

use polars::prelude::*;

/// Converts a Polars AnyValue to a String representation.
/// Returns empty string for Null, properly formats numeric types.
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
        other => other.to_string(),
    }
}

/// Converts AnyValue to String, returning None if the result is empty.
pub fn any_to_string_non_empty(value: AnyValue<'_>) -> Option<String> {
    let s = any_to_string(value);
    if s.trim().is_empty() { None } else { Some(s) }
}

/// Formats a floating-point number as a string without trailing zeros.
pub fn format_numeric(v: f64) -> String {
    let s = format!("{v}");
    // Strip unnecessary trailing zeros while keeping at least one decimal place
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Converts an AnyValue to f64, returning None for non-numeric or null values.
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

/// Converts an AnyValue to i64, returning None for non-integer or null values.
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

/// Parses a string as f64, returning None for invalid or empty strings.
pub fn parse_f64(value: &str) -> Option<f64> {
    if value.trim().is_empty() {
        return None;
    }
    value.trim().parse::<f64>().ok()
}

/// Parses a string as i64, returning None for invalid or empty strings.
pub fn parse_i64(value: &str) -> Option<i64> {
    if value.trim().is_empty() {
        return None;
    }
    value.trim().parse::<i64>().ok()
}
