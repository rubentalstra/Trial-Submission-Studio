//! Numeric normalization utilities.

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

/// Formats a floating-point number as a string without trailing zeros.
pub fn format_numeric(v: f64) -> String {
    let s = format!("{v}");
    // Strip unnecessary trailing zeros while keeping at least one decimal place
    // But keep e.g. 10.0 as 10? Or 10.0?
    // The original implementation was:
    // s.trim_end_matches('0').trim_end_matches('.').to_string()
    // This converts "10.0" -> "10.", -> "10"
    // "10.50" -> "10.5"
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
