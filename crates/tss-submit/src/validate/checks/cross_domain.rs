//! Cross-domain reference validation.
//!
//! Validates referential integrity across SDTM domains:
//! - All USUBJIDs in non-DM domains must exist in DM
//! - Parent record references (--SPID) must exist in parent domain
//!
//! These checks ensure data consistency across the submission package.

use polars::prelude::DataFrame;
use std::collections::HashSet;

use super::super::column_reader::ColumnReader;
use super::super::issue::Issue;

/// Extract all USUBJIDs from the DM domain.
///
/// Returns a set of unique, non-empty USUBJID values.
pub fn extract_dm_subjects(dm_df: &DataFrame) -> HashSet<String> {
    let reader = ColumnReader::new(dm_df);
    let mut subjects = HashSet::new();

    if let Some(values) = reader.values("USUBJID") {
        for (_, value) in values {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                subjects.insert(trimmed.to_string());
            }
        }
    }

    subjects
}

/// Check that all USUBJIDs in a domain exist in the DM domain.
///
/// # Arguments
/// * `domain_name` - Name of the domain being checked (e.g., "AE", "LB")
/// * `df` - DataFrame of the domain to check
/// * `dm_subjects` - Set of valid USUBJIDs from DM domain
///
/// # Returns
/// A vector of issues (empty if all USUBJIDs are valid).
pub fn check_usubjid_in_dm(
    domain_name: &str,
    df: &DataFrame,
    dm_subjects: &HashSet<String>,
) -> Vec<Issue> {
    let reader = ColumnReader::new(df);

    let Some(values) = reader.values("USUBJID") else {
        // No USUBJID column - nothing to check
        return vec![];
    };

    let mut missing_values = Vec::new();
    let mut missing_count = 0u64;

    for (_, usubjid) in values {
        let trimmed = usubjid.trim();
        // Skip empty values (handled by other validation)
        if !trimmed.is_empty() && !dm_subjects.contains(trimmed) {
            missing_count += 1;
            // Collect up to 5 sample values
            if missing_values.len() < 5 && !missing_values.contains(&trimmed.to_string()) {
                missing_values.push(trimmed.to_string());
            }
        }
    }

    if missing_count > 0 {
        vec![Issue::UsubjidNotInDm {
            domain: domain_name.to_string(),
            missing_count,
            samples: missing_values,
        }]
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn dm_df() -> DataFrame {
        df! {
            "USUBJID" => &["STUDY-001", "STUDY-002", "STUDY-003"],
            "AGE" => &[25, 30, 35],
        }
        .unwrap()
    }

    fn ae_df_valid() -> DataFrame {
        df! {
            "USUBJID" => &["STUDY-001", "STUDY-002", "STUDY-001"],
            "AETERM" => &["HEADACHE", "NAUSEA", "FATIGUE"],
        }
        .unwrap()
    }

    fn ae_df_invalid() -> DataFrame {
        df! {
            "USUBJID" => &["STUDY-001", "STUDY-999", "STUDY-888"],
            "AETERM" => &["HEADACHE", "NAUSEA", "FATIGUE"],
        }
        .unwrap()
    }

    #[test]
    fn test_extract_dm_subjects() {
        let dm = dm_df();
        let subjects = extract_dm_subjects(&dm);

        assert_eq!(subjects.len(), 3);
        assert!(subjects.contains("STUDY-001"));
        assert!(subjects.contains("STUDY-002"));
        assert!(subjects.contains("STUDY-003"));
    }

    #[test]
    fn test_check_usubjid_all_valid() {
        let dm = dm_df();
        let ae = ae_df_valid();
        let dm_subjects = extract_dm_subjects(&dm);

        let issues = check_usubjid_in_dm("AE", &ae, &dm_subjects);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_check_usubjid_some_missing() {
        let dm = dm_df();
        let ae = ae_df_invalid();
        let dm_subjects = extract_dm_subjects(&dm);

        let issues = check_usubjid_in_dm("AE", &ae, &dm_subjects);
        assert_eq!(issues.len(), 1);

        match &issues[0] {
            Issue::UsubjidNotInDm {
                domain,
                missing_count,
                samples,
            } => {
                assert_eq!(domain, "AE");
                assert_eq!(*missing_count, 2);
                assert!(samples.contains(&"STUDY-999".to_string()));
                assert!(samples.contains(&"STUDY-888".to_string()));
            }
            _ => panic!("Expected UsubjidNotInDm issue"),
        }
    }

    #[test]
    fn test_check_usubjid_no_usubjid_column() {
        let dm = dm_df();
        let lb = df! {
            "LBTEST" => &["ALT", "AST"],
        }
        .unwrap();
        let dm_subjects = extract_dm_subjects(&dm);

        let issues = check_usubjid_in_dm("LB", &lb, &dm_subjects);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_extract_dm_subjects_empty() {
        let dm = DataFrame::empty();
        let subjects = extract_dm_subjects(&dm);
        assert!(subjects.is_empty());
    }

    #[test]
    fn test_check_usubjid_empty_subjects() {
        let dm_subjects = HashSet::new();
        let ae = ae_df_valid();

        // All USUBJIDs should be flagged as missing
        let issues = check_usubjid_in_dm("AE", &ae, &dm_subjects);
        assert_eq!(issues.len(), 1);

        match &issues[0] {
            Issue::UsubjidNotInDm { missing_count, .. } => {
                assert_eq!(*missing_count, 3); // All 3 rows have missing USUBJIDs
            }
            _ => panic!("Expected UsubjidNotInDm issue"),
        }
    }
}
