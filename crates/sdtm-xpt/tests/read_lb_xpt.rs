//! Test for reading the lb.xpt test data file.
//!
//! This test validates that the XPT reader correctly parses a real CDISC
//! laboratory results dataset with all its metadata and observations.

use std::path::Path;

use sdtm_xpt::{XptType, read_xpt};

/// Path to the test data file
fn lb_xpt_path() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/lb.xpt"))
}

#[test]
fn test_read_lb_xpt_exists() {
    let path = lb_xpt_path();
    assert!(path.exists(), "Test file not found: {}", path.display());
}

#[test]
fn test_read_lb_xpt_basic_structure() {
    let dataset = read_xpt(lb_xpt_path()).expect("Failed to read lb.xpt");

    // Verify basic dataset properties
    assert_eq!(dataset.name, "LB", "Dataset name should be LB");
    assert_eq!(
        dataset.label.as_deref(),
        Some("Laboratory Test Results"),
        "Dataset label mismatch"
    );

    // Print basic info for debugging
    println!("Dataset: {} - {:?}", dataset.name, dataset.label);
    println!("Number of columns: {}", dataset.columns.len());
    println!("Number of rows: {}", dataset.num_rows());
}

#[test]
fn test_read_lb_xpt_column_metadata() {
    let dataset = read_xpt(lb_xpt_path()).expect("Failed to read lb.xpt");

    // Based on the hex dump, we expect these columns:
    // STUDYID, DOMAIN, USUBJID, LBSEQ, LBTESTCD, LBTEST, LBCAT, LBORRES, LBORRESU, etc.

    println!("\n=== Column Metadata ===");
    for (i, col) in dataset.columns.iter().enumerate() {
        println!(
            "[{:2}] {:12} ({:4}, len={:3}) label={:?}",
            i, col.name, col.data_type, col.length, col.label
        );
        if col.format.is_some() || col.format_length > 0 || col.format_decimals > 0 {
            println!(
                "     format: {:?} ({}, {})",
                col.format, col.format_length, col.format_decimals
            );
        }
        if col.informat.is_some() || col.informat_length > 0 || col.informat_decimals > 0 {
            println!(
                "     informat: {:?} ({}, {})",
                col.informat, col.informat_length, col.informat_decimals
            );
        }
        println!("     justification: {:?}", col.justification);
    }

    // Verify expected column count (from NAMESTR header: 23 variables)
    assert_eq!(
        dataset.columns.len(),
        23,
        "Expected 23 columns in LB dataset"
    );

    // Verify specific columns
    let studyid = dataset
        .column_by_name("STUDYID")
        .expect("STUDYID column missing");
    assert_eq!(studyid.data_type, XptType::Char);
    assert_eq!(studyid.length, 12);
    assert_eq!(studyid.label.as_deref(), Some("Study Identifier"));

    let domain = dataset
        .column_by_name("DOMAIN")
        .expect("DOMAIN column missing");
    assert_eq!(domain.data_type, XptType::Char);
    assert_eq!(domain.length, 2);
    assert_eq!(domain.label.as_deref(), Some("Domain Abbreviation"));

    let usubjid = dataset
        .column_by_name("USUBJID")
        .expect("USUBJID column missing");
    assert_eq!(usubjid.data_type, XptType::Char);
    assert_eq!(usubjid.length, 8);
    assert_eq!(usubjid.label.as_deref(), Some("Unique Subject Identifier"));

    let lbseq = dataset
        .column_by_name("LBSEQ")
        .expect("LBSEQ column missing");
    assert_eq!(lbseq.data_type, XptType::Num);
    assert_eq!(lbseq.length, 8);
    assert_eq!(lbseq.label.as_deref(), Some("Sequence Number"));

    let lbstresn = dataset
        .column_by_name("LBSTRESN")
        .expect("LBSTRESN column missing");
    assert_eq!(lbstresn.data_type, XptType::Num);
    assert_eq!(
        lbstresn.label.as_deref(),
        Some("Numeric Result/Finding in Standard Units")
    );
}

#[test]
fn test_read_lb_xpt_data_values() {
    let dataset = read_xpt(lb_xpt_path()).expect("Failed to read lb.xpt");

    println!("\n=== Sample Data (first 5 rows) ===");
    for (row_idx, row) in dataset.rows.iter().take(5).enumerate() {
        println!("Row {}: ", row_idx);
        for (col_idx, value) in row.iter().enumerate() {
            let col_name = &dataset.columns[col_idx].name;
            println!("  {} = {:?}", col_name, value);
        }
        println!();
    }

    // Verify we have rows
    assert!(dataset.num_rows() > 0, "Dataset should have rows");

    // Check first row values
    if let Some(row) = dataset.row(0) {
        let studyid_idx = dataset.column_index("STUDYID").unwrap();
        let domain_idx = dataset.column_index("DOMAIN").unwrap();
        let lbtestcd_idx = dataset.column_index("LBTESTCD").unwrap();

        assert_eq!(row[studyid_idx].as_str(), Some("CDISCPILOT01"));
        assert_eq!(row[domain_idx].as_str(), Some("LB"));
        // From hex dump, first test is ALB (Albumin)
        assert_eq!(row[lbtestcd_idx].as_str(), Some("ALB"));
    }
}

#[test]
fn test_read_lb_xpt_format_specifications() {
    let dataset = read_xpt(lb_xpt_path()).expect("Failed to read lb.xpt");

    println!("\n=== Format Specifications ===");

    // Check columns with format specifications
    let cols_with_formats: Vec<_> = dataset
        .columns
        .iter()
        .filter(|c| c.format.is_some() || c.format_length > 0)
        .collect();

    for col in &cols_with_formats {
        println!(
            "{}: format={:?} len={} dec={}",
            col.name, col.format, col.format_length, col.format_decimals
        );
    }

    // Check columns with informat specifications
    let cols_with_informats: Vec<_> = dataset
        .columns
        .iter()
        .filter(|c| c.informat.is_some() || c.informat_length > 0)
        .collect();

    println!("\n=== Informat Specifications ===");
    for col in &cols_with_informats {
        println!(
            "{}: informat={:?} len={} dec={}",
            col.name, col.informat, col.informat_length, col.informat_decimals
        );
    }
}

#[test]
fn test_read_lb_xpt_numeric_values() {
    let dataset = read_xpt(lb_xpt_path()).expect("Failed to read lb.xpt");

    let lbstresn_idx = dataset
        .column_index("LBSTRESN")
        .expect("LBSTRESN column missing");

    println!("\n=== Numeric Values (LBSTRESN, first 10) ===");
    for (row_idx, row) in dataset.rows.iter().take(10).enumerate() {
        let value = &row[lbstresn_idx];
        println!("Row {}: {:?}", row_idx, value);

        // Check for missing values
        if value.is_missing() {
            if let Some(num_val) = value.as_numeric() {
                println!("  -> Missing value type: {:?}", num_val.missing_type());
            }
        } else if let Some(num) = value.as_f64() {
            println!("  -> Numeric value: {}", num);
        }
    }

    // Verify we can read numeric values correctly
    // From hex dump, first row ALB has value around 39 g/L (39 in standard units)
    if let Some(row) = dataset.row(0) {
        let value = &row[lbstresn_idx];
        if let Some(num) = value.as_f64() {
            // Albumin 39 g/L
            assert!(
                (num - 39.0).abs() < 0.1,
                "Expected LBSTRESN ~39.0, got {}",
                num
            );
        }
    }
}

#[test]
fn test_read_lb_xpt_missing_values() {
    let dataset = read_xpt(lb_xpt_path()).expect("Failed to read lb.xpt");

    // Find numeric columns and count missing values
    let numeric_cols: Vec<_> = dataset
        .columns
        .iter()
        .enumerate()
        .filter(|(_, c)| c.data_type == XptType::Num)
        .collect();

    println!("\n=== Missing Values Summary ===");
    for (col_idx, col) in &numeric_cols {
        let missing_count = dataset
            .rows
            .iter()
            .filter(|row| row[*col_idx].is_missing())
            .count();

        if missing_count > 0 {
            println!("{}: {} missing values", col.name, missing_count);
        }
    }
}

#[test]
fn test_read_lb_xpt_observation_length() {
    let dataset = read_xpt(lb_xpt_path()).expect("Failed to read lb.xpt");

    let obs_len = dataset.observation_length();
    println!("\n=== Observation Info ===");
    println!("Observation length: {} bytes", obs_len);
    println!("Total rows: {}", dataset.num_rows());

    // Observation length should be sum of all column lengths
    let computed_len: usize = dataset.columns.iter().map(|c| c.length as usize).sum();
    assert_eq!(obs_len, computed_len);
}

#[test]
fn test_read_lb_xpt_column_positions() {
    let dataset = read_xpt(lb_xpt_path()).expect("Failed to read lb.xpt");

    println!("\n=== Column Positions ===");
    let mut pos = 0usize;
    for col in &dataset.columns {
        println!(
            "{:12}: offset {:4}, length {:3} (ends at {:4})",
            col.name,
            pos,
            col.length,
            pos + col.length as usize
        );
        pos += col.length as usize;
    }
}
