//! Expected variable checks (SDTMIG 4.1).
//!
//! Checks that Expected (Exp) variables are present (warnings only).
//!
//! A variable is considered "missing" if:
//! - The column doesn't exist in the DataFrame, OR
//! - The column exists but ALL values are null/empty (unmapped)
//!
//! Variables marked as "not collected" are exempt from these checks,
//! as the user has explicitly acknowledged the data was not collected.

use polars::prelude::{AnyValue, DataFrame};
use std::collections::BTreeSet;
use tss_model::any_to_string;
use tss_model::{CoreDesignation, Domain};

use crate::issue::Issue;
use crate::util::CaseInsensitiveSet;

/// Check expected variables are present.
///
/// # Arguments
/// * `domain` - SDTM domain definition
/// * `df` - DataFrame to validate
/// * `columns` - Case-insensitive column name lookup
/// * `not_collected` - Variables explicitly marked as "not collected" by user
pub fn check(
    domain: &Domain,
    df: &DataFrame,
    columns: &CaseInsensitiveSet,
    not_collected: &BTreeSet<String>,
) -> Vec<Issue> {
    let mut issues = Vec::new();
    let row_count = df.height();

    for variable in &domain.variables {
        if variable.core != Some(CoreDesignation::Expected) {
            continue;
        }

        // Skip variables explicitly marked as "not collected"
        // The user has acknowledged this data was not collected in the study
        if not_collected.contains(&variable.name) {
            continue;
        }

        // Check presence
        let Some(column) = columns.get(&variable.name) else {
            issues.push(Issue::ExpectedMissing {
                variable: variable.name.clone(),
            });
            continue;
        };

        // Check if ALL values are empty (effectively unmapped)
        if row_count > 0 && is_all_empty(df, column, row_count) {
            issues.push(Issue::ExpectedMissing {
                variable: variable.name.clone(),
            });
        }
    }

    issues
}

/// Check if all values in a column are null/empty.
fn is_all_empty(df: &DataFrame, column: &str, row_count: usize) -> bool {
    let Ok(series) = df.column(column) else {
        return true;
    };

    for idx in 0..row_count {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        let str_value = any_to_string(value);
        if !str_value.trim().is_empty() {
            return false;
        }
    }
    true
}
