use std::fs;
use std::path::PathBuf;

use sdtm_ingest::{
    IngestOptions, SchemaHint, build_column_hints, read_csv_table, read_csv_table_with_options,
};

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
    let table = read_csv_table(&path).expect("read csv");
    assert_eq!(table.headers, vec!["A", "B", "C"]);
    let hints = build_column_hints(&table);

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
fn reads_table_with_multiple_headers() {
    let contents = "Label A,Label B,Label C\nA,B,C\n1,x,\n2,y,z\n";
    let path = temp_file("multi.csv", contents);
    let table = read_csv_table(&path).expect("read csv");
    assert_eq!(table.headers, vec!["A", "B", "C"]);
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.rows[0], vec!["1", "x", ""]);
    assert_eq!(table.rows[1], vec!["2", "y", "z"]);
    assert_eq!(
        table.labels,
        Some(vec![
            "Label A".to_string(),
            "Label B".to_string(),
            "Label C".to_string()
        ])
    );

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn reads_table_with_explicit_header_row() {
    // Row 0: garbage, Row 1: labels, Row 2: headers, Row 3+: data
    let contents = "garbage,row,here\nLabel A,Label B,Label C\nA,B,C\n1,x,\n2,y,z\n";
    let path = temp_file("explicit_header.csv", contents);

    // Use explicit header row index (row 2, 0-indexed)
    let options = IngestOptions::with_header_row(2).label_row(1);
    let table = read_csv_table_with_options(&path, &options).expect("read csv");

    assert_eq!(table.headers, vec!["A", "B", "C"]);
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.rows[0], vec!["1", "x", ""]);
    assert_eq!(table.rows[1], vec!["2", "y", "z"]);
    assert_eq!(
        table.labels,
        Some(vec![
            "Label A".to_string(),
            "Label B".to_string(),
            "Label C".to_string()
        ])
    );

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn reads_table_with_explicit_schema() {
    // File has no headers - just data
    let contents = "1,x,foo\n2,y,bar\n3,z,baz\n";
    let path = temp_file("schema_only.csv", contents);

    let options = IngestOptions::with_schema(
        vec!["COL1".to_string(), "COL2".to_string(), "COL3".to_string()],
        Some(vec![
            "Column One".to_string(),
            "Column Two".to_string(),
            "Column Three".to_string(),
        ]),
    );
    let table = read_csv_table_with_options(&path, &options).expect("read csv");

    assert_eq!(table.headers, vec!["COL1", "COL2", "COL3"]);
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0], vec!["1", "x", "foo"]);
    assert_eq!(table.rows[1], vec!["2", "y", "bar"]);
    assert_eq!(table.rows[2], vec!["3", "z", "baz"]);
    assert_eq!(
        table.labels,
        Some(vec![
            "Column One".to_string(),
            "Column Two".to_string(),
            "Column Three".to_string()
        ])
    );

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn default_ingest_options_uses_heuristics() {
    let contents = "A,B,C\n1,x,\n2,y,z\n";
    let path = temp_file("default_opts.csv", contents);

    let options = IngestOptions::default();
    let table = read_csv_table_with_options(&path, &options).expect("read csv");

    assert_eq!(table.headers, vec!["A", "B", "C"]);
    assert_eq!(table.rows.len(), 2);

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn ingest_options_serialize_deserialize() {
    let options = IngestOptions {
        header_row_index: Some(2),
        label_row_index: Some(1),
        schema: Some(SchemaHint {
            headers: vec!["A".to_string(), "B".to_string()],
            labels: Some(vec!["Label A".to_string(), "Label B".to_string()]),
        }),
        max_header_scan_rows: Some(20),
    };

    let json = serde_json::to_string(&options).expect("serialize");
    let parsed: IngestOptions = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.header_row_index, Some(2));
    assert_eq!(parsed.label_row_index, Some(1));
    assert_eq!(parsed.max_header_scan_rows, Some(20));
    assert!(parsed.schema.is_some());
    let schema = parsed.schema.unwrap();
    assert_eq!(schema.headers, vec!["A", "B"]);
    assert_eq!(
        schema.labels,
        Some(vec!["Label A".to_string(), "Label B".to_string()])
    );
}
