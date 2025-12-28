use sdtm_model::DatasetClass;
use sdtm_standards::{load_default_ct_registry, load_default_sdtm_ig_domains};

#[test]
fn loads_sdtmig_domains() {
    let domains = load_default_sdtm_ig_domains().expect("load sdtmig domains");
    assert!(!domains.is_empty());
    let dm = domains.iter().find(|d| d.code == "DM");
    assert!(dm.is_some());
}

#[test]
fn loads_ct_registry() {
    let registry = load_default_ct_registry().expect("load ct");
    assert!(!registry.catalogs.is_empty());
    assert!(registry.catalogs.contains_key("SDTM CT"));
    assert!(registry.catalogs.contains_key("SEND CT"));
}

#[test]
fn ct_catalog_has_version_and_publishing_set() {
    let registry = load_default_ct_registry().expect("load ct");
    let sdtm_catalog = registry.catalogs.get("SDTM CT").expect("SDTM CT catalog");

    // Verify publishing_set is parsed from filename (e.g., SDTM_CT_2024-03-29.csv)
    assert_eq!(
        sdtm_catalog.publishing_set.as_deref(),
        Some("SDTM"),
        "publishing_set should be 'SDTM'"
    );

    // Verify version is parsed (the date portion)
    assert!(
        sdtm_catalog.version.is_some(),
        "version should be set from filename"
    );
    let version = sdtm_catalog.version.as_ref().unwrap();
    // Version should look like a date: YYYY-MM-DD
    assert!(
        version.contains('-'),
        "version should be date format like '2024-03-29', got: {}",
        version
    );
}

#[test]
fn resolved_ct_includes_catalog_reference() {
    let registry = load_default_ct_registry().expect("load ct");
    // Resolve a known codelist (SEX - C66731)
    let resolved = registry
        .resolve("C66731", None)
        .expect("SEX codelist should exist");

    // Verify we can access the catalog metadata through the resolved reference
    assert!(!resolved.source().is_empty());
    assert!(
        resolved.catalog.publishing_set.is_some(),
        "catalog should have publishing_set"
    );
    assert!(
        resolved.catalog.version.is_some(),
        "catalog should have version"
    );
}

#[test]
fn loads_send_country_codelist() {
    let registry = load_default_ct_registry().expect("load ct");
    let send_catalog = registry.catalogs.get("SEND CT").expect("send ct");
    // Find the COUNTRY codelist (C66786) in the catalog
    let ct = send_catalog
        .codelists
        .values()
        .find(|cl| cl.code == "C66786")
        .expect("country codelist");
    assert_eq!(ct.code, "C66786");
    assert!(ct.submission_values().contains(&"USA"));
}

#[test]
fn dataset_class_from_str_parsing() {
    use std::str::FromStr;

    // Test parsing various formats
    assert_eq!(
        DatasetClass::from_str("Interventions").unwrap(),
        DatasetClass::Interventions
    );
    assert_eq!(
        DatasetClass::from_str("EVENTS").unwrap(),
        DatasetClass::Events
    );
    assert_eq!(
        DatasetClass::from_str("findings").unwrap(),
        DatasetClass::Findings
    );
    assert_eq!(
        DatasetClass::from_str("Findings About").unwrap(),
        DatasetClass::FindingsAbout
    );
    assert_eq!(
        DatasetClass::from_str("Special-Purpose").unwrap(),
        DatasetClass::SpecialPurpose
    );
    assert_eq!(
        DatasetClass::from_str("Special Purpose").unwrap(),
        DatasetClass::SpecialPurpose
    );
    assert_eq!(
        DatasetClass::from_str("Trial Design").unwrap(),
        DatasetClass::TrialDesign
    );
    assert_eq!(
        DatasetClass::from_str("Study Reference").unwrap(),
        DatasetClass::StudyReference
    );
    assert_eq!(
        DatasetClass::from_str("Relationship").unwrap(),
        DatasetClass::Relationship
    );

    // Test invalid class
    assert!(DatasetClass::from_str("InvalidClass").is_err());
}

#[test]
fn dataset_class_display_and_as_str() {
    assert_eq!(DatasetClass::Interventions.as_str(), "Interventions");
    assert_eq!(DatasetClass::Events.as_str(), "Events");
    assert_eq!(DatasetClass::Findings.as_str(), "Findings");
    assert_eq!(DatasetClass::FindingsAbout.as_str(), "Findings About");
    assert_eq!(DatasetClass::SpecialPurpose.as_str(), "Special-Purpose");
    assert_eq!(DatasetClass::TrialDesign.as_str(), "Trial Design");
    assert_eq!(DatasetClass::StudyReference.as_str(), "Study Reference");
    assert_eq!(DatasetClass::Relationship.as_str(), "Relationship");

    // Test Display trait
    assert_eq!(format!("{}", DatasetClass::Events), "Events");
}

#[test]
fn country_codelist_c66786_loaded_correctly() {
    // Verify the COUNTRY codelist (C66786) is loaded from CT files
    // This tests the example the user mentioned
    let ct_registry = load_default_ct_registry().expect("load ct");

    // Should be able to resolve C66786
    let resolved = ct_registry
        .resolve("C66786", None)
        .expect("C66786 should resolve");

    assert_eq!(resolved.codelist.code, "C66786");
    assert!(
        !resolved.codelist.extensible,
        "COUNTRY codelist is non-extensible"
    );

    // Should have many country codes as submission values
    let values = resolved.codelist.submission_values();
    assert!(
        values.len() > 100,
        "COUNTRY should have many values, got {}",
        values.len()
    );

    // Should include specific countries from the user's example
    assert!(values.contains(&"ABW"), "Should include ABW (Aruba)");
    assert!(values.contains(&"AFG"), "Should include AFG (Afghanistan)");
    assert!(values.contains(&"AGO"), "Should include AGO (Angola)");
    assert!(values.contains(&"USA"), "Should include USA");
}
