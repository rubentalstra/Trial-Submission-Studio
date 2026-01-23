//! Required variable checks (SDTMIG 4.1).
//!
//! Checks that all Required (Req) variables are present and populated.

use polars::prelude::DataFrame;
use tss_standards::{CoreDesignation, SdtmDomain};

use super::super::column_reader::ColumnReader;
use super::super::issue::Issue;
use super::super::util::CaseInsensitiveSet;

/// Check required variables are present and populated.
pub fn check(domain: &SdtmDomain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();
    let row_count = df.height() as u64;
    let reader = ColumnReader::new(df);

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
        let null_count = reader.count_nulls(column);

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
