//! End-to-end roundtrip tests for XPT V5 and V8 formats.
//!
//! These tests verify that data written to XPT files can be read back
//! with full fidelity for both format versions.

use std::path::Path;
use tempfile::tempdir;

use tss_xpt::{
    MissingValue, XptColumn, XptDataset, XptValue, XptVersion, XptWriterOptions, read_xpt,
    write_xpt_with_options,
};

// ============================================================================
// Helper functions
// ============================================================================

fn create_basic_dataset(name: &str) -> XptDataset {
    let mut dataset = XptDataset::with_columns(
        name,
        vec![
            XptColumn::character("STUDYID", 12).with_label("Study Identifier"),
            XptColumn::character("USUBJID", 20).with_label("Unique Subject ID"),
            XptColumn::numeric("AGE").with_label("Age in Years"),
            XptColumn::numeric("HEIGHT").with_label("Height (cm)"),
            XptColumn::character("SEX", 1).with_label("Sex"),
        ],
    );

    dataset.add_row(vec![
        XptValue::character("STUDY001"),
        XptValue::character("STUDY001-001"),
        XptValue::numeric(25.0),
        XptValue::numeric(175.5),
        XptValue::character("M"),
    ]);

    dataset.add_row(vec![
        XptValue::character("STUDY001"),
        XptValue::character("STUDY001-002"),
        XptValue::numeric(32.0),
        XptValue::numeric(162.0),
        XptValue::character("F"),
    ]);

    dataset.add_row(vec![
        XptValue::character("STUDY001"),
        XptValue::character("STUDY001-003"),
        XptValue::numeric(45.0),
        XptValue::numeric(180.0),
        XptValue::character("M"),
    ]);

    dataset
}

fn create_dataset_with_missing_values(name: &str) -> XptDataset {
    let mut dataset = XptDataset::with_columns(
        name,
        vec![
            XptColumn::character("SUBJID", 10),
            XptColumn::numeric("VALUE1"),
            XptColumn::numeric("VALUE2"),
            XptColumn::character("NOTE", 20),
        ],
    );

    // Row with all values present
    dataset.add_row(vec![
        XptValue::character("SUBJ001"),
        XptValue::numeric(100.0),
        XptValue::numeric(200.0),
        XptValue::character("Complete"),
    ]);

    // Row with standard missing (.)
    dataset.add_row(vec![
        XptValue::character("SUBJ002"),
        XptValue::numeric_missing(),
        XptValue::numeric(150.0),
        XptValue::character("Missing val1"),
    ]);

    // Row with special missing (.A)
    dataset.add_row(vec![
        XptValue::character("SUBJ003"),
        XptValue::numeric_missing_with(MissingValue::Special('A')),
        XptValue::numeric_missing_with(MissingValue::Special('Z')),
        XptValue::character("Special missing"),
    ]);

    // Row with underscore missing (._)
    dataset.add_row(vec![
        XptValue::character("SUBJ004"),
        XptValue::numeric_missing_with(MissingValue::Underscore),
        XptValue::numeric(999.0),
        XptValue::character("Underscore"),
    ]);

    // Row with empty character
    dataset.add_row(vec![
        XptValue::character("SUBJ005"),
        XptValue::numeric(0.0),
        XptValue::numeric(-123.456),
        XptValue::character(""),
    ]);

    dataset
}

fn create_dataset_with_edge_cases(name: &str) -> XptDataset {
    let mut dataset = XptDataset::with_columns(
        name,
        vec![
            XptColumn::character("ID", 8),
            XptColumn::numeric("SMALL"),
            XptColumn::numeric("LARGE"),
            XptColumn::numeric("PRECISE"),
            XptColumn::character("LONGTEXT", 200),
        ],
    );

    // Very small numbers
    dataset.add_row(vec![
        XptValue::character("EDGE001"),
        XptValue::numeric(0.000001),
        XptValue::numeric(1e15),
        XptValue::numeric(std::f64::consts::PI),
        XptValue::character("A".repeat(200)),
    ]);

    // Negative numbers
    dataset.add_row(vec![
        XptValue::character("EDGE002"),
        XptValue::numeric(-0.000001),
        XptValue::numeric(-1e15),
        XptValue::numeric(-std::f64::consts::E),
        XptValue::character("Short"),
    ]);

    // Zero
    dataset.add_row(vec![
        XptValue::character("EDGE003"),
        XptValue::numeric(0.0),
        XptValue::numeric(-0.0),
        XptValue::numeric(0.0),
        XptValue::character("Zero values"),
    ]);

    // Max/min safe integers
    dataset.add_row(vec![
        XptValue::character("EDGE004"),
        XptValue::numeric(9007199254740992.0), // 2^53
        XptValue::numeric(-9007199254740992.0),
        XptValue::numeric(1.0),
        XptValue::character("Large integers"),
    ]);

    dataset
}

fn create_v8_long_names_dataset(name: &str) -> XptDataset {
    // V8 allows longer names (up to 32 chars) and labels (up to 256 chars)
    let mut dataset = XptDataset::with_columns(
        name,
        vec![
            XptColumn::character("VERYLONGVARIABLENAME123", 50)
                .with_label("This is a very long label that exceeds the V5 limit of 40 characters and goes all the way to the V8 limit"),
            XptColumn::numeric("ANOTHERLONGNAME456789")
                .with_label("Another extended label for V8 format testing purposes"),
            XptColumn::character("SHORTNAME", 10)
                .with_label("Short"),
        ],
    );

    dataset.add_row(vec![
        XptValue::character("Value for long variable name column"),
        XptValue::numeric(123.456),
        XptValue::character("Short"),
    ]);

    dataset.add_row(vec![
        XptValue::character("Another value"),
        XptValue::numeric(789.012),
        XptValue::character("Val2"),
    ]);

    dataset
}

fn assert_datasets_equal(original: &XptDataset, loaded: &XptDataset) {
    assert_eq!(
        original.name.to_uppercase(),
        loaded.name.to_uppercase(),
        "Dataset names should match"
    );
    assert_eq!(
        original.num_columns(),
        loaded.num_columns(),
        "Column counts should match"
    );
    assert_eq!(
        original.num_rows(),
        loaded.num_rows(),
        "Row counts should match"
    );

    // Check columns
    for (i, (orig_col, loaded_col)) in original
        .columns
        .iter()
        .zip(loaded.columns.iter())
        .enumerate()
    {
        assert_eq!(
            orig_col.name.to_uppercase(),
            loaded_col.name.to_uppercase(),
            "Column {} names should match",
            i
        );
        assert_eq!(
            orig_col.data_type, loaded_col.data_type,
            "Column {} types should match",
            i
        );
        assert_eq!(
            orig_col.length, loaded_col.length,
            "Column {} lengths should match",
            i
        );
    }

    // Check values
    for row_idx in 0..original.num_rows() {
        for col_idx in 0..original.num_columns() {
            let orig_val = original.value(row_idx, col_idx).unwrap();
            let loaded_val = loaded.value(row_idx, col_idx).unwrap();

            match (orig_val, loaded_val) {
                (XptValue::Char(a), XptValue::Char(b)) => {
                    // Character values are right-trimmed on read
                    assert_eq!(
                        a.trim_end(),
                        b.trim_end(),
                        "Character value mismatch at row {}, col {}",
                        row_idx,
                        col_idx
                    );
                }
                (XptValue::Num(a), XptValue::Num(b)) => {
                    match (a.value(), b.value()) {
                        (Some(av), Some(bv)) => {
                            // Allow small floating point differences due to IBM float conversion
                            let diff = (av - bv).abs();
                            let tolerance = av.abs() * 1e-10 + 1e-15;
                            assert!(
                                diff <= tolerance,
                                "Numeric value mismatch at row {}, col {}: {} vs {} (diff: {})",
                                row_idx,
                                col_idx,
                                av,
                                bv,
                                diff
                            );
                        }
                        (None, None) => {
                            // Both are missing - check missing type matches
                            assert_eq!(
                                a.missing_type(),
                                b.missing_type(),
                                "Missing value type mismatch at row {}, col {}",
                                row_idx,
                                col_idx
                            );
                        }
                        _ => panic!(
                            "Missing/present mismatch at row {}, col {}",
                            row_idx, col_idx
                        ),
                    }
                }
                _ => panic!("Value type mismatch at row {}, col {}", row_idx, col_idx),
            }
        }
    }
}

// ============================================================================
// V5 Roundtrip Tests
// ============================================================================

#[test]
fn test_v5_roundtrip_basic() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("basic_v5.xpt");

    let original = create_basic_dataset("DM");

    // Write V5
    let options = XptWriterOptions::default().with_version(XptVersion::V5);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_datasets_equal(&original, &loaded);
}

#[test]
fn test_v5_roundtrip_missing_values() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("missing_v5.xpt");

    let original = create_dataset_with_missing_values("TEST");

    // Write V5
    let options = XptWriterOptions::default().with_version(XptVersion::V5);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_datasets_equal(&original, &loaded);
}

#[test]
fn test_v5_roundtrip_edge_cases() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("edge_v5.xpt");

    let original = create_dataset_with_edge_cases("EDGE");

    // Write V5
    let options = XptWriterOptions::default().with_version(XptVersion::V5);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_datasets_equal(&original, &loaded);
}

#[test]
fn test_v5_roundtrip_empty_dataset() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("empty_v5.xpt");

    let original = XptDataset::with_columns(
        "EMPTY",
        vec![XptColumn::character("COL1", 10), XptColumn::numeric("COL2")],
    );
    // No rows added

    // Write V5
    let options = XptWriterOptions::default().with_version(XptVersion::V5);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_eq!(loaded.num_rows(), 0);
    assert_eq!(loaded.num_columns(), 2);
}

#[test]
fn test_v5_roundtrip_single_row() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("single_v5.xpt");

    let mut original = XptDataset::with_columns(
        "SINGLE",
        vec![XptColumn::character("ID", 5), XptColumn::numeric("VAL")],
    );
    original.add_row(vec![XptValue::character("ONE"), XptValue::numeric(42.0)]);

    // Write V5
    let options = XptWriterOptions::default().with_version(XptVersion::V5);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_datasets_equal(&original, &loaded);
}

#[test]
fn test_v5_roundtrip_many_columns() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("manycol_v5.xpt");

    // Create dataset with 50 columns
    let columns: Vec<XptColumn> = (0..50)
        .map(|i| {
            if i % 2 == 0 {
                XptColumn::numeric(format!("N{:02}", i))
            } else {
                XptColumn::character(format!("C{:02}", i), 10)
            }
        })
        .collect();

    let mut original = XptDataset::with_columns("WIDE", columns);

    // Add a few rows
    for row_num in 0..5 {
        let values: Vec<XptValue> = (0..50)
            .map(|i| {
                if i % 2 == 0 {
                    XptValue::numeric((row_num * 50 + i) as f64)
                } else {
                    XptValue::character(format!("R{}C{}", row_num, i))
                }
            })
            .collect();
        original.add_row(values);
    }

    // Write V5
    let options = XptWriterOptions::default().with_version(XptVersion::V5);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_datasets_equal(&original, &loaded);
}

#[test]
fn test_v5_roundtrip_many_rows() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("manyrow_v5.xpt");

    let mut original = XptDataset::with_columns(
        "TALL",
        vec![
            XptColumn::character("ID", 10),
            XptColumn::numeric("SEQ"),
            XptColumn::numeric("VALUE"),
        ],
    );

    // Add 1000 rows
    for i in 0..1000 {
        original.add_row(vec![
            XptValue::character(format!("SUBJ{:04}", i)),
            XptValue::numeric(i as f64),
            XptValue::numeric((i as f64) * 1.5),
        ]);
    }

    // Write V5
    let options = XptWriterOptions::default().with_version(XptVersion::V5);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_datasets_equal(&original, &loaded);
}

// ============================================================================
// V8 Roundtrip Tests
// ============================================================================

#[test]
fn test_v8_roundtrip_basic() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("basic_v8.xpt");

    let original = create_basic_dataset("DM");

    // Write V8
    let options = XptWriterOptions::default().with_version(XptVersion::V8);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_datasets_equal(&original, &loaded);
}

#[test]
fn test_v8_roundtrip_missing_values() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("missing_v8.xpt");

    let original = create_dataset_with_missing_values("TEST");

    // Write V8
    let options = XptWriterOptions::default().with_version(XptVersion::V8);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_datasets_equal(&original, &loaded);
}

#[test]
fn test_v8_roundtrip_edge_cases() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("edge_v8.xpt");

    let original = create_dataset_with_edge_cases("EDGE");

    // Write V8
    let options = XptWriterOptions::default().with_version(XptVersion::V8);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_datasets_equal(&original, &loaded);
}

#[test]
fn test_v8_roundtrip_long_names() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("longnames_v8.xpt");

    let original = create_v8_long_names_dataset("LONGNAMES");

    // Write V8
    let options = XptWriterOptions::default().with_version(XptVersion::V8);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify structure
    assert_eq!(loaded.num_columns(), 3);
    assert_eq!(loaded.num_rows(), 2);

    // V8 should preserve long names
    assert_eq!(
        loaded.columns[0].name.to_uppercase(),
        "VERYLONGVARIABLENAME123"
    );
    assert_eq!(
        loaded.columns[1].name.to_uppercase(),
        "ANOTHERLONGNAME456789"
    );

    // Verify data
    assert_datasets_equal(&original, &loaded);
}

#[test]
fn test_v8_roundtrip_empty_dataset() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("empty_v8.xpt");

    let original = XptDataset::with_columns(
        "EMPTYV8",
        vec![
            XptColumn::character("LONGCOLUMNNAME", 10),
            XptColumn::numeric("ANOTHERLONGNAME"),
        ],
    );

    // Write V8
    let options = XptWriterOptions::default().with_version(XptVersion::V8);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_eq!(loaded.num_rows(), 0);
    assert_eq!(loaded.num_columns(), 2);
}

#[test]
fn test_v8_roundtrip_many_rows() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("manyrow_v8.xpt");

    let mut original = XptDataset::with_columns(
        "TALLV8",
        vec![
            XptColumn::character("SUBJECTIDENTIFIER", 20),
            XptColumn::numeric("SEQUENCENUMBER"),
            XptColumn::numeric("MEASUREMENTVALUE"),
        ],
    );

    // Add 1000 rows
    for i in 0..1000 {
        original.add_row(vec![
            XptValue::character(format!("SUBJECT{:06}", i)),
            XptValue::numeric(i as f64),
            XptValue::numeric((i as f64).sqrt()),
        ]);
    }

    // Write V8
    let options = XptWriterOptions::default().with_version(XptVersion::V8);
    write_xpt_with_options(&path, &original, &options).unwrap();

    // Read back
    let loaded = read_xpt(&path).unwrap();

    // Verify
    assert_datasets_equal(&original, &loaded);
}

// ============================================================================
// Cross-version Tests
// ============================================================================

#[test]
fn test_v5_and_v8_produce_same_data() {
    let dir = tempdir().unwrap();
    let v5_path = dir.path().join("compare_v5.xpt");
    let v8_path = dir.path().join("compare_v8.xpt");

    // Use dataset with short names (compatible with both versions)
    let original = create_basic_dataset("DM");

    // Write V5
    let v5_options = XptWriterOptions::default().with_version(XptVersion::V5);
    write_xpt_with_options(&v5_path, &original, &v5_options).unwrap();

    // Write V8
    let v8_options = XptWriterOptions::default().with_version(XptVersion::V8);
    write_xpt_with_options(&v8_path, &original, &v8_options).unwrap();

    // Read both
    let loaded_v5 = read_xpt(&v5_path).unwrap();
    let loaded_v8 = read_xpt(&v8_path).unwrap();

    // Both should have same data
    assert_datasets_equal(&loaded_v5, &loaded_v8);
}

// ============================================================================
// Real File Tests
// ============================================================================

#[test]
fn test_read_real_xpt_file() {
    let path = Path::new("tests/data/lb.xpt");
    if !path.exists() {
        eprintln!("Skipping test_read_real_xpt_file: lb.xpt not found");
        return;
    }

    let dataset = read_xpt(path).unwrap();

    // Basic sanity checks
    assert!(!dataset.name.is_empty(), "Dataset should have a name");
    assert!(dataset.num_columns() > 0, "Dataset should have columns");
    assert!(dataset.num_rows() > 0, "Dataset should have rows");

    // Print some info for debugging
    println!("Dataset: {}", dataset.name);
    println!("Columns: {}", dataset.num_columns());
    println!("Rows: {}", dataset.num_rows());
    for (i, col) in dataset.columns.iter().take(5).enumerate() {
        println!("  Column {}: {} ({:?})", i, col.name, col.data_type);
    }
}

#[test]
fn test_roundtrip_real_xpt_file() {
    let path = Path::new("tests/data/lb.xpt");
    if !path.exists() {
        eprintln!("Skipping test_roundtrip_real_xpt_file: lb.xpt not found");
        return;
    }

    let dir = tempdir().unwrap();
    let output_path = dir.path().join("lb_roundtrip.xpt");

    // Read original
    let original = read_xpt(path).unwrap();

    // Write back (same version as detected)
    write_xpt_with_options(&output_path, &original, &XptWriterOptions::default()).unwrap();

    // Read again
    let reloaded = read_xpt(&output_path).unwrap();

    // Verify
    assert_datasets_equal(&original, &reloaded);
}
