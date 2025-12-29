//! Shared operations for domain processors.
//!
//! This module provides reusable functions for common data transformations
//! used across multiple SDTM domain processors. Each operation follows
//! SDTMIG v3.4 guidelines.

use std::collections::HashMap;

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use super::common::{
    col, has_column, map_values, normalize_ct_value, preferred_term_for, resolve_ct_value,
    set_string_column, string_column,
};
use crate::pipeline_context::PipelineContext;

/// Copy values from source column to target column where target is empty.
///
/// This is used for fallback patterns like:
/// - ORRES → STRESC (copy original result to standardized result)
/// - ORRESU → STRESU (copy original unit to standardized unit)
/// - TESTCD → TEST (copy test code to test name)
/// - TERM → DECOD (copy verbatim term to dictionary term)
///
/// # Arguments
///
/// * `df` - DataFrame to modify
/// * `source_col` - Column name to copy from (e.g., "LBORRES")
/// * `target_col` - Column name to copy to (e.g., "LBSTRESC")
pub fn backward_fill(df: &mut DataFrame, source_col: &str, target_col: &str) -> Result<()> {
    if !has_column(df, source_col) || !has_column(df, target_col) {
        return Ok(());
    }

    let source_vals = string_column(df, source_col)?;
    let mut target_vals = string_column(df, target_col)?;

    for (target, source) in target_vals.iter_mut().zip(source_vals.iter()) {
        if target.is_empty() && !source.is_empty() {
            *target = source.clone();
        }
    }

    set_string_column(df, target_col, target_vals)?;
    Ok(())
}

/// Copy values from source column to target column where target is empty,
/// using domain variable resolution.
///
/// # Arguments
///
/// * `domain` - Domain metadata for column name resolution
/// * `df` - DataFrame to modify
/// * `source_var` - Variable name to copy from (e.g., "ORRES")
/// * `target_var` - Variable name to copy to (e.g., "STRESC")
pub fn backward_fill_var(
    domain: &Domain,
    df: &mut DataFrame,
    source_var: &str,
    target_var: &str,
) -> Result<()> {
    let source_col = match col(domain, source_var) {
        Some(name) => name,
        None => return Ok(()),
    };
    let target_col = match col(domain, target_var) {
        Some(name) => name,
        None => return Ok(()),
    };

    backward_fill(df, source_col, target_col)
}

/// Clear unit column when corresponding result column is empty.
///
/// Per SDTMIG v3.4, units should only be populated when there's
/// a corresponding result value. This prevents orphaned units.
///
/// # Arguments
///
/// * `df` - DataFrame to modify
/// * `result_col` - Result column name (e.g., "LBORRES")
/// * `unit_col` - Unit column name (e.g., "LBORRESU")
pub fn clear_unit_when_empty(df: &mut DataFrame, result_col: &str, unit_col: &str) -> Result<()> {
    if !has_column(df, result_col) || !has_column(df, unit_col) {
        return Ok(());
    }

    let result_vals = string_column(df, result_col)?;
    let mut unit_vals = string_column(df, unit_col)?;

    for (unit, result) in unit_vals.iter_mut().zip(result_vals.iter()) {
        if result.is_empty() {
            unit.clear();
        }
    }

    set_string_column(df, unit_col, unit_vals)?;
    Ok(())
}

/// Clear unit column when corresponding result column is empty,
/// using domain variable resolution.
///
/// # Arguments
///
/// * `domain` - Domain metadata for column name resolution
/// * `df` - DataFrame to modify
/// * `result_var` - Result variable name (e.g., "ORRES")
/// * `unit_var` - Unit variable name (e.g., "ORRESU")
pub fn clear_unit_when_empty_var(
    domain: &Domain,
    df: &mut DataFrame,
    result_var: &str,
    unit_var: &str,
) -> Result<()> {
    let result_col = match col(domain, result_var) {
        Some(name) => name,
        None => return Ok(()),
    };
    let unit_col = match col(domain, unit_var) {
        Some(name) => name,
        None => return Ok(()),
    };

    clear_unit_when_empty(df, result_col, unit_col)
}

/// Standard Y/N value mapping for boolean-like fields.
///
/// Maps common variations to standard CDISC submission values:
/// - "YES", "Y", "1", "TRUE" → "Y"
/// - "NO", "N", "0", "FALSE" → "N"
/// - "", "NAN", "<NA>" → ""
///
/// Additional clinical significance mappings:
/// - "CS" → "Y" (Clinically Significant)
/// - "NCS" → "N" (Not Clinically Significant)
///
/// # Note
///
/// For CT-based normalization, prefer using `normalize_ct_columns` with
/// the appropriate codelist (e.g., C66742 for No Yes Response).
pub fn yn_mapping() -> HashMap<String, String> {
    map_values([
        ("YES", "Y"),
        ("Y", "Y"),
        ("1", "Y"),
        ("TRUE", "Y"),
        ("NO", "N"),
        ("N", "N"),
        ("0", "N"),
        ("FALSE", "N"),
        ("CS", "Y"),
        ("NCS", "N"),
        ("", ""),
        ("NAN", ""),
        ("<NA>", ""),
    ])
}

/// Normalize multiple columns via Controlled Terminology.
///
/// Applies CT normalization to all specified columns using the same codelist.
///
/// # Arguments
///
/// * `domain` - Domain metadata
/// * `df` - DataFrame to modify
/// * `context` - Pipeline context with CT registry
/// * `ct_field` - CT field name to resolve (e.g., "LBORRESU")
/// * `columns` - Column variable names to normalize (e.g., ["LBORRESU", "LBSTRESU"])
pub fn normalize_ct_columns(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
    ct_field: &str,
    columns: &[&str],
) -> Result<()> {
    let ct = match context.resolve_ct(domain, ct_field) {
        Some(ct) => ct,
        None => return Ok(()),
    };

    for col_name in columns {
        if let Some(name) = col(domain, col_name)
            && has_column(df, name)
        {
            let mut values = string_column(df, name)?;
            for value in &mut values {
                *value = normalize_ct_value(ct, value, context.options.ct_matching);
            }
            set_string_column(df, name, values)?;
        }
    }

    Ok(())
}

/// Derive test name from test code using CT preferred terms.
///
/// When test name (--TEST) is empty or matches the test code (--TESTCD),
/// attempts to populate it with the CT preferred term for the code.
///
/// # Arguments
///
/// * `domain` - Domain metadata
/// * `df` - DataFrame to modify
/// * `context` - Pipeline context with CT registry
/// * `test_var` - Test name variable (e.g., "LBTEST")
/// * `testcd_var` - Test code variable (e.g., "LBTESTCD")
/// * `ct_field` - CT field for test codes (e.g., "LBTESTCD")
pub fn derive_test_from_testcd(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
    test_var: &str,
    testcd_var: &str,
    ct_field: &str,
) -> Result<()> {
    let test_col = match col(domain, test_var) {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };
    let testcd_col = match col(domain, testcd_var) {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };
    let ct = match context.resolve_ct(domain, ct_field) {
        Some(ct) => ct,
        None => return Ok(()),
    };

    let ct_names = context.resolve_ct(domain, test_var);
    let mut test_vals = string_column(df, test_col)?;
    let testcd_vals = string_column(df, testcd_col)?;

    for (test, testcd) in test_vals.iter_mut().zip(testcd_vals.iter()) {
        if testcd.is_empty() {
            continue;
        }

        let needs_label = test.is_empty() || test.eq_ignore_ascii_case(testcd);
        let valid_name = ct_names
            .map(|ct| {
                let canonical = normalize_ct_value(ct, test, context.options.ct_matching);
                ct.submission_values().iter().any(|val| val == &canonical)
            })
            .unwrap_or(true);

        if !needs_label && valid_name {
            continue;
        }

        if let Some(preferred) = preferred_term_for(ct, testcd) {
            *test = preferred;
        }
    }

    set_string_column(df, test_col, test_vals)?;
    Ok(())
}

/// Resolve test code from test name using CT when code is invalid.
///
/// If the test code (--TESTCD) is empty or not in the CT submission values,
/// attempts to resolve it from the test name (--TEST).
///
/// # Arguments
///
/// * `domain` - Domain metadata
/// * `df` - DataFrame to modify
/// * `context` - Pipeline context with CT registry
/// * `testcd_var` - Test code variable (e.g., "LBTESTCD")
/// * `test_var` - Test name variable (e.g., "LBTEST")
/// * `ct_field` - CT field for test codes (e.g., "LBTESTCD")
pub fn resolve_testcd_from_test(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
    testcd_var: &str,
    test_var: &str,
    ct_field: &str,
) -> Result<()> {
    let testcd_col = match col(domain, testcd_var) {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };
    let test_col = match col(domain, test_var) {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };
    let ct = match context.resolve_ct(domain, ct_field) {
        Some(ct) => ct,
        None => return Ok(()),
    };

    let test_vals = string_column(df, test_col)?;
    let mut testcd_vals = string_column(df, testcd_col)?;

    for (testcd, test) in testcd_vals.iter_mut().zip(test_vals.iter()) {
        let existing = testcd.clone();
        let valid =
            !existing.is_empty() && ct.submission_values().iter().any(|val| val == &existing);

        if valid {
            continue;
        }

        if let Some(mapped) = resolve_ct_value(ct, test, context.options.ct_matching) {
            *testcd = mapped;
        }
    }

    set_string_column(df, testcd_col, testcd_vals)?;
    Ok(())
}

/// Check if a string value represents a missing/NA value.
///
/// Recognizes common NA representations (case-insensitive):
/// - Empty string
/// - "NA", "N/A", "<NA>"
/// - "NAN", "nan"
/// - "None"
/// - "UNK", "UNKNOWN"
///
/// # Examples
///
/// ```ignore
/// assert!(is_na_value("NA"));
/// assert!(is_na_value("n/a"));
/// assert!(is_na_value("<NA>"));
/// assert!(is_na_value("nan"));
/// assert!(is_na_value("None"));
/// assert!(is_na_value("UNKNOWN"));
/// assert!(!is_na_value("NORMAL"));
/// ```
pub fn is_na_value(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return true;
    }
    matches!(
        trimmed.to_uppercase().as_str(),
        "NA" | "N/A" | "<NA>" | "NAN" | "NONE" | "UNK" | "UNKNOWN"
    )
}

/// Clean NA-like values from string columns.
///
/// Replaces common NA representations with empty strings using [`is_na_value`].
/// This provides consistent NA handling across all domain processors.
///
/// # Arguments
///
/// * `df` - DataFrame to modify
/// * `column` - Column name to clean
pub fn clean_na_values(df: &mut DataFrame, column: &str) -> Result<()> {
    if !has_column(df, column) {
        return Ok(());
    }

    let values = string_column(df, column)?
        .into_iter()
        .map(|value| {
            let trimmed = value.trim();
            if is_na_value(trimmed) {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .collect();

    set_string_column(df, column, values)?;
    Ok(())
}

/// Clean NA-like values from multiple columns using domain resolution.
///
/// # Arguments
///
/// * `domain` - Domain metadata
/// * `df` - DataFrame to modify
/// * `columns` - Variable names to clean
pub fn clean_na_values_vars(domain: &Domain, df: &mut DataFrame, columns: &[&str]) -> Result<()> {
    for col_name in columns {
        if let Some(name) = col(domain, col_name)
            && has_column(df, name)
        {
            clean_na_values(df, name)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_na_value_recognizes_empty() {
        assert!(is_na_value(""));
        assert!(is_na_value("   "));
        assert!(is_na_value("\t\n"));
    }

    #[test]
    fn is_na_value_recognizes_na_variants() {
        // Case insensitive NA patterns
        assert!(is_na_value("NA"));
        assert!(is_na_value("na"));
        assert!(is_na_value("N/A"));
        assert!(is_na_value("n/a"));
        assert!(is_na_value("<NA>"));
        assert!(is_na_value("<na>"));
    }

    #[test]
    fn is_na_value_recognizes_nan_variants() {
        assert!(is_na_value("NAN"));
        assert!(is_na_value("nan"));
        assert!(is_na_value("NaN"));
    }

    #[test]
    fn is_na_value_recognizes_none() {
        assert!(is_na_value("NONE"));
        assert!(is_na_value("None"));
        assert!(is_na_value("none"));
    }

    #[test]
    fn is_na_value_recognizes_unknown() {
        assert!(is_na_value("UNK"));
        assert!(is_na_value("unk"));
        assert!(is_na_value("UNKNOWN"));
        assert!(is_na_value("unknown"));
        assert!(is_na_value("Unknown"));
    }

    #[test]
    fn is_na_value_rejects_valid_values() {
        assert!(!is_na_value("NORMAL"));
        assert!(!is_na_value("123"));
        assert!(!is_na_value("mg/dL"));
        assert!(!is_na_value("POSITIVE"));
        assert!(!is_na_value("Y"));
        assert!(!is_na_value("NANA")); // Contains NA but is not NA
    }

    #[test]
    fn is_na_value_handles_whitespace() {
        assert!(is_na_value("  NA  "));
        assert!(is_na_value("\tNAN\n"));
        assert!(is_na_value("  UNKNOWN  "));
    }
}
