//! Unit tests for CSV table parsing utilities.

use sdtm_ingest::{looks_like_label, looks_like_variable_code, parse_csv_line};

#[test]
fn test_looks_like_label() {
    // Long strings look like labels
    assert!(looks_like_label("Site sequence number"));
    assert!(looks_like_label("Subject Id with spaces"));
    assert!(looks_like_label("Reason - Code"));

    // Short strings without spaces don't look like labels
    assert!(!looks_like_label("AGE"));
    assert!(!looks_like_label("SEX"));
    assert!(!looks_like_label("SiteSeq"));
}

#[test]
fn test_looks_like_variable_code() {
    // Short uppercase codes
    assert!(looks_like_variable_code("AGE"));
    assert!(looks_like_variable_code("SEX"));
    assert!(looks_like_variable_code("MENOSTAT"));
    assert!(looks_like_variable_code("SiteSeq"));
    assert!(looks_like_variable_code("SubjectId"));
    assert!(looks_like_variable_code("ICYNCD"));

    // Long labels with spaces don't look like codes
    assert!(!looks_like_variable_code("Site sequence number"));
    assert!(!looks_like_variable_code("Subject Id with spaces"));
    assert!(!looks_like_variable_code(""));
}

#[test]
fn test_parse_csv_line() {
    let line = "Site sequence number,Site name,Subject Id";
    let parsed = parse_csv_line(line);
    assert_eq!(
        parsed,
        vec!["Site sequence number", "Site name", "Subject Id"]
    );

    // With quotes
    let line2 = "\"Quoted field\",Normal,\"Has, comma\"";
    let parsed2 = parse_csv_line(line2);
    assert_eq!(parsed2, vec!["Quoted field", "Normal", "Has, comma"]);
}
