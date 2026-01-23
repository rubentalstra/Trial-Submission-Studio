//! Identifier null checks (SDTMIG 4.1.2).
//!
//! Checks that Identifier role variables have no null values.

use polars::prelude::DataFrame;
use tss_standards::{SdtmDomain, VariableRole};

use super::super::column_reader::ColumnReader;
use super::super::issue::Issue;
use super::super::util::CaseInsensitiveSet;

/// Check that Identifier role variables have no null values.
pub fn check(domain: &SdtmDomain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();
    let reader = ColumnReader::new(df);

    for variable in &domain.variables {
        if variable.role != Some(VariableRole::Identifier) {
            continue;
        }

        let Some(column) = columns.get(&variable.name) else {
            continue;
        };

        let null_count = reader.count_nulls(column);
        if null_count > 0 {
            issues.push(Issue::IdentifierNull {
                variable: variable.name.clone(),
                null_count,
            });
        }
    }

    issues
}
