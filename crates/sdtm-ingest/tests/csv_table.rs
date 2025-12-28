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
