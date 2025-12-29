//! Integration tests for the pipeline module.

use polars::prelude::{Column, DataFrame, IntoColumn, NamedFrom, Series};

use sdtm_cli::pipeline::extract_reference_starts;

fn test_df(columns: Vec<(&str, Vec<&str>)>) -> DataFrame {
    let cols: Vec<Column> = columns
        .into_iter()
        .map(|(name, values)| {
            Series::new(
                name.into(),
                values.iter().copied().map(String::from).collect::<Vec<_>>(),
            )
            .into_column()
        })
        .collect();
    DataFrame::new(cols).unwrap()
}

#[test]
fn test_extract_reference_starts_basic() {
    let df = test_df(vec![
        ("USUBJID", vec!["STUDY01-001", "STUDY01-002", "STUDY01-003"]),
        ("RFSTDTC", vec!["2024-01-15", "2024-01-16", "2024-01-17"]),
    ]);

    let starts = extract_reference_starts(&df);

    assert_eq!(starts.len(), 3);
    assert_eq!(starts.get("STUDY01-001"), Some(&"2024-01-15".to_string()));
    assert_eq!(starts.get("STUDY01-002"), Some(&"2024-01-16".to_string()));
    assert_eq!(starts.get("STUDY01-003"), Some(&"2024-01-17".to_string()));
}

#[test]
fn test_extract_reference_starts_with_missing_values() {
    let df = test_df(vec![
        ("USUBJID", vec!["STUDY01-001", "STUDY01-002", ""]),
        ("RFSTDTC", vec!["2024-01-15", "", "2024-01-17"]),
    ]);

    let starts = extract_reference_starts(&df);

    // Only the first subject has both USUBJID and RFSTDTC
    assert_eq!(starts.len(), 1);
    assert_eq!(starts.get("STUDY01-001"), Some(&"2024-01-15".to_string()));
}

#[test]
fn test_extract_reference_starts_case_insensitive() {
    let df = test_df(vec![
        ("usubjid", vec!["STUDY01-001"]),
        ("rfstdtc", vec!["2024-01-15"]),
    ]);

    let starts = extract_reference_starts(&df);

    assert_eq!(starts.len(), 1);
    assert_eq!(starts.get("STUDY01-001"), Some(&"2024-01-15".to_string()));
}

#[test]
fn test_extract_reference_starts_missing_columns() {
    // Missing RFSTDTC column
    let df1 = test_df(vec![("USUBJID", vec!["STUDY01-001"])]);
    let starts1 = extract_reference_starts(&df1);
    assert!(starts1.is_empty());

    // Missing USUBJID column
    let df2 = test_df(vec![("RFSTDTC", vec!["2024-01-15"])]);
    let starts2 = extract_reference_starts(&df2);
    assert!(starts2.is_empty());
}

#[test]
fn test_extract_reference_starts_first_value_wins() {
    // Duplicate USUBJID - first RFSTDTC value should be kept
    let df = test_df(vec![
        ("USUBJID", vec!["STUDY01-001", "STUDY01-001"]),
        ("RFSTDTC", vec!["2024-01-15", "2024-01-20"]),
    ]);

    let starts = extract_reference_starts(&df);

    assert_eq!(starts.len(), 1);
    // First value wins
    assert_eq!(starts.get("STUDY01-001"), Some(&"2024-01-15".to_string()));
}
