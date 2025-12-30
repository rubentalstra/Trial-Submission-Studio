#![allow(missing_docs)]

use std::fs;
use std::path::PathBuf;

use sdtm_ingest::{build_column_hints, read_csv_table};

fn temp_file(name: &str, contents: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    dir.push(format!("sdtm_ingest_table_{stamp}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(name);
    fs::write(&path, contents).expect("write file");
    path
}

#[test]
fn reads_table_and_builds_hints() {
    let path = temp_file("test.csv", "A,B,C\n1,x,\n2,x,y\n");
    let df = read_csv_table(&path).expect("read csv");

    // Check column names
    let columns: Vec<String> = df
        .get_column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(columns, vec!["A", "B", "C"]);

    // Check row count
    assert_eq!(df.height(), 2);

    let hints = build_column_hints(&df);

    let a = hints.get("A").expect("A hint");
    assert!(a.is_numeric);
    assert!((a.unique_ratio - 1.0).abs() < 1e-6);
    assert!((a.null_ratio - 0.0).abs() < 1e-6);
    assert!(a.label.is_none());

    let b = hints.get("B").expect("B hint");
    assert!(!b.is_numeric);
    assert!((b.unique_ratio - 0.5).abs() < 1e-6);

    let c = hints.get("C").expect("C hint");
    assert!(!c.is_numeric);
    assert!((c.null_ratio - 0.5).abs() < 1e-6);

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn reads_table_with_double_header_edc_format() {
    // EDC export format: Row 0 = labels, Row 1 = variable codes, Row 2+ = data
    let contents = "Label A,Label B,Label C\nVARA,VARB,VARC\n1,x,\n2,y,z\n";
    let path = temp_file("multi.csv", contents);
    let df = read_csv_table(&path).expect("read csv");

    // Should skip the label row and use variable codes as headers
    let columns: Vec<String> = df
        .get_column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(columns, vec!["VARA", "VARB", "VARC"]);

    // Should have 2 data rows (not 3, because variable code row was used as header)
    assert_eq!(df.height(), 2);

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn reads_normal_csv_without_double_header() {
    // Normal CSV: Row 0 = headers (short codes), Row 1+ = data
    let contents = "ID,NAME,AGE\n1,Alice,30\n2,Bob,25\n";
    let path = temp_file("normal.csv", contents);
    let df = read_csv_table(&path).expect("read csv");

    // Headers should remain as-is
    let columns: Vec<String> = df
        .get_column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(columns, vec!["ID", "NAME", "AGE"]);

    // Should have 2 data rows
    assert_eq!(df.height(), 2);

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}
