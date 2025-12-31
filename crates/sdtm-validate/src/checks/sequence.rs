//! Sequence uniqueness validation (SDTMIG 4.1.5).
//!
//! Checks that --SEQ values are unique per USUBJID.

use std::collections::HashSet;

use polars::prelude::{AnyValue, DataFrame};
use sdtm_ingest::any_to_string;
use sdtm_model::Domain;

use crate::issue::Issue;
use crate::util::CaseInsensitiveSet;

/// Check that --SEQ values are unique per USUBJID.
pub fn check(domain: &Domain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
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
    let (Ok(subj_series), Ok(seq_series)) = (df.column(subject_col), df.column(seq_col)) else {
        return 0;
    };

    // Build map of (USUBJID, SEQ) pairs and count duplicates
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut duplicate_count = 0u64;

    for idx in 0..df.height() {
        let subj = any_to_string(subj_series.get(idx).unwrap_or(AnyValue::Null));
        let seq = any_to_string(seq_series.get(idx).unwrap_or(AnyValue::Null));

        let key = (subj.trim().to_string(), seq.trim().to_string());
        if key.0.is_empty() || key.1.is_empty() {
            continue;
        }

        if !seen.insert(key) {
            duplicate_count += 1;
        }
    }

    duplicate_count
}
