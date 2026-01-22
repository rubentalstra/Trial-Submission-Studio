//! Text length validation (SDTMIG 2.4).
//!
//! Checks that character variables don't exceed their defined length.

use polars::prelude::DataFrame;
use tss_standards::{SdtmDomain, VariableType};

use super::super::column_reader::ColumnReader;
use super::super::issue::Issue;
use super::super::util::CaseInsensitiveSet;

/// Check that character variables don't exceed their defined length.
pub fn check(domain: &SdtmDomain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();
    let reader = ColumnReader::new(df);

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

        let (exceeded_count, max_found) = reader.length_violations(column, max_length);
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
