use std::fs;
use std::path::PathBuf;

use proptest::prelude::*;
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
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/SDTM-MSG_v2.0_Sample_Submission_Package/m5/datasets/cdiscpilot01/tabulations/sdtm/dm.xpt");
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
    let options = XptWriterOptions {
        missing_numeric: MissingNumeric::Standard,
        ..Default::default()
    };
    write_xpt(&path, &dataset, &options).expect("write xpt");
    let round = read_xpt(&path).expect("read back");
    assert_eq!(round.name, "TEST");
    assert_eq!(round.columns.len(), 2);
    assert_eq!(round.rows.len(), 2);
    assert_eq!(round.rows[0][1], XptValue::Num(Some(12.5)));
    assert_eq!(round.rows[1][1], XptValue::Num(None));
    fs::remove_file(&path).ok();
}

// Property-based tests for XPT round-trip

/// Strategy for generating valid SAS names (1-8 uppercase ASCII chars, starting with letter)
fn sas_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9]{0,7}".prop_map(|s| s.to_uppercase())
}

/// Strategy for generating numeric values (avoiding extreme values that may lose precision)
fn numeric_value_strategy() -> impl Strategy<Value = Option<f64>> {
    prop_oneof![
        Just(None),
        (-1e10f64..1e10f64).prop_map(Some),
        Just(Some(0.0)),
        Just(Some(1.0)),
        Just(Some(-1.0)),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Test that numeric values survive round-trip through XPT format.
    #[test]
    fn proptest_numeric_roundtrip(value in numeric_value_strategy()) {
        let path = temp_file(&format!("proptest_num_{:?}", std::thread::current().id()));
        let dataset = XptDataset {
            name: "TEST".to_string(),
            label: None,
            columns: vec![XptColumn {
                name: "VAL".to_string(),
                label: None,
                data_type: XptType::Num,
                length: 8,
            }],
            rows: vec![vec![XptValue::Num(value)]],
        };
        let options = XptWriterOptions::default();
        write_xpt(&path, &dataset, &options).expect("write");
        let round = read_xpt(&path).expect("read");

        match (value, &round.rows[0][0]) {
            (None, XptValue::Num(None)) => {}
            (Some(orig), XptValue::Num(Some(read))) => {
                // Allow small floating point differences
                prop_assert!((orig - read).abs() < 1e-10 || (orig - read).abs() / orig.abs() < 1e-10,
                    "Value mismatch: {} vs {}", orig, read);
            }
            _ => prop_assert!(false, "Type mismatch"),
        }
        fs::remove_file(&path).ok();
    }

    /// Test that character values survive round-trip (up to column length).
    #[test]
    fn proptest_char_roundtrip(value in "[A-Za-z][A-Za-z0-9 ]{0,39}") {
        let path = temp_file(&format!("proptest_char_{:?}", std::thread::current().id()));
        let length = value.len().max(1).min(200) as u16;
        let dataset = XptDataset {
            name: "TEST".to_string(),
            label: None,
            columns: vec![XptColumn {
                name: "VAL".to_string(),
                label: None,
                data_type: XptType::Char,
                length,
            }],
            rows: vec![vec![XptValue::Char(value.clone())]],
        };
        let options = XptWriterOptions::default();
        write_xpt(&path, &dataset, &options).expect("write");
        let round = read_xpt(&path).expect("read");

        prop_assert!(!round.rows.is_empty(), "Expected at least one row");
        if let XptValue::Char(read) = &round.rows[0][0] {
            // XPT pads/truncates to column length, so compare trimmed
            let expected = value.chars().take(length as usize).collect::<String>();
            prop_assert_eq!(read.trim_end(), expected.trim_end());
        } else {
            prop_assert!(false, "Expected Char value");
        }
        fs::remove_file(&path).ok();
    }

    /// Test that dataset names survive round-trip.
    #[test]
    fn proptest_dataset_name_roundtrip(name in sas_name_strategy()) {
        let path = temp_file(&format!("proptest_name_{:?}", std::thread::current().id()));
        let dataset = XptDataset {
            name: name.clone(),
            label: None,
            columns: vec![XptColumn {
                name: "ID".to_string(),
                label: None,
                data_type: XptType::Num,
                length: 8,
            }],
            rows: vec![vec![XptValue::Num(Some(1.0))]],
        };
        let options = XptWriterOptions::default();
        write_xpt(&path, &dataset, &options).expect("write");
        let round = read_xpt(&path).expect("read");
        prop_assert_eq!(&round.name, &name);
        fs::remove_file(&path).ok();
    }
}
