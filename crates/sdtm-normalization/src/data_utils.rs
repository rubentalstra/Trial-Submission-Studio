//! Data manipulation utilities for SDTM processing.
//!
//! Internal utilities for extracting and transforming DataFrame values,
//! handling source table metadata, and sanitizing test codes.

use polars::prelude::{AnyValue, DataFrame};
use sdtm_ingest::any_to_string;
use sdtm_model::MappingConfig;

/// Get a string value from a DataFrame column at the given row index.
pub fn column_value_string(df: &DataFrame, name: &str, idx: usize) -> String {
    match df.column(name) {
        Ok(series) => any_to_string(series.get(idx).unwrap_or(AnyValue::Null)),
        Err(_) => String::new(),
    }
}

/// Extract all trimmed string values from a DataFrame column.
pub fn column_trimmed_values(df: &DataFrame, name: &str) -> Option<Vec<String>> {
    let series = df.column(name).ok()?;
    let mut values = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        values.push(value.trim().to_string());
    }
    Some(values)
}

/// Get the label for a column from CSV table metadata.
pub fn table_label(_table: &DataFrame, _column: &str) -> Option<String> {
    // Labels are no longer supported in the optimized ingestion
    None
}

/// Find the source column name for a target SDTM variable in a mapping config.
pub fn mapping_source_for_target(mapping: &MappingConfig, target: &str) -> Option<String> {
    mapping
        .mappings
        .iter()
        .find(|entry| entry.target_variable.eq_ignore_ascii_case(target))
        .map(|entry| entry.source_column.clone())
}

/// Strip wrapping double quotes from a string value.
///
/// If the string starts and ends with double quotes, removes them.
/// Always trims leading/trailing whitespace.
///
/// # Examples
///
/// ```
/// use sdtm_normalization::data_utils::strip_quotes;
///
/// assert_eq!(strip_quotes("\"hello\""), "hello");
/// assert_eq!(strip_quotes("  \"world\"  "), "world");
/// assert_eq!(strip_quotes("unquoted"), "unquoted");
/// assert_eq!(strip_quotes("\"partial"), "\"partial");
/// ```
pub fn strip_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Remove all double quotes from a string value.
///
/// Used for SDTM identifiers (USUBJID, STUDYID) that should never contain quotes.
/// Always trims leading/trailing whitespace first.
///
/// # Examples
///
/// ```
/// use sdtm_normalization::data_utils::strip_all_quotes;
///
/// assert_eq!(strip_all_quotes("\"hello\""), "hello");
/// assert_eq!(strip_all_quotes("he\"llo"), "hello");
/// assert_eq!(strip_all_quotes("unquoted"), "unquoted");
/// ```
pub fn strip_all_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if !trimmed.contains('"') {
        return trimmed.to_string();
    }
    trimmed.chars().filter(|ch| *ch != '"').collect()
}

/// Sanitize a raw string into a valid SDTM identifier.
///
/// Converts to uppercase alphanumeric, replaces other characters with underscore,
/// collapses multiple underscores, and limits to max_len characters.
///
/// # Arguments
///
/// * `raw` - The raw input string
/// * `fallback` - Default value if result would be empty
/// * `prefix` - Character to prepend if result starts with digit
/// * `max_len` - Maximum length of result
fn sanitize_sdtm_identifier(raw: &str, fallback: &str, prefix: char, max_len: usize) -> String {
    // Build result, collapsing non-alphanumeric to single underscore
    let mut safe = String::with_capacity(raw.len());
    let mut last_was_underscore = true; // Treat start as underscore to skip leading
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            safe.push(ch.to_ascii_uppercase());
            last_was_underscore = false;
        } else if !last_was_underscore {
            safe.push('_');
            last_was_underscore = true;
        }
    }

    // Trim trailing underscore
    if safe.ends_with('_') {
        safe.pop();
    }

    // Use fallback if empty
    if safe.is_empty() {
        return if fallback.len() <= max_len {
            fallback.to_string()
        } else {
            fallback.chars().take(max_len).collect()
        };
    }

    // Prefix if starts with digit
    if safe.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        safe.insert(0, prefix);
    }

    if safe.len() <= max_len {
        safe
    } else {
        safe.chars().take(max_len).collect()
    }
}

/// Sanitize a test name into a valid --TESTCD code.
///
/// Per SDTMIG, test codes must be uppercase alphanumeric, start with a letter,
/// and be at most 8 characters.
pub fn sanitize_test_code(raw: &str) -> String {
    sanitize_sdtm_identifier(raw, "TEST", 'T', 8)
}

/// Sanitize a qualifier name into a valid QNAM code.
///
/// Per SDTMIG, QNAM values must be uppercase alphanumeric, start with a letter,
/// and be at most 8 characters.
pub fn sanitize_qnam(raw: &str) -> String {
    sanitize_sdtm_identifier(raw, "QVAL", 'Q', 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_quotes_removes_wrapping_quotes() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
        assert_eq!(strip_quotes("\"world\""), "world");
    }

    #[test]
    fn strip_quotes_trims_whitespace() {
        assert_eq!(strip_quotes("  \"hello\"  "), "hello");
        assert_eq!(strip_quotes("  unquoted  "), "unquoted");
    }

    #[test]
    fn strip_quotes_leaves_partial_quotes() {
        assert_eq!(strip_quotes("\"partial"), "\"partial");
        assert_eq!(strip_quotes("partial\""), "partial\"");
    }

    #[test]
    fn strip_quotes_handles_unquoted() {
        assert_eq!(strip_quotes("unquoted"), "unquoted");
        assert_eq!(strip_quotes(""), "");
    }

    #[test]
    fn strip_all_quotes_removes_all_quotes() {
        assert_eq!(strip_all_quotes("\"hello\""), "hello");
        assert_eq!(strip_all_quotes("he\"llo"), "hello");
        assert_eq!(strip_all_quotes("\"a\"b\"c\""), "abc");
    }

    #[test]
    fn strip_all_quotes_handles_no_quotes() {
        assert_eq!(strip_all_quotes("unquoted"), "unquoted");
        assert_eq!(strip_all_quotes("  trimmed  "), "trimmed");
    }

    #[test]
    fn sanitize_test_code_uppercase_alphanumeric() {
        assert_eq!(sanitize_test_code("weight"), "WEIGHT");
        assert_eq!(sanitize_test_code("SYSBP"), "SYSBP");
        assert_eq!(sanitize_test_code("wt-kg"), "WT_KG"); // 5 chars, fits
        assert_eq!(sanitize_test_code("weight-kg"), "WEIGHT_K"); // 9 chars truncated to 8
    }

    #[test]
    fn sanitize_test_code_truncates_to_8_chars() {
        assert_eq!(sanitize_test_code("verylongname"), "VERYLONG");
    }

    #[test]
    fn sanitize_test_code_prefixes_if_starts_with_digit() {
        assert_eq!(sanitize_test_code("123test"), "T123TEST");
    }

    #[test]
    fn sanitize_test_code_fallback_for_empty() {
        assert_eq!(sanitize_test_code(""), "TEST");
        assert_eq!(sanitize_test_code("---"), "TEST");
    }

    #[test]
    fn sanitize_qnam_uppercase_alphanumeric() {
        assert_eq!(sanitize_qnam("custom"), "CUSTOM");
        assert_eq!(sanitize_qnam("my-value"), "MY_VALUE");
    }

    #[test]
    fn sanitize_qnam_collapses_underscores() {
        assert_eq!(sanitize_qnam("a--b"), "A_B");
        assert_eq!(sanitize_qnam("a---b"), "A_B");
    }

    #[test]
    fn sanitize_qnam_prefixes_if_starts_with_digit() {
        assert_eq!(sanitize_qnam("123value"), "Q123VALU");
    }

    #[test]
    fn sanitize_qnam_fallback_for_empty() {
        assert_eq!(sanitize_qnam(""), "QVAL");
        assert_eq!(sanitize_qnam("---"), "QVAL");
    }
}
