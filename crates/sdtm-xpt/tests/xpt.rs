use std::fs;
use std::path::PathBuf;

use sdtm_xpt::{MissingNumeric, XptColumn, XptDataset, XptType, XptValue, XptWriterOptions};
use sdtm_xpt::{read_xpt, write_xpt};

fn temp_file(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    dir.push(format!("sdtm_xpt_{stamp}_{name}.xpt"));
    dir
}

#[test]
fn reads_dm_xpt() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/validation/data/xpt/dm.xpt");
    let dataset = read_xpt(&path).expect("read dm");
    assert_eq!(dataset.name, "DM");
    assert_eq!(dataset.label.as_deref(), Some("Demographics"));
    assert_eq!(dataset.columns.len(), 26);
    let age = dataset
        .columns
        .iter()
        .find(|col| col.name == "AGE")
        .expect("AGE column");
    assert_eq!(age.data_type, XptType::Num);
    assert_eq!(age.length, 8);
    let first = dataset.rows.first().expect("row");
    let study_idx = dataset
        .columns
        .iter()
        .position(|col| col.name == "STUDYID")
        .expect("STUDYID");
    let age_idx = dataset
        .columns
        .iter()
        .position(|col| col.name == "AGE")
        .expect("AGE");
    assert_eq!(first[study_idx], XptValue::Char("CDISCPILOT01".to_string()));
    assert_eq!(first[age_idx], XptValue::Num(Some(84.0)));
}

#[test]
fn writes_and_reads_roundtrip() {
    let path = temp_file("roundtrip");
    let dataset = XptDataset {
        name: "TEST".to_string(),
        label: Some("Test Dataset".to_string()),
        columns: vec![
            XptColumn {
                name: "SUBJID".to_string(),
                label: Some("Subject Identifier".to_string()),
                data_type: XptType::Char,
                length: 8,
            },
            XptColumn {
                name: "VALUE".to_string(),
                label: Some("Value".to_string()),
                data_type: XptType::Num,
                length: 8,
            },
        ],
        rows: vec![
            vec![
                XptValue::Char("SUBJ001".to_string()),
                XptValue::Num(Some(12.5)),
            ],
            vec![XptValue::Char("SUBJ002".to_string()), XptValue::Num(None)],
        ],
    };
    let mut options = XptWriterOptions::default();
    options.missing_numeric = MissingNumeric::Standard;
    write_xpt(&path, &dataset, &options).expect("write xpt");
    let round = read_xpt(&path).expect("read back");
    assert_eq!(round.name, "TEST");
    assert_eq!(round.columns.len(), 2);
    assert_eq!(round.rows.len(), 2);
    assert_eq!(round.rows[0][1], XptValue::Num(Some(12.5)));
    assert_eq!(round.rows[1][1], XptValue::Num(None));
    fs::remove_file(&path).ok();
}
