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

use polars::prelude::DataFrame;
use std::collections::BTreeSet;
use tss_standards::{CoreDesignation, SdtmDomain};

use super::super::column_reader::ColumnReader;
use super::super::issue::Issue;
use super::super::util::CaseInsensitiveSet;

/// Check expected variables are present.
///
/// # Arguments
/// * `domain` - SDTM domain definition
/// * `df` - DataFrame to validate
/// * `columns` - Case-insensitive column name lookup
/// * `not_collected` - Variables explicitly marked as "not collected" by user
pub fn check(
    domain: &SdtmDomain,
    df: &DataFrame,
    columns: &CaseInsensitiveSet,
    not_collected: &BTreeSet<String>,
) -> Vec<Issue> {
    let mut issues = Vec::new();
    let reader = ColumnReader::new(df);

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
        if reader.all_null(column) {
            issues.push(Issue::ExpectedMissing {
                variable: variable.name.clone(),
            });
        }
    }

    issues
}
