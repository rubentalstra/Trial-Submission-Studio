//! Integration tests for V8 format support.
//!
//! These tests verify that V8 format files can be written and read back correctly,
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
fn test_v8_basic_roundtrip() {
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

    let read_back = roundtrip(&dataset, XptVersion::V8);

    assert_eq!(read_back.name, "DM");
    assert_eq!(read_back.columns.len(), 2);
    assert_eq!(read_back.num_rows(), 1);
}

#[test]
fn test_v8_long_variable_name() {
    // V8 allows variable names up to 32 characters
    let long_name = "VERYLONGVARIABLENAME"; // 20 chars

    let mut col = XptColumn::numeric("PLACEHOLDER");
    col.name = long_name.to_string();
    col.label = Some("A variable with a long name".to_string());

    let mut dataset = XptDataset::with_columns("TEST", vec![col]);
    dataset.add_row(vec![XptValue::numeric(123.456)]);

    let read_back = roundtrip(&dataset, XptVersion::V8);

    assert_eq!(read_back.columns[0].name, long_name);
    assert_eq!(
        read_back.columns[0].label,
        Some("A variable with a long name".to_string())
    );
}

#[test]
fn test_v8_long_dataset_name() {
    // V8 allows dataset names up to 32 characters
    let long_dataset_name = "VERYLONGDATASETNAME"; // 19 chars

    let mut dataset = XptDataset::with_columns(long_dataset_name, vec![XptColumn::numeric("VAR1")]);
    dataset.add_row(vec![XptValue::numeric(1.0)]);

    let read_back = roundtrip(&dataset, XptVersion::V8);

    assert_eq!(read_back.name, long_dataset_name);
}

#[test]
fn test_v8_long_label() {
    // V8 allows labels up to 256 characters
    let long_label = "A".repeat(100); // 100 chars (V5 limit is 40)

    let col = XptColumn::numeric("VAR1").with_label(&long_label);

    let mut dataset = XptDataset::with_columns("TEST", vec![col]);
    dataset.add_row(vec![XptValue::numeric(1.0)]);

    let read_back = roundtrip(&dataset, XptVersion::V8);

    // Long label should be preserved via LABELV8 section
    assert_eq!(read_back.columns[0].label, Some(long_label));
}

#[test]
fn test_v8_multiple_variables_with_long_names() {
    let mut col1 = XptColumn::character("PLACEHOLDER", 20);
    col1.name = "FIRSTLONGVARIABLENAME".to_string();
    col1.label = Some("First variable".to_string());

    let mut col2 = XptColumn::numeric("PLACEHOLDER");
    col2.name = "SECONDLONGVARIABLENAME".to_string();
    col2.label = Some("Second variable".to_string());

    let mut col3 = XptColumn::character("SHORTVAR", 10);
    col3.label = Some("Short name variable".to_string());

    let mut dataset = XptDataset::with_columns("TEST", vec![col1, col2, col3]);
    dataset.add_row(vec![
        XptValue::character("VALUE1"),
        XptValue::numeric(42.0),
        XptValue::character("SHORT"),
    ]);

    let read_back = roundtrip(&dataset, XptVersion::V8);

    assert_eq!(read_back.columns[0].name, "FIRSTLONGVARIABLENAME");
    assert_eq!(read_back.columns[1].name, "SECONDLONGVARIABLENAME");
    assert_eq!(read_back.columns[2].name, "SHORTVAR");
}

#[test]
fn test_v8_mixed_short_and_long_labels() {
    let long_label = "This is a very long label that exceeds the V5 limit of forty characters";
    let short_label = "Short label";

    let col1 = XptColumn::numeric("VAR1").with_label(short_label);
    let col2 = XptColumn::numeric("VAR2").with_label(long_label);
    let col3 = XptColumn::numeric("VAR3").with_label("Another short");

    let mut dataset = XptDataset::with_columns("TEST", vec![col1, col2, col3]);
    dataset.add_row(vec![
        XptValue::numeric(1.0),
        XptValue::numeric(2.0),
        XptValue::numeric(3.0),
    ]);

    let read_back = roundtrip(&dataset, XptVersion::V8);

    assert_eq!(read_back.columns[0].label, Some(short_label.to_string()));
    assert_eq!(read_back.columns[1].label, Some(long_label.to_string()));
    assert_eq!(
        read_back.columns[2].label,
        Some("Another short".to_string())
    );
}

#[test]
fn test_v8_with_format() {
    let col = XptColumn::numeric("STARTDT")
        .with_label("Start Date")
        .with_format("DATE9", 9, 0);

    let mut dataset = XptDataset::with_columns("TEST", vec![col]);
    dataset.add_row(vec![XptValue::numeric(21916.0)]); // Some date value

    let read_back = roundtrip(&dataset, XptVersion::V8);

    assert_eq!(read_back.columns[0].format, Some("DATE9".to_string()));
    assert_eq!(read_back.columns[0].format_length, 9);
    assert_eq!(read_back.columns[0].format_decimals, 0);
}

#[test]
fn test_v8_empty_dataset() {
    let dataset = XptDataset::with_columns(
        "EMPTY",
        vec![
            XptColumn::character("USUBJID", 20),
            XptColumn::numeric("AGE"),
        ],
    );

    let read_back = roundtrip(&dataset, XptVersion::V8);

    assert_eq!(read_back.name, "EMPTY");
    assert_eq!(read_back.columns.len(), 2);
    assert_eq!(read_back.num_rows(), 0);
}

#[test]
fn test_v8_with_missing_values() {
    use sdtm_xpt::MissingValue;

    let mut dataset = XptDataset::with_columns("TEST", vec![XptColumn::numeric("VALUE")]);

    dataset.add_row(vec![XptValue::numeric(1.0)]);
    dataset.add_row(vec![XptValue::numeric_missing()]);
    dataset.add_row(vec![XptValue::numeric_missing_with(MissingValue::Special(
        'A',
    ))]);
    dataset.add_row(vec![XptValue::numeric(2.0)]);

    let read_back = roundtrip(&dataset, XptVersion::V8);

    assert_eq!(read_back.num_rows(), 4);

    // Check values
    let row0 = &read_back.rows[0];
    let row1 = &read_back.rows[1];
    let row2 = &read_back.rows[2];
    let row3 = &read_back.rows[3];

    assert!(!row0[0].is_missing());
    assert!(row1[0].is_missing());
    assert!(row2[0].is_missing());
    assert!(!row3[0].is_missing());
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

#[test]
fn test_v8_accepts_long_variable_name() {
    // V8 should accept variable names up to 32 chars
    let mut col = XptColumn::numeric("PLACEHOLDER");
    col.name = "VERYLONGVARIABLENAME12345".to_string(); // 25 chars

    let mut dataset = XptDataset::with_columns("TEST", vec![col]);
    dataset.add_row(vec![XptValue::numeric(1.0)]);

    let mut buffer = Vec::new();
    let options = XptWriterOptions::default().with_version(XptVersion::V8);
    let writer = XptWriter::with_options(Cursor::new(&mut buffer), options);

    let result = writer.write_dataset(&dataset);
    assert!(result.is_ok());
}

#[test]
fn test_debug_label_section() {
    // Debug test to understand label section behavior
    let long_label = "This is a very long label exceeding forty chars for testing";
    let col = XptColumn::numeric("VAR1").with_label(long_label);
    let mut dataset = XptDataset::with_columns("TEST", vec![col]);
    dataset.add_row(vec![XptValue::numeric(1.0)]);

    let mut buffer = Vec::new();
    let options = XptWriterOptions::default().with_version(XptVersion::V8);

    {
        let writer = XptWriter::with_options(Cursor::new(&mut buffer), options);
        writer.write_dataset(&dataset).unwrap();
    }

    eprintln!(
        "Buffer size: {} bytes ({} records)",
        buffer.len(),
        buffer.len() / 80
    );

    for i in 0..(buffer.len() / 80) {
        let start = i * 80;
        let record = &buffer[start..start + 80];
        let header_check = String::from_utf8_lossy(&record[..48]);
        if header_check.starts_with("HEADER RECORD") {
            eprintln!("Record {}: {}", i, header_check);
        }
    }

    let reader = XptReader::new(Cursor::new(&buffer));
    let read_back = reader.read_dataset().unwrap();
    eprintln!("Read back label: {:?}", read_back.columns[0].label);

    // The test should pass if LABELV8 header is present
    let has_labelv8 = (0..(buffer.len() / 80)).any(|i| {
        let start = i * 80;
        let record = &buffer[start..start + 80];
        record.starts_with(b"HEADER RECORD*******LABELV8")
    });

    assert!(
        has_labelv8,
        "LABELV8 header should be present for long labels"
    );
}

#[test]
fn test_version_autodetection() {
    // Write with V8, read back and verify version is detected correctly
    let mut col = XptColumn::numeric("PLACEHOLDER");
    col.name = "LONGVARIABLENAME".to_string(); // > 8 chars

    let mut dataset = XptDataset::with_columns("TEST", vec![col]);
    dataset.add_row(vec![XptValue::numeric(42.0)]);

    let mut buffer = Vec::new();
    let options = XptWriterOptions::default().with_version(XptVersion::V8);

    {
        let writer = XptWriter::with_options(Cursor::new(&mut buffer), options);
        writer.write_dataset(&dataset).unwrap();
    }

    // Read back - version should be auto-detected
    let reader = XptReader::new(Cursor::new(&buffer));
    let read_back = reader.read_dataset().unwrap();

    // Long name should be preserved (only possible if V8 was detected)
    assert_eq!(read_back.columns[0].name, "LONGVARIABLENAME");
}
