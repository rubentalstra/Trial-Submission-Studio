//! Tests for streaming CSV reading functionality.

use std::fs;
use std::path::PathBuf;

use sdtm_ingest::{
    FileSizeCategory, IngestOptions, StreamingCsvReader, StreamingOptions, build_column_hints_auto,
    read_csv_table_auto, read_csv_table_auto_with_options, should_use_streaming,
    should_use_streaming_with_threshold,
};

fn temp_file(name: &str, contents: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    dir.push(format!("sdtm_streaming_{stamp}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(name);
    fs::write(&path, contents).expect("write file");
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn streaming_reader_reads_csv() {
    let contents = "A,B,C\n1,x,foo\n2,y,bar\n3,z,baz\n";
    let path = temp_file("stream_test.csv", contents);

    let options = StreamingOptions::default();
    let reader = StreamingCsvReader::new(&path, options).expect("create reader");
    let table = reader.read_as_csv_table().expect("read as csv table");

    assert_eq!(table.headers, vec!["A", "B", "C"]);
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0], vec!["1", "x", "foo"]);

    cleanup(&path);
}

#[test]
fn streaming_reader_samples_rows() {
    // Create a larger dataset
    let mut contents = String::from("ID,VALUE\n");
    for i in 1..=100 {
        contents.push_str(&format!("{},{}\n", i, i * 10));
    }
    let path = temp_file("sample_test.csv", &contents);

    let options = StreamingOptions::default().with_sample_size(10);
    let reader = StreamingCsvReader::new(&path, options).expect("create reader");
    let sample = reader.sample_as_csv_table().expect("sample rows");

    // Should only have sampled 10 rows
    assert_eq!(sample.headers, vec!["ID", "VALUE"]);
    assert_eq!(sample.rows.len(), 10);
    assert_eq!(sample.rows[0], vec!["1", "10"]);

    cleanup(&path);
}

#[test]
fn streaming_reader_builds_column_hints() {
    let contents = "ID,VALUE,NAME\n1,100,Alice\n2,200,Bob\n3,,Charlie\n";
    let path = temp_file("hints_test.csv", contents);

    let options = StreamingOptions::default();
    let reader = StreamingCsvReader::new(&path, options).expect("create reader");
    let hints = reader.build_column_hints().expect("build hints");

    assert!(hints.contains_key("ID"));
    assert!(hints.contains_key("VALUE"));
    assert!(hints.contains_key("NAME"));

    let id_hint = hints.get("ID").unwrap();
    assert!(id_hint.is_numeric);

    let name_hint = hints.get("NAME").unwrap();
    assert!(!name_hint.is_numeric);

    cleanup(&path);
}

#[test]
fn file_size_category_detection() {
    let contents = "A,B\n1,2\n"; // Very small file
    let path = temp_file("size_test.csv", contents);

    let category = FileSizeCategory::from_path(&path).expect("get category");
    assert_eq!(category, FileSizeCategory::Small);

    cleanup(&path);
}

#[test]
fn should_use_streaming_threshold() {
    let contents = "A,B\n1,2\n"; // ~10 bytes
    let path = temp_file("threshold_test.csv", contents);

    // With high threshold, should not use streaming
    assert!(!should_use_streaming_with_threshold(&path, 1000));

    // With low threshold, should use streaming
    assert!(should_use_streaming_with_threshold(&path, 5));

    // Default threshold (10 MB) should not trigger for small files
    assert!(!should_use_streaming(&path));

    cleanup(&path);
}

#[test]
fn read_csv_table_auto_small_file() {
    let contents = "A,B,C\n1,x,foo\n2,y,bar\n";
    let path = temp_file("auto_small.csv", contents);

    let table = read_csv_table_auto(&path).expect("read auto");
    assert_eq!(table.headers, vec!["A", "B", "C"]);
    assert_eq!(table.rows.len(), 2);

    cleanup(&path);
}

#[test]
fn read_csv_table_auto_with_options_works() {
    let contents = "garbage\nA,B,C\n1,x,foo\n2,y,bar\n";
    let path = temp_file("auto_options.csv", contents);

    let options = IngestOptions::with_header_row(1);
    let table = read_csv_table_auto_with_options(&path, &options).expect("read auto with options");

    // Should respect header row option
    assert_eq!(table.headers, vec!["A", "B", "C"]);
    assert_eq!(table.rows.len(), 2);

    cleanup(&path);
}

#[test]
fn build_column_hints_auto_small_file() {
    let contents = "ID,VALUE\n1,100\n2,200\n3,300\n";
    let path = temp_file("hints_auto.csv", contents);

    let hints = build_column_hints_auto(&path).expect("build hints auto");

    assert!(hints.contains_key("ID"));
    assert!(hints.contains_key("VALUE"));

    let id_hint = hints.get("ID").unwrap();
    assert!(id_hint.is_numeric);

    cleanup(&path);
}

#[test]
fn streaming_options_builder() {
    let options = StreamingOptions::default()
        .with_sample_size(500)
        .with_parallel(false)
        .with_chunk_size(25000)
        .with_low_memory(true);

    assert_eq!(options.sample_size, 500);
    assert!(!options.parallel);
    assert_eq!(options.chunk_size, 25000);
    assert!(options.low_memory);
}

#[test]
fn file_size_category_recommended_options() {
    let small_opts = FileSizeCategory::Small.recommended_options();
    assert_eq!(small_opts.sample_size, 500);
    assert!(!small_opts.parallel);

    let large_opts = FileSizeCategory::Large.recommended_options();
    assert_eq!(large_opts.sample_size, 2000);
    assert!(large_opts.parallel);

    let very_large_opts = FileSizeCategory::VeryLarge.recommended_options();
    assert!(very_large_opts.low_memory);
}
