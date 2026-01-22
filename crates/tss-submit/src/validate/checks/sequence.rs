//! Sequence uniqueness validation (SDTMIG 4.1.5).
//!
//! Checks that --SEQ values are unique per USUBJID.

use std::collections::HashSet;

use polars::prelude::DataFrame;
use tss_standards::SdtmDomain;

use super::super::column_reader::ColumnReader;
use super::super::issue::Issue;
use super::super::util::CaseInsensitiveSet;

/// Check that --SEQ values are unique per USUBJID.
pub fn check(domain: &SdtmDomain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Find the --SEQ variable for this domain
    let seq_var_name = format!("{}SEQ", domain.name.to_uppercase());
    let seq_column = columns.get(&seq_var_name);

    // USUBJID should always be present
    let usubjid_column = columns.get("USUBJID");

    // If no SEQ column or no USUBJID, skip this check
    let (Some(seq_col), Some(subj_col)) = (seq_column, usubjid_column) else {
        return issues;
    };

    let duplicate_count = count_duplicate_sequences(df, subj_col, seq_col);
    if duplicate_count > 0 {
        issues.push(Issue::DuplicateSequence {
            variable: seq_var_name,
            duplicate_count,
        });
    }

    issues
}

/// Count duplicate sequence values per subject.
fn count_duplicate_sequences(df: &DataFrame, subject_col: &str, seq_col: &str) -> u64 {
    let reader = ColumnReader::new(df);
    let by_subject = reader.values_by_subject(subject_col, seq_col);

    // Count sequences that appear more than once per subject
    let mut duplicate_count = 0u64;
    for values in by_subject.values() {
        let mut seen: HashSet<&str> = HashSet::new();
        for seq in values {
            let trimmed = seq.trim();
            if !trimmed.is_empty() && !seen.insert(trimmed) {
                duplicate_count += 1;
            }
        }
    }

    duplicate_count
}
