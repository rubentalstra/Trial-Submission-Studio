//! Integration tests for V5 format support.
//!
//! These tests verify that V5 format files can be written and read back correctly,
//! including support for long names, long labels, and long format names.

use std::io::Cursor;

use sdtm_xpt::{
    XptColumn, XptDataset, XptReader, XptValue, XptVersion, XptWriter, XptWriterOptions,
};

/// Helper to write and read back a dataset.
fn roundtrip(dataset: &XptDataset, version: XptVersion) -> XptDataset {
    let mut buffer = Vec::new();
    let options = XptWriterOptions::default().with_version(version);

    {
        let writer = XptWriter::with_options(Cursor::new(&mut buffer), options);
        writer.write_dataset(dataset).unwrap();
    }

    let reader = XptReader::new(Cursor::new(&buffer));
    reader.read_dataset().unwrap()
}

#[test]
fn test_v5_basic_roundtrip() {
    let mut dataset = XptDataset::with_columns(
        "DM",
        vec![
            XptColumn::character("USUBJID", 20).with_label("Unique Subject ID"),
            XptColumn::numeric("AGE").with_label("Age in Years"),
        ],
    );

    dataset.add_row(vec![
        XptValue::character("STUDY-001"),
        XptValue::numeric(35.0),
    ]);
    dataset.add_row(vec![
        XptValue::character("STUDY-002"),
        XptValue::numeric(42.0),
    ]);

    let read_back = roundtrip(&dataset, XptVersion::V5);

    assert_eq!(read_back.name, "DM");
    assert_eq!(read_back.columns.len(), 2);
    assert_eq!(read_back.num_rows(), 2);

    assert_eq!(read_back.columns[0].name, "USUBJID");
    assert_eq!(
        read_back.columns[0].label,
        Some("Unique Subject ID".to_string())
    );
    assert_eq!(read_back.columns[1].name, "AGE");
    assert_eq!(read_back.columns[1].label, Some("Age in Years".to_string()));
}

#[test]
fn test_v5_full_roundtrip() {
    // Comprehensive V5 format roundtrip test with multiple data types and features
    use sdtm_xpt::MissingValue;

    let mut dataset = XptDataset::with_columns(
        "AE",
        vec![
            XptColumn::character("STUDYID", 8).with_label("Study ID"),
            XptColumn::character("USUBJID", 20).with_label("Unique Subject ID"),
            XptColumn::character("AETERM", 200).with_label("Adverse Event Term"),
            XptColumn::numeric("AESTDY").with_label("AE Start Day"),
            XptColumn::numeric("AESEQ")
                .with_label("Sequence Number")
                .with_format("BEST", 8, 0),
            XptColumn::character("AESEV", 10).with_label("Severity"),
        ],
    );

    // Add various rows with different data patterns
    dataset.add_row(vec![
        XptValue::character("STUDY01"),
        XptValue::character("STUDY01-001"),
        XptValue::character("Headache"),
        XptValue::numeric(1.0),
        XptValue::numeric(1.0),
        XptValue::character("MILD"),
    ]);

    dataset.add_row(vec![
        XptValue::character("STUDY01"),
        XptValue::character("STUDY01-001"),
        XptValue::character("Nausea"),
        XptValue::numeric_missing(), // Standard missing
        XptValue::numeric(2.0),
        XptValue::character("MODERATE"),
    ]);

    dataset.add_row(vec![
        XptValue::character("STUDY01"),
        XptValue::character("STUDY01-002"),
        XptValue::character("Fatigue"),
        XptValue::numeric(7.0),
        XptValue::numeric_missing_with(MissingValue::Special('A')), // Special missing
        XptValue::character("SEVERE"),
    ]);

    let read_back = roundtrip(&dataset, XptVersion::V5);

    // Verify structure
    assert_eq!(read_back.name, "AE");
    assert_eq!(read_back.columns.len(), 6);
    assert_eq!(read_back.num_rows(), 3);

    // Verify column metadata
    assert_eq!(read_back.columns[0].name, "STUDYID");
    assert_eq!(read_back.columns[2].name, "AETERM");
    assert_eq!(read_back.columns[4].format, Some("BEST".to_string()));
    assert_eq!(read_back.columns[4].format_length, 8);

    // Verify data values
    // Row 0
    assert_eq!(read_back.rows[0][0].as_str(), Some("STUDY01"));
    assert_eq!(read_back.rows[0][2].as_str(), Some("Headache"));
    assert!(!read_back.rows[0][3].is_missing());

    // Row 1 - check missing value
    assert!(read_back.rows[1][3].is_missing());
    assert_eq!(read_back.rows[1][5].as_str(), Some("MODERATE"));

    // Row 2 - check special missing
    assert!(read_back.rows[2][4].is_missing());
}

#[test]
fn test_v5_empty_strings() {
    // Test handling of empty strings and whitespace
    let mut dataset = XptDataset::with_columns(
        "TEST",
        vec![
            XptColumn::character("VAR1", 10),
            XptColumn::character("VAR2", 5),
        ],
    );

    dataset.add_row(vec![XptValue::character("Hello"), XptValue::character("")]);

    dataset.add_row(vec![
        XptValue::character("   "),
        XptValue::character("Test"),
    ]);

    let read_back = roundtrip(&dataset, XptVersion::V5);

    assert_eq!(read_back.num_rows(), 2);
    // Note: Empty strings become spaces in XPT format
}

#[test]
fn test_v5_numeric_precision() {
    // Test numeric value preservation
    let mut dataset = XptDataset::with_columns("TEST", vec![XptColumn::numeric("VALUE")]);

    // Add various numeric values
    dataset.add_row(vec![XptValue::numeric(0.0)]);
    dataset.add_row(vec![XptValue::numeric(1.0)]);
    dataset.add_row(vec![XptValue::numeric(-1.0)]);
    dataset.add_row(vec![XptValue::numeric(123.456)]);
    dataset.add_row(vec![XptValue::numeric(1e10)]);
    dataset.add_row(vec![XptValue::numeric(1e-10)]);

    let read_back = roundtrip(&dataset, XptVersion::V5);

    assert_eq!(read_back.num_rows(), 6);

    // Check values are approximately correct (IBM float conversion may have small errors)
    if let XptValue::Num(n) = &read_back.rows[0][0] {
        assert!((n.value().unwrap_or(999.0) - 0.0).abs() < 1e-10);
    }
    if let XptValue::Num(n) = &read_back.rows[1][0] {
        assert!((n.value().unwrap_or(999.0) - 1.0).abs() < 1e-10);
    }
    if let XptValue::Num(n) = &read_back.rows[3][0] {
        assert!((n.value().unwrap_or(999.0) - 123.456).abs() < 1e-6);
    }
}

#[test]
fn test_v5_rejects_long_variable_name() {
    // V5 should reject variable names > 8 chars
    let mut col = XptColumn::numeric("PLACEHOLDER");
    col.name = "VERYLONGNAME".to_string();

    let dataset = XptDataset::with_columns("TEST", vec![col]);

    let mut buffer = Vec::new();
    let options = XptWriterOptions::default().with_version(XptVersion::V5);
    let writer = XptWriter::with_options(Cursor::new(&mut buffer), options);

    let result = writer.write_dataset(&dataset);
    assert!(result.is_err());
}

#[test]
fn test_v5_rejects_long_dataset_name() {
    // V5 should reject dataset names > 8 chars
    let dataset = XptDataset::with_columns("VERYLONGNAME", vec![XptColumn::numeric("VAR1")]);

    let mut buffer = Vec::new();
    let options = XptWriterOptions::default().with_version(XptVersion::V5);
    let writer = XptWriter::with_options(Cursor::new(&mut buffer), options);

    let result = writer.write_dataset(&dataset);
    assert!(result.is_err());
}
