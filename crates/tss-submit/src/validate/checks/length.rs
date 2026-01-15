//! Text length validation (SDTMIG 2.4).
//!
//! Checks that character variables don't exceed their defined length.

use polars::prelude::{AnyValue, DataFrame};
use tss_standards::any_to_string;
use tss_standards::{SdtmDomain, VariableType};

use super::super::issue::Issue;
use super::super::util::CaseInsensitiveSet;

/// Check that character variables don't exceed their defined length.
pub fn check(domain: &SdtmDomain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        // Only check Char variables with a defined length
        if variable.data_type != VariableType::Char {
            continue;
        }

        let Some(max_length) = variable.length else {
            continue;
        };

        let Some(column) = columns.get(&variable.name) else {
            continue;
        };

        let (exceeded_count, max_found) = collect_length_violations(df, column, max_length);
        if exceeded_count > 0 {
            issues.push(Issue::TextTooLong {
                variable: variable.name.clone(),
                exceeded_count,
                max_found,
                max_allowed: max_length,
            });
        }
    }

    issues
}

/// Collect values that exceed the specified length.
fn collect_length_violations(df: &DataFrame, column: &str, max_length: u32) -> (u64, usize) {
    let Ok(series) = df.column(column) else {
        return (0, 0);
    };

    let mut count = 0u64;
    let mut max_found = 0usize;

    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        let str_value = any_to_string(value);
        let len = str_value.len();

        if len > max_length as usize {
            count += 1;
            max_found = max_found.max(len);
        }
    }

    (count, max_found)
}
