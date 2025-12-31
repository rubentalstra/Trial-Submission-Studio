//! Expected variable checks (SDTMIG 4.1).
//!
//! Checks that Expected (Exp) variables are present (warnings only).
//!
//! A variable is considered "missing" if:
//! - The column doesn't exist in the DataFrame, OR
//! - The column exists but ALL values are null/empty (unmapped)

use polars::prelude::{AnyValue, DataFrame};
use sdtm_ingest::any_to_string;
use sdtm_model::{CoreDesignation, Domain};

use crate::issue::Issue;
use crate::util::CaseInsensitiveSet;

/// Check expected variables are present.
pub fn check(domain: &Domain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();
    let row_count = df.height();

    for variable in &domain.variables {
        if variable.core != Some(CoreDesignation::Expected) {
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
