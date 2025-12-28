use sdtm_model::DatasetClass;
use sdtm_standards::{
    load_default_ct_registry, load_default_domain_registry, load_default_sdtm_domains,
    load_default_sdtm_ig_domains,
};

#[test]
fn loads_sdtmig_domains() {
    let domains = load_default_sdtm_ig_domains().expect("load sdtmig domains");
    assert!(!domains.is_empty());
    let dm = domains.iter().find(|d| d.code == "DM");
    assert!(dm.is_some());
}

#[test]
fn loads_sdtm_domains() {
    let domains = load_default_sdtm_domains().expect("load sdtm domains");
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
    assert!(ct.submission_values().iter().any(|value| *value == "USA"));
}

// Tests for DomainRegistry and DatasetClass functionality per SDTMIG v3.4

#[test]
fn domain_registry_loads_all_classes() {
    let registry = load_default_domain_registry().expect("load domain registry");

    // Per SDTMIG v3.4, we should have domains from all classes
    assert!(!registry.is_empty());
    assert!(registry.len() >= 50); // SDTMIG v3.4 has 63 domains

    // Check that DM is classified as Special-Purpose (Chapter 5)
    let dm = registry.get("DM").expect("DM domain");
    assert_eq!(dm.dataset_class, Some(DatasetClass::SpecialPurpose));

    // Check that AE is classified as Events (Chapter 6)
    let ae = registry.get("AE").expect("AE domain");
    assert_eq!(ae.dataset_class, Some(DatasetClass::Events));

    // Check that LB is classified as Findings (Chapter 6)
    let lb = registry.get("LB").expect("LB domain");
    assert_eq!(lb.dataset_class, Some(DatasetClass::Findings));

    // Check that CM is classified as Interventions (Chapter 6)
    let cm = registry.get("CM").expect("CM domain");
    assert_eq!(cm.dataset_class, Some(DatasetClass::Interventions));

    // Check that TA is classified as Trial Design (Chapter 7)
    let ta = registry.get("TA").expect("TA domain");
    assert_eq!(ta.dataset_class, Some(DatasetClass::TrialDesign));
}

#[test]
fn domain_registry_queries_by_class() {
    let registry = load_default_domain_registry().expect("load domain registry");

    // General Observation domains per SDTMIG v3.4 Section 2.1
    let go_domains = registry.get_general_observation_domains();
    assert!(!go_domains.is_empty());

    // All GO domains should be Interventions, Events, Findings, or Findings About
    for domain in &go_domains {
        assert!(
            domain.is_general_observation(),
            "Domain {} should be a General Observation domain",
            domain.code
        );
    }

    // Check specific class queries
    let interventions = registry.get_by_class(DatasetClass::Interventions);
    assert!(!interventions.is_empty());
    // AG, CM, EC, EX, ML, PR, SU are Interventions
    assert!(interventions.iter().any(|d| d.code == "CM"));
    assert!(interventions.iter().any(|d| d.code == "EX"));

    let events = registry.get_by_class(DatasetClass::Events);
    assert!(!events.is_empty());
    // AE, BE, CE, DS, DV, HO, MH are Events
    assert!(events.iter().any(|d| d.code == "AE"));
    assert!(events.iter().any(|d| d.code == "MH"));

    let findings = registry.get_by_class(DatasetClass::Findings);
    assert!(!findings.is_empty());
    // LB, VS, EG, etc. are Findings
    assert!(findings.iter().any(|d| d.code == "LB"));
    assert!(findings.iter().any(|d| d.code == "VS"));
}

#[test]
fn domain_registry_special_purpose_domains() {
    let registry = load_default_domain_registry().expect("load domain registry");

    // Special-Purpose domains per SDTMIG v3.4 Chapter 5
    let sp_domains = registry.get_special_purpose_domains();
    assert!(!sp_domains.is_empty());

    // CO, DM, SE, SM, SV are Special-Purpose
    let codes: Vec<&str> = sp_domains.iter().map(|d| d.code.as_str()).collect();
    assert!(codes.contains(&"CO"), "CO should be Special-Purpose");
    assert!(codes.contains(&"DM"), "DM should be Special-Purpose");
    assert!(codes.contains(&"SE"), "SE should be Special-Purpose");
    assert!(codes.contains(&"SV"), "SV should be Special-Purpose");
}

#[test]
fn domain_registry_trial_design_domains() {
    let registry = load_default_domain_registry().expect("load domain registry");

    // Trial Design domains per SDTMIG v3.4 Chapter 7
    let td_domains = registry.get_trial_design_domains();
    assert!(!td_domains.is_empty());

    // TA, TD, TE, TI, TM, TS, TV are Trial Design
    let codes: Vec<&str> = td_domains.iter().map(|d| d.code.as_str()).collect();
    assert!(codes.contains(&"TA"), "TA should be Trial Design");
    assert!(codes.contains(&"TE"), "TE should be Trial Design");
    assert!(codes.contains(&"TS"), "TS should be Trial Design");
    assert!(codes.contains(&"TV"), "TV should be Trial Design");
}

#[test]
fn domain_registry_relationship_domains() {
    let registry = load_default_domain_registry().expect("load domain registry");

    // Relationship datasets per SDTMIG v3.4 Chapter 8
    let rel_domains = registry.get_relationship_domains();
    assert!(!rel_domains.is_empty());

    // RELREC, RELSPEC, RELSUB, SUPPQUAL are Relationship
    let codes: Vec<&str> = rel_domains.iter().map(|d| d.code.as_str()).collect();
    assert!(codes.contains(&"RELREC"), "RELREC should be Relationship");
    assert!(codes.contains(&"RELSPEC"), "RELSPEC should be Relationship");
    assert!(codes.contains(&"RELSUB"), "RELSUB should be Relationship");
    assert!(
        codes.contains(&"SUPPQUAL"),
        "SUPPQUAL should be Relationship"
    );
}

#[test]
fn domain_registry_class_helpers() {
    let registry = load_default_domain_registry().expect("load domain registry");

    // Test is_class helper
    assert!(registry.is_class("DM", DatasetClass::SpecialPurpose));
    assert!(registry.is_class("AE", DatasetClass::Events));
    assert!(registry.is_class("LB", DatasetClass::Findings));
    assert!(registry.is_class("CM", DatasetClass::Interventions));
    assert!(registry.is_class("TA", DatasetClass::TrialDesign));
    assert!(registry.is_class("RELREC", DatasetClass::Relationship));

    // Test is_general_observation helper
    assert!(registry.is_general_observation("AE"));
    assert!(registry.is_general_observation("LB"));
    assert!(registry.is_general_observation("CM"));
    assert!(!registry.is_general_observation("DM"));
    assert!(!registry.is_general_observation("TA"));
    assert!(!registry.is_general_observation("RELREC"));
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
fn dynamic_rule_generator_ct_extensibility() {
    use sdtm_standards::{
        RuleContext, RuleGenerator, RuleSeverity, load_default_ct_registry,
        load_default_sdtm_ig_domains,
    };

    let domains = load_default_sdtm_ig_domains().expect("load domains");
    let ct_registry = load_default_ct_registry().expect("load ct");
    let generator = RuleGenerator::new();

    // Get the AE domain (has many CT-controlled variables)
    let ae = domains.iter().find(|d| d.code == "AE").expect("AE domain");
    let rules = generator.generate_rules_for_domain(ae, &ct_registry);

    // Find CT rules and check extensibility affects severity
    let ct_rules: Vec<_> = rules
        .iter()
        .filter(|r| matches!(r.context, RuleContext::ControlledTerminology { .. }))
        .collect();

    let mut found_non_extensible = false;

    for rule in ct_rules {
        if let RuleContext::ControlledTerminology { extensible, .. } = &rule.context {
            if *extensible {
                // Extensible codelists should be warnings
                assert_eq!(
                    rule.severity,
                    RuleSeverity::Warning,
                    "Extensible CT should be Warning"
                );
            } else {
                found_non_extensible = true;
                // Non-extensible codelists should be errors
                assert_eq!(
                    rule.severity,
                    RuleSeverity::Error,
                    "Non-extensible CT should be Error"
                );
            }
        }
    }

    // Should have non-extensible CT rules
    assert!(found_non_extensible, "Should have non-extensible CT rules");
}

#[test]
fn dynamic_rule_generator_summary() {
    use sdtm_standards::{RuleGenerator, load_default_ct_registry, load_default_sdtm_ig_domains};

    let domains = load_default_sdtm_ig_domains().expect("load domains");
    let ct_registry = load_default_ct_registry().expect("load ct");
    let generator = RuleGenerator::new();

    // Generate summary across all domains
    let summary = generator.generate_summary(&domains, &ct_registry);

    // Should have many rules
    assert!(
        summary.total_rules > 1000,
        "Should generate many rules across all domains, got {}",
        summary.total_rules
    );

    // Should have rules for key categories
    assert!(
        summary.by_category.contains_key("Required Value Missing"),
        "Should have Required Value Missing rules"
    );
    assert!(
        summary
            .by_category
            .contains_key("Required Variable Missing"),
        "Should have Required Variable Missing rules"
    );

    // Should have rules for many domains
    assert!(
        summary.by_domain.len() > 50,
        "Should have rules for many domains"
    );
    assert!(summary.by_domain.contains_key("DM"), "Should have DM rules");
    assert!(summary.by_domain.contains_key("AE"), "Should have AE rules");
    assert!(summary.by_domain.contains_key("LB"), "Should have LB rules");
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
    assert!(
        values.iter().any(|v| *v == "ABW"),
        "Should include ABW (Aruba)"
    );
    assert!(
        values.iter().any(|v| *v == "AFG"),
        "Should include AFG (Afghanistan)"
    );
    assert!(
        values.iter().any(|v| *v == "AGO"),
        "Should include AGO (Angola)"
    );
    assert!(values.iter().any(|v| *v == "USA"), "Should include USA");
}
