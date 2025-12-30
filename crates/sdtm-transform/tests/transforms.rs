//! Tests for SDTM transforms.

use polars::prelude::*;
use sdtm_transform::{apply_usubjid_prefix, assign_sequence_numbers};

#[test]
fn test_apply_usubjid_prefix_adds_prefix() {
    let mut df = DataFrame::new(vec![
        Series::new("USUBJID".into(), vec!["001", "002", "003"]).into(),
    ])
    .unwrap();

    let modified = apply_usubjid_prefix(&mut df, "STUDY01", "USUBJID", None).unwrap();

    assert_eq!(modified, 3);
    let col = df.column("USUBJID").unwrap().str().unwrap();
    assert_eq!(col.get(0), Some("STUDY01-001"));
    assert_eq!(col.get(1), Some("STUDY01-002"));
    assert_eq!(col.get(2), Some("STUDY01-003"));
}

#[test]
fn test_apply_usubjid_prefix_skips_existing() {
    let mut df = DataFrame::new(vec![
        Series::new("USUBJID".into(), vec!["STUDY01-001", "002"]).into(),
    ])
    .unwrap();

    let modified = apply_usubjid_prefix(&mut df, "STUDY01", "USUBJID", None).unwrap();

    assert_eq!(modified, 1); // Only second row modified
    let col = df.column("USUBJID").unwrap().str().unwrap();
    assert_eq!(col.get(0), Some("STUDY01-001")); // Unchanged
    assert_eq!(col.get(1), Some("STUDY01-002")); // Prefixed
}

#[test]
fn test_assign_sequence_numbers() {
    let mut df = DataFrame::new(vec![
        Series::new("USUBJID".into(), vec!["A", "A", "B", "A", "B"]).into(),
    ])
    .unwrap();

    let count = assign_sequence_numbers(&mut df, "AESEQ", "USUBJID").unwrap();

    assert_eq!(count, 5);
    let seq = df.column("AESEQ").unwrap().f64().unwrap();
    // A gets 1, 2, 3; B gets 1, 2
    assert_eq!(seq.get(0), Some(1.0)); // A-1
    assert_eq!(seq.get(1), Some(2.0)); // A-2
    assert_eq!(seq.get(2), Some(1.0)); // B-1
    assert_eq!(seq.get(3), Some(3.0)); // A-3
    assert_eq!(seq.get(4), Some(2.0)); // B-2
}
