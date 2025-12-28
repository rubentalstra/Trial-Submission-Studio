//! Tests for cross-domain validation module.

use std::collections::BTreeMap;

use polars::prelude::*;

use sdtm_validate::cross_domain::{
    CrossDomainValidationInput, infer_base_domain, validate_cross_domain,
};

fn make_df(columns: Vec<(&str, Vec<&str>)>) -> DataFrame {
    let mut cols = Vec::new();
    for (name, values) in columns {
        let series = Column::new(name.into(), values);
        cols.push(series);
    }
    DataFrame::new(cols).unwrap()
}

// --- Basic validation result tests ---

#[test]
fn validate_cross_domain_returns_empty_result() {
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

    // Cross-domain validation now returns empty (CT is our only source of truth)
    assert!(!result.has_issues());
    assert_eq!(result.total_issues(), 0);
}

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
fn into_reports_produces_empty_for_clean_data() {
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
    let reports = result.into_reports();

    assert!(reports.is_empty());
}

#[test]
fn merge_into_empty_map_no_issues() {
    let df = make_df(vec![
        ("STUDYID", vec!["STUDY1"]),
        ("USUBJID", vec!["SUBJ-001"]),
    ]);

    let mut frames = BTreeMap::new();
    frames.insert("DM".to_string(), &df);

    let input = CrossDomainValidationInput {
        frames: &frames,
        split_mappings: None,
    };

    let result = validate_cross_domain(input);
    let mut reports = BTreeMap::new();
    result.merge_into(&mut reports);

    // No issues to merge
    assert!(reports.is_empty());
}

// --- infer_base_domain tests ---

#[test]
fn infer_base_domain_standard() {
    assert_eq!(infer_base_domain("LB"), "LB");
    assert_eq!(infer_base_domain("DM"), "DM");
    assert_eq!(infer_base_domain("AE"), "AE");
    assert_eq!(infer_base_domain("VS"), "VS");
}

#[test]
fn infer_base_domain_split() {
    assert_eq!(infer_base_domain("LBCH"), "LB");
    assert_eq!(infer_base_domain("LBHE"), "LB");
    assert_eq!(infer_base_domain("QSFT"), "QS");
    assert_eq!(infer_base_domain("EGHR"), "EG");
}

#[test]
fn infer_base_domain_supp() {
    assert_eq!(infer_base_domain("SUPPDM"), "DM");
    assert_eq!(infer_base_domain("SUPPLB"), "LB");
    assert_eq!(infer_base_domain("SUPPAE"), "AE");
}

#[test]
fn infer_base_domain_relationship() {
    assert_eq!(infer_base_domain("RELREC"), "RELREC");
    assert_eq!(infer_base_domain("RELSPEC"), "RELSPEC");
    assert_eq!(infer_base_domain("RELSUB"), "RELSUB");
}

#[test]
fn infer_base_domain_lowercase() {
    assert_eq!(infer_base_domain("lb"), "LB");
    assert_eq!(infer_base_domain("lbch"), "LB");
    assert_eq!(infer_base_domain("suppdm"), "DM");
}

#[test]
fn infer_base_domain_unknown() {
    // Unknown domains returned as-is (uppercase)
    assert_eq!(infer_base_domain("CUSTOM"), "CUSTOM");
    assert_eq!(infer_base_domain("XX"), "XX");
}
