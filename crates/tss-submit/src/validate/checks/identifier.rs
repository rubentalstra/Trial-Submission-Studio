//! Identifier null checks (SDTMIG 4.1.2).
//!
//! Checks that Identifier role variables have no null values.

use polars::prelude::{AnyValue, DataFrame};
use tss_standards::any_to_string;
use tss_standards::{SdtmDomain, VariableRole};

use super::super::issue::Issue;
use super::super::util::CaseInsensitiveSet;

/// Check that Identifier role variables have no null values.
pub fn check(domain: &SdtmDomain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        if variable.role != Some(VariableRole::Identifier) {
            continue;
        }

        let Some(column) = columns.get(&variable.name) else {
            continue;
        };

        let null_count = count_null_values(df, column);
        if null_count > 0 {
            issues.push(Issue::IdentifierNull {
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
