#![allow(missing_docs)]

use std::path::PathBuf;

use sdtm_standards::load_ct_catalog;

fn test_ct_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/Controlled_Terminology/2024-03-29")
}

#[test]
fn test_load_sdtm_ct() {
    let path = test_ct_dir().join("SDTM_CT_2024-03-29.csv");
    if !path.exists() {
        return; // Skip if CT file not available
    }

    let catalog = load_ct_catalog(&path).unwrap();
    assert_eq!(catalog.label, "SDTM CT");
    assert_eq!(catalog.version, Some("2024-03-29".to_string()));
    assert_eq!(catalog.publishing_set, Some("SDTM".to_string()));

    // Check SEX codelist (C66731)
    let sex = catalog.get("C66731").expect("Sex codelist should exist");
    assert_eq!(sex.code, "C66731");
    assert_eq!(sex.name, "Sex");
    assert!(!sex.extensible);

    // Check terms
    assert!(sex.is_valid("F"));
    assert!(sex.is_valid("M"));
    assert!(sex.is_valid("U"));
    assert!(sex.is_valid("INTERSEX"));

    // Check synonym normalization
    assert_eq!(sex.normalize("Female"), "F");
    assert_eq!(sex.normalize("Male"), "M");
    assert_eq!(sex.normalize("UNK"), "U");
    assert_eq!(sex.normalize("Unknown"), "U");
}

#[test]
fn test_extensible_codelist() {
    let path = test_ct_dir().join("SDTM_CT_2024-03-29.csv");
    if !path.exists() {
        return;
    }

    let catalog = load_ct_catalog(&path).unwrap();

    // Unit codelist (C71620) is extensible
    let unit = catalog.get("C71620").expect("Unit codelist should exist");
    assert!(unit.extensible);
}
