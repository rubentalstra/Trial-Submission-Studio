//! ISO 8601 date format validation (SDTMIG Chapter 7).
//!
//! Checks that date/datetime variables conform to ISO 8601 format.

use std::sync::LazyLock;

use polars::prelude::{AnyValue, DataFrame};
use regex::Regex;
use tss_model::any_to_string;
use tss_model::Domain;

use crate::issue::Issue;
use crate::util::CaseInsensitiveSet;

/// ISO 8601 date patterns per SDTMIG Chapter 7.
/// Supports: YYYY, YYYY-MM, YYYY-MM-DD, YYYY-MM-DDTHH:MM:SS
static ISO8601_DATE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(\d{4})(-((0[1-9]|1[0-2]))(-((0[1-9]|[12]\d|3[01]))(T(([01]\d|2[0-3]):([0-5]\d)(:([0-5]\d)(\.\d+)?)?)?)?)?)?$",
    )
    .expect("Invalid ISO 8601 regex")
});

/// Known date/time variable name suffixes that require ISO 8601 validation.
const DATE_SUFFIXES: &[&str] = &["DTC", "DTM", "DT", "TM", "STDTC", "ENDTC", "STDT", "ENDT"];

/// Check that date/datetime variables conform to ISO 8601 format.
pub fn check(domain: &Domain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        // Only check variables that appear to be date/time fields
        if !is_date_variable(&variable.name) {
            continue;
        }

        let Some(column) = columns.get(&variable.name) else {
            continue;
        };

        let (invalid_count, samples) = collect_invalid_dates(df, column);
        if invalid_count > 0 {
            issues.push(Issue::InvalidDate {
                variable: variable.name.clone(),
                invalid_count,
                samples,
            });
        }
    }

    issues
}

/// Check if a variable name indicates it's a date/time field.
pub fn is_date_variable(name: &str) -> bool {
    let upper = name.to_uppercase();
    DATE_SUFFIXES.iter().any(|suffix| upper.ends_with(suffix))
}

/// Collect values that don't conform to ISO 8601.
fn collect_invalid_dates(df: &DataFrame, column: &str) -> (u64, Vec<String>) {
    let Ok(series) = df.column(column) else {
        return (0, vec![]);
    };

    let mut count = 0u64;
    let mut samples = Vec::new();
    const MAX_SAMPLES: usize = 5;

    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        let str_value = any_to_string(value);
        let trimmed = str_value.trim();

        if trimmed.is_empty() {
            continue; // Nulls are OK for dates
        }

        if !ISO8601_DATE_REGEX.is_match(trimmed) {
            count += 1;
            if samples.len() < MAX_SAMPLES {
                samples.push(trimmed.to_string());
            }
        }
    }

    (count, samples)
}
