//! Required variable checks (SDTMIG 4.1).
//!
//! Checks that all Required (Req) variables are present and populated.

use polars::prelude::{AnyValue, DataFrame};
use tss_standards::any_to_string;
use tss_standards::{CoreDesignation, SdtmDomain};

use super::super::issue::Issue;
use super::super::util::CaseInsensitiveSet;

/// Check required variables are present and populated.
pub fn check(domain: &SdtmDomain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();
    let row_count = df.height() as u64;

    for variable in &domain.variables {
        if variable.core != Some(CoreDesignation::Required) {
            continue;
        }

        // Check presence
        let Some(column) = columns.get(&variable.name) else {
            issues.push(Issue::RequiredMissing {
                variable: variable.name.clone(),
            });
            continue;
        };

        // Check population (no nulls allowed for Req)
        let null_count = count_null_values(df, column);

        // If ALL values are null/empty, treat as "missing" (unmapped)
        if null_count == row_count && row_count > 0 {
            issues.push(Issue::RequiredMissing {
                variable: variable.name.clone(),
            });
        } else if null_count > 0 {
            issues.push(Issue::RequiredEmpty {
                variable: variable.name.clone(),
                null_count,
            });
        }
    }

    issues
}

/// Count null/empty values in a column.
fn count_null_values(df: &DataFrame, column: &str) -> u64 {
    let Ok(series) = df.column(column) else {
        return 0;
    };

    let mut count = 0u64;
    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        let str_value = any_to_string(value);
        if str_value.trim().is_empty() {
            count += 1;
        }
    }
    count
}
