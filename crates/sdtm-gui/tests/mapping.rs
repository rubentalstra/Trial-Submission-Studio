//! Tests for mapping service.

use polars::prelude::*;
use sdtm_model::{Domain, Variable, VariableType};
use std::collections::BTreeMap;

use sdtm_gui::services::{MappingService, VariableMappingStatus};

fn make_variable(name: &str, label: &str, core: Option<&str>) -> Variable {
    Variable {
        name: name.to_string(),
        label: Some(label.to_string()),
        data_type: VariableType::Char,
        length: None,
        role: None,
        core: core.map(String::from),
        codelist_code: None,
        order: None,
        described_value_domain: None,
    }
}

fn test_domain() -> Domain {
    Domain {
        code: "DM".to_string(),
        description: Some("Demographics".to_string()),
        class_name: Some("SPECIAL PURPOSE".to_string()),
        dataset_class: None,
        label: Some("Demographics".to_string()),
        structure: None,
        dataset_name: None,
        variables: vec![
            make_variable("STUDYID", "Study Identifier", Some("Req")),
            make_variable("USUBJID", "Unique Subject Identifier", Some("Req")),
            make_variable("AGE", "Age", Some("Exp")),
        ],
    }
}

#[test]
fn test_mapping_state_accept_suggestion() {
    let domain = test_domain();
    let columns = vec!["AGE".to_string()];
    let hints = BTreeMap::new();

    let mut state = MappingService::create_mapping_state(domain, "STUDY01", &columns, hints);

    // Should have suggestion for AGE
    assert!(state.get_suggestion_for("AGE").is_some());

    // Accept it
    state.accept_suggestion("AGE");
    assert!(state.get_accepted_for("AGE").is_some());
    assert_eq!(
        state.variable_status("AGE"),
        VariableMappingStatus::Accepted
    );
}

#[test]
fn test_mapping_state_clear() {
    let domain = test_domain();
    let columns = vec!["AGE".to_string()];
    let hints = BTreeMap::new();

    let mut state = MappingService::create_mapping_state(domain, "STUDY01", &columns, hints);

    state.accept_suggestion("AGE");
    assert!(state.get_accepted_for("AGE").is_some());

    state.clear_mapping("AGE");
    assert!(state.get_accepted_for("AGE").is_none());
}

#[test]
fn test_extract_column_hints() {
    let df = DataFrame::new(vec![
        Series::new("NAME".into(), vec!["Alice", "Bob", "Charlie"]).into(),
        Series::new("AGE".into(), vec![25i64, 30, 35]).into(),
    ])
    .unwrap();

    let hints = MappingService::extract_column_hints(&df);

    assert!(hints.contains_key("NAME"));
    assert!(hints.contains_key("AGE"));
    assert!(!hints["NAME"].is_numeric);
    assert!(hints["AGE"].is_numeric);
}

#[test]
fn test_ct_variable_mapping_and_detection() {
    // Create domain with DTHFL that has a codelist code
    let domain = Domain {
        code: "DM".to_string(),
        description: Some("Demographics".to_string()),
        class_name: None,
        dataset_class: None,
        label: None,
        structure: None,
        dataset_name: None,
        variables: vec![Variable {
            name: "DTHFL".to_string(),
            label: Some("Subject Death Flag".to_string()),
            data_type: VariableType::Char,
            length: None,
            role: Some("Record Qualifier".to_string()),
            core: Some("Exp".to_string()),
            codelist_code: Some("C66742".to_string()),
            order: None,
            described_value_domain: None,
        }],
    };

    let columns = vec!["DEATH_FLAG".to_string()];
    let hints = BTreeMap::new();

    let mut state = MappingService::create_mapping_state(domain, "STUDY01", &columns, hints);

    // Manually accept a mapping for DTHFL
    state.accept_manual("DTHFL", "DEATH_FLAG");

    // Verify the mapping is stored correctly
    let accepted = state.get_accepted_for("DTHFL");
    assert!(
        accepted.is_some(),
        "DTHFL should be in accepted mappings after accept_manual"
    );
    assert_eq!(accepted.unwrap().0, "DEATH_FLAG");

    // Verify we can find the variable with codelist
    let dthfl_var = state
        .sdtm_domain
        .variables
        .iter()
        .find(|v| v.name == "DTHFL");
    assert!(dthfl_var.is_some());
    assert_eq!(dthfl_var.unwrap().codelist_code.as_deref(), Some("C66742"));

    // Now check if the transform detection would work
    // This simulates what rebuild_transforms_if_needed does
    let ct_transforms: Vec<_> = state
        .sdtm_domain
        .variables
        .iter()
        .filter(|v| v.codelist_code.is_some())
        .filter(|v| state.get_accepted_for(&v.name).is_some())
        .map(|v| (v.name.clone(), v.codelist_code.clone()))
        .collect();

    assert!(
        !ct_transforms.is_empty(),
        "Should detect CT normalization for DTHFL"
    );
    assert_eq!(ct_transforms[0].0, "DTHFL");
    assert_eq!(ct_transforms[0].1, Some("C66742".to_string()));
}

#[test]
fn test_ct_detection_with_loaded_dm_domain() {
    use sdtm_standards::load_default_sdtm_ig_domains;

    // Load the actual DM domain from standards
    let domains = load_default_sdtm_ig_domains().expect("load domains");
    let dm = domains
        .into_iter()
        .find(|d| d.code == "DM")
        .expect("DM domain");

    // Verify DTHFL has codelist_code
    let dthfl = dm
        .variables
        .iter()
        .find(|v| v.name == "DTHFL")
        .expect("DTHFL");
    assert_eq!(dthfl.codelist_code.as_deref(), Some("C66742"));

    // Create mapping state
    let columns = vec![
        "DEATH_FLAG".to_string(),
        "SEX".to_string(),
        "RACE".to_string(),
    ];
    let hints = BTreeMap::new();
    let mut state = MappingService::create_mapping_state(dm, "STUDY01", &columns, hints);

    // Accept mappings for CT variables
    state.accept_manual("DTHFL", "DEATH_FLAG");
    state.accept_manual("SEX", "SEX");
    state.accept_manual("RACE", "RACE");

    // Verify all three are accepted
    assert!(state.get_accepted_for("DTHFL").is_some());
    assert!(state.get_accepted_for("SEX").is_some());
    assert!(state.get_accepted_for("RACE").is_some());

    // Detect CT transforms
    let ct_transforms: Vec<_> = state
        .sdtm_domain
        .variables
        .iter()
        .filter(|v| v.codelist_code.is_some())
        .filter(|v| state.get_accepted_for(&v.name).is_some())
        .map(|v| v.name.clone())
        .collect();

    // Should detect DTHFL, SEX, RACE (all have CT codelists and are accepted)
    assert!(
        ct_transforms.contains(&"DTHFL".to_string()),
        "DTHFL should be detected. Got: {:?}",
        ct_transforms
    );
    assert!(
        ct_transforms.contains(&"SEX".to_string()),
        "SEX should be detected. Got: {:?}",
        ct_transforms
    );
    assert!(
        ct_transforms.contains(&"RACE".to_string()),
        "RACE should be detected. Got: {:?}",
        ct_transforms
    );
}
