//! Tests for cross-domain validation module.
//!
//! Per AGENTS.md: Place tests in the `tests/` folder, not inline `#[cfg(test)]` modules.

use std::collections::BTreeMap;

use polars::prelude::*;

use sdtm_validate::{CrossDomainValidationInput, validate_cross_domain};

fn make_df(cols: Vec<(&str, Vec<&str>)>) -> DataFrame {
    let columns: Vec<Column> = cols
        .into_iter()
        .map(|(name, values)| Column::new(name.into(), values))
        .collect();
    DataFrame::new(columns).expect("dataframe")
}

// --- SEQ uniqueness across splits tests ---

#[test]
fn seq_across_splits_no_splits_returns_empty() {
    let df = make_df(vec![
        ("USUBJID", vec!["SUBJ-001", "SUBJ-002"]),
        ("LBSEQ", vec!["1", "1"]),
    ]);
    let mut frames = BTreeMap::new();
    frames.insert("LB".to_string(), &df);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    assert_eq!(result.seq_violations, 0);
    assert!(!result.has_issues());
}

#[test]
fn seq_across_splits_unique_values_passes() {
    let df_lbch = make_df(vec![
        ("USUBJID", vec!["SUBJ-001", "SUBJ-002"]),
        ("LBSEQ", vec!["1", "1"]),
    ]);
    let df_lbhe = make_df(vec![
        ("USUBJID", vec!["SUBJ-001", "SUBJ-002"]),
        ("LBSEQ", vec!["2", "2"]),
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("LBCH".to_string(), &df_lbch);
    frames.insert("LBHE".to_string(), &df_lbhe);

    let mut split_mappings = BTreeMap::new();
    split_mappings.insert("LBCH".to_string(), "LB".to_string());
    split_mappings.insert("LBHE".to_string(), "LB".to_string());

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: Some(&split_mappings),
    };

    let result = validate_cross_domain(input);
    assert_eq!(result.seq_violations, 0);
}

#[test]
fn seq_across_splits_duplicate_values_fails() {
    // Same --SEQ value for same USUBJID across splits
    let df_lbch = make_df(vec![("USUBJID", vec!["SUBJ-001"]), ("LBSEQ", vec!["1"])]);
    let df_lbhe = make_df(vec![
        ("USUBJID", vec!["SUBJ-001"]),
        ("LBSEQ", vec!["1"]), // Duplicate!
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("LBCH".to_string(), &df_lbch);
    frames.insert("LBHE".to_string(), &df_lbhe);

    let mut split_mappings = BTreeMap::new();
    split_mappings.insert("LBCH".to_string(), "LB".to_string());
    split_mappings.insert("LBHE".to_string(), "LB".to_string());

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: Some(&split_mappings),
    };

    let result = validate_cross_domain(input);
    assert!(
        result.seq_violations > 0,
        "should detect duplicate SEQ across splits"
    );
    assert!(result.has_issues());

    // Check the issue is in the LB domain (base domain for splits)
    assert!(result.issues_by_domain.contains_key("LB"));
}

// --- SUPPQUAL QNAM uniqueness tests ---

#[test]
fn supp_qnam_unique_passes() {
    let df = make_df(vec![
        ("STUDYID", vec!["STUDY1", "STUDY1"]),
        ("RDOMAIN", vec!["DM", "DM"]),
        ("USUBJID", vec!["SUBJ-001", "SUBJ-002"]),
        ("IDVAR", vec!["DMSEQ", "DMSEQ"]),
        ("IDVARVAL", vec!["1", "1"]),
        ("QNAM", vec!["CUSTOM1", "CUSTOM1"]), // Different subjects, same QNAM is OK
        ("QVAL", vec!["value1", "value2"]),
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("SUPPDM".to_string(), &df);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    assert_eq!(result.qnam_violations, 0);
}

#[test]
fn supp_qnam_duplicate_same_key_fails() {
    // Same (STUDYID, RDOMAIN, USUBJID, IDVAR, IDVARVAL) with duplicate QNAM
    let df = make_df(vec![
        ("STUDYID", vec!["STUDY1", "STUDY1"]),
        ("RDOMAIN", vec!["DM", "DM"]),
        ("USUBJID", vec!["SUBJ-001", "SUBJ-001"]),
        ("IDVAR", vec!["DMSEQ", "DMSEQ"]),
        ("IDVARVAL", vec!["1", "1"]),
        ("QNAM", vec!["CUSTOM1", "CUSTOM1"]), // Duplicate QNAM for same key!
        ("QVAL", vec!["value1", "value2"]),
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("SUPPDM".to_string(), &df);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    assert!(result.qnam_violations > 0, "should detect duplicate QNAM");
    assert!(result.has_issues());
}

// --- QVAL non-empty tests ---

#[test]
fn supp_qval_non_empty_passes() {
    let df = make_df(vec![
        ("STUDYID", vec!["STUDY1"]),
        ("RDOMAIN", vec!["DM"]),
        ("USUBJID", vec!["SUBJ-001"]),
        ("QNAM", vec!["CUSTOM1"]),
        ("QVAL", vec!["valid_value"]),
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("SUPPDM".to_string(), &df);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    assert_eq!(result.qval_violations, 0);
}

#[test]
fn supp_qval_empty_fails() {
    let df = make_df(vec![
        ("STUDYID", vec!["STUDY1", "STUDY1"]),
        ("RDOMAIN", vec!["DM", "DM"]),
        ("USUBJID", vec!["SUBJ-001", "SUBJ-002"]),
        ("QNAM", vec!["CUSTOM1", "CUSTOM2"]),
        ("QVAL", vec!["", "   "]), // Both empty/whitespace
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("SUPPDM".to_string(), &df);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    assert!(result.qval_violations > 0, "should detect empty QVAL");
    assert!(result.has_issues());
}

// --- RELREC relationship integrity tests ---

#[test]
fn relrec_valid_references_passes() {
    // Create parent domain with USUBJID and AESEQ
    let df_ae = make_df(vec![
        ("STUDYID", vec!["STUDY1"]),
        ("USUBJID", vec!["SUBJ-001"]),
        ("AESEQ", vec!["1"]),
    ]);

    // Create child domain with USUBJID and CMSEQ
    let df_cm = make_df(vec![
        ("STUDYID", vec!["STUDY1"]),
        ("USUBJID", vec!["SUBJ-001"]),
        ("CMSEQ", vec!["1"]),
    ]);

    // Create RELREC linking AE to CM
    let df_relrec = make_df(vec![
        ("STUDYID", vec!["STUDY1", "STUDY1"]),
        ("RDOMAIN", vec!["AE", "CM"]),
        ("USUBJID", vec!["SUBJ-001", "SUBJ-001"]),
        ("IDVAR", vec!["AESEQ", "CMSEQ"]),
        ("IDVARVAL", vec!["1", "1"]),
        ("RELID", vec!["R1", "R1"]),
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("AE".to_string(), &df_ae);
    frames.insert("CM".to_string(), &df_cm);
    frames.insert("RELREC".to_string(), &df_relrec);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    // Note: This validates the structure of RELREC; specific reference validation
    // is more complex and may be deferred
    assert_eq!(result.relrec_violations, 0);
}

#[test]
fn relrec_missing_domain_fails() {
    // RELREC references a domain that doesn't exist
    let df_ae = make_df(vec![
        ("STUDYID", vec!["STUDY1"]),
        ("USUBJID", vec!["SUBJ-001"]),
        ("AESEQ", vec!["1"]),
    ]);

    let df_relrec = make_df(vec![
        ("STUDYID", vec!["STUDY1", "STUDY1"]),
        ("RDOMAIN", vec!["AE", "XX"]), // XX doesn't exist!
        ("USUBJID", vec!["SUBJ-001", "SUBJ-001"]),
        ("IDVAR", vec!["AESEQ", "XXSEQ"]),
        ("IDVARVAL", vec!["1", "1"]),
        ("RELID", vec!["R1", "R1"]),
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("AE".to_string(), &df_ae);
    frames.insert("RELREC".to_string(), &df_relrec);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    assert!(
        result.relrec_violations > 0,
        "should detect missing referenced domain"
    );
}

// --- CrossDomainValidationResult methods tests ---

#[test]
fn result_merge_into_empty_map() {
    let df = make_df(vec![
        ("STUDYID", vec!["STUDY1"]),
        ("RDOMAIN", vec!["DM"]),
        ("USUBJID", vec!["SUBJ-001"]),
        ("QNAM", vec!["CUSTOM1"]),
        ("QVAL", vec![""]), // Empty QVAL
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("SUPPDM".to_string(), &df);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    assert!(result.has_issues());

    let mut reports = BTreeMap::new();
    result.merge_into(&mut reports);

    assert!(reports.contains_key("SUPPDM"));
    assert!(!reports.get("SUPPDM").unwrap().issues.is_empty());
}

#[test]
fn result_total_issues_count() {
    // Create multiple issues
    let df = make_df(vec![
        ("STUDYID", vec!["STUDY1", "STUDY1"]),
        ("RDOMAIN", vec!["DM", "DM"]),
        ("USUBJID", vec!["SUBJ-001", "SUBJ-001"]),
        ("IDVAR", vec!["DMSEQ", "DMSEQ"]),
        ("IDVARVAL", vec!["1", "1"]),
        ("QNAM", vec!["CUSTOM1", "CUSTOM1"]), // Duplicate
        ("QVAL", vec!["", ""]),               // Also empty
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("SUPPDM".to_string(), &df);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    assert!(result.total_issues() > 0);
}

#[test]
fn into_reports_produces_conformance_reports() {
    let df = make_df(vec![
        ("STUDYID", vec!["STUDY1"]),
        ("RDOMAIN", vec!["DM"]),
        ("USUBJID", vec!["SUBJ-001"]),
        ("QNAM", vec!["CUSTOM1"]),
        ("QVAL", vec![""]), // Empty QVAL
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("SUPPDM".to_string(), &df);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    let reports = result.into_reports();

    assert!(!reports.is_empty());
    let suppdm_report = reports.iter().find(|r| r.domain_code == "SUPPDM");
    assert!(suppdm_report.is_some());
}

// --- Edge cases ---

#[test]
fn empty_frames_returns_no_issues() {
    let frames: BTreeMap<String, &DataFrame> = BTreeMap::new();

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    assert!(!result.has_issues());
    assert_eq!(result.total_issues(), 0);
}

#[test]
fn non_supp_domains_skip_qnam_qval_checks() {
    // Regular domain should not trigger QNAM/QVAL checks
    let df = make_df(vec![
        ("STUDYID", vec!["STUDY1"]),
        ("USUBJID", vec!["SUBJ-001"]),
        ("DMSEQ", vec!["1"]),
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("DM".to_string(), &df);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    assert_eq!(result.qnam_violations, 0);
    assert_eq!(result.qval_violations, 0);
}
