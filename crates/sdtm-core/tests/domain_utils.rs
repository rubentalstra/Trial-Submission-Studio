use sdtm_core::{
    SdtmRole, order_variables_by_role, reorder_columns_by_role, validate_column_order,
    variable_sort_key,
};
use sdtm_model::{Domain, Variable, VariableType};

fn make_variable(name: &str, role: &str, order: u32) -> Variable {
    Variable {
        name: name.to_string(),
        label: Some(name.to_string()),
        data_type: VariableType::Char,
        length: Some(200),
        role: Some(role.to_string()),
        core: Some("Req".to_string()),
        codelist_code: None,
        order: Some(order),
    }
}

fn make_ae_domain() -> Domain {
    Domain {
        code: "AE".to_string(),
        description: Some("Adverse Events".to_string()),
        class_name: Some("Events".to_string()),
        dataset_class: None,
        label: Some("Adverse Events".to_string()),
        structure: Some("One record per adverse event per subject".to_string()),
        dataset_name: Some("AE".to_string()),
        variables: vec![
            make_variable("STUDYID", "Identifier", 1),
            make_variable("DOMAIN", "Identifier", 2),
            make_variable("USUBJID", "Identifier", 3),
            make_variable("AESEQ", "Identifier", 5),
            make_variable("AETERM", "Topic", 9),
            make_variable("AEDECOD", "Synonym Qualifier", 13),
            make_variable("AECAT", "Grouping Qualifier", 19),
            make_variable("AESEV", "Record Qualifier", 27),
            make_variable("AESTDTC", "Timing", 53),
            make_variable("AEENDTC", "Timing", 54),
        ],
    }
}

#[test]
fn sdtm_role_parse_parses_all_roles() {
    assert_eq!(SdtmRole::parse("Identifier"), Some(SdtmRole::Identifier));
    assert_eq!(SdtmRole::parse("Topic"), Some(SdtmRole::Topic));
    assert_eq!(
        SdtmRole::parse("Grouping Qualifier"),
        Some(SdtmRole::GroupingQualifier)
    );
    assert_eq!(
        SdtmRole::parse("Result Qualifier"),
        Some(SdtmRole::ResultQualifier)
    );
    assert_eq!(
        SdtmRole::parse("Synonym Qualifier"),
        Some(SdtmRole::SynonymQualifier)
    );
    assert_eq!(
        SdtmRole::parse("Record Qualifier"),
        Some(SdtmRole::RecordQualifier)
    );
    assert_eq!(
        SdtmRole::parse("Variable Qualifier"),
        Some(SdtmRole::VariableQualifier)
    );
    assert_eq!(SdtmRole::parse("Rule"), Some(SdtmRole::Rule));
    assert_eq!(SdtmRole::parse("Timing"), Some(SdtmRole::Timing));
    assert_eq!(SdtmRole::parse("Unknown"), None);
    assert_eq!(SdtmRole::parse(""), None);
}

#[test]
fn sdtm_role_parse_is_case_insensitive() {
    assert_eq!(SdtmRole::parse("identifier"), Some(SdtmRole::Identifier));
    assert_eq!(SdtmRole::parse("IDENTIFIER"), Some(SdtmRole::Identifier));
    assert_eq!(SdtmRole::parse("TIMING"), Some(SdtmRole::Timing));
    assert_eq!(
        SdtmRole::parse("grouping qualifier"),
        Some(SdtmRole::GroupingQualifier)
    );
}

#[test]
fn sdtm_role_sort_order_is_correct() {
    // Per SDTMIG: Identifiers, Topic, Qualifiers, Rule, Timing
    assert!(SdtmRole::Identifier.sort_order() < SdtmRole::Topic.sort_order());
    assert!(SdtmRole::Topic.sort_order() < SdtmRole::GroupingQualifier.sort_order());
    assert!(SdtmRole::GroupingQualifier.sort_order() < SdtmRole::ResultQualifier.sort_order());
    assert!(SdtmRole::ResultQualifier.sort_order() < SdtmRole::SynonymQualifier.sort_order());
    assert!(SdtmRole::SynonymQualifier.sort_order() < SdtmRole::RecordQualifier.sort_order());
    assert!(SdtmRole::RecordQualifier.sort_order() < SdtmRole::VariableQualifier.sort_order());
    assert!(SdtmRole::VariableQualifier.sort_order() < SdtmRole::Rule.sort_order());
    assert!(SdtmRole::Rule.sort_order() < SdtmRole::Timing.sort_order());
}

#[test]
fn sdtm_role_is_qualifier_returns_true_for_qualifier_roles() {
    assert!(!SdtmRole::Identifier.is_qualifier());
    assert!(!SdtmRole::Topic.is_qualifier());
    assert!(SdtmRole::GroupingQualifier.is_qualifier());
    assert!(SdtmRole::ResultQualifier.is_qualifier());
    assert!(SdtmRole::SynonymQualifier.is_qualifier());
    assert!(SdtmRole::RecordQualifier.is_qualifier());
    assert!(SdtmRole::VariableQualifier.is_qualifier());
    assert!(!SdtmRole::Rule.is_qualifier());
    assert!(!SdtmRole::Timing.is_qualifier());
}

#[test]
fn variable_sort_key_uses_role_then_order() {
    let id_var = make_variable("STUDYID", "Identifier", 1);
    let topic_var = make_variable("AETERM", "Topic", 9);
    let timing_var = make_variable("AESTDTC", "Timing", 53);

    let id_key = variable_sort_key(&id_var);
    let topic_key = variable_sort_key(&topic_var);
    let timing_key = variable_sort_key(&timing_var);

    assert!(id_key < topic_key);
    assert!(topic_key < timing_key);
}

#[test]
fn order_variables_by_role_sorts_correctly() {
    // Create variables in wrong order
    let unordered = vec![
        make_variable("AESTDTC", "Timing", 53),         // Timing
        make_variable("AETERM", "Topic", 9),            // Topic
        make_variable("STUDYID", "Identifier", 1),      // Identifier
        make_variable("AESEV", "Record Qualifier", 27), // Record Qualifier
    ];

    let ordered = order_variables_by_role(&unordered);

    assert_eq!(ordered[0].name, "STUDYID"); // Identifier first
    assert_eq!(ordered[1].name, "AETERM"); // Topic second
    assert_eq!(ordered[2].name, "AESEV"); // Record Qualifier third
    assert_eq!(ordered[3].name, "AESTDTC"); // Timing last
}

#[test]
fn validate_column_order_detects_violations() {
    let domain = make_ae_domain();

    // Wrong order: Timing before Topic
    let wrong_order = vec![
        "STUDYID".to_string(),
        "AESTDTC".to_string(), // Timing before Topic - wrong!
        "AETERM".to_string(),  // Topic
    ];

    let result = validate_column_order(&wrong_order, &domain);
    assert!(!result.is_valid);
    assert!(!result.violations.is_empty());
    assert!(result.violations[0].contains("AETERM"));
}

#[test]
fn validate_column_order_accepts_correct_order() {
    let domain = make_ae_domain();

    // Correct order: Identifier, Topic, Qualifier, Timing
    let correct_order = vec![
        "STUDYID".to_string(),
        "DOMAIN".to_string(),
        "USUBJID".to_string(),
        "AETERM".to_string(),
        "AESEV".to_string(),
        "AESTDTC".to_string(),
    ];

    let result = validate_column_order(&correct_order, &domain);
    assert!(result.is_valid);
    assert!(result.violations.is_empty());
}

#[test]
fn reorder_columns_by_role_fixes_order() {
    let domain = make_ae_domain();

    // Wrong order
    let wrong_order = vec![
        "AESTDTC".to_string(), // Timing
        "AETERM".to_string(),  // Topic
        "STUDYID".to_string(), // Identifier
        "AESEV".to_string(),   // Record Qualifier
    ];

    let reordered = reorder_columns_by_role(&wrong_order, &domain);

    assert_eq!(reordered[0], "STUDYID"); // Identifier first
    assert_eq!(reordered[1], "AETERM"); // Topic second
    assert_eq!(reordered[2], "AESEV"); // Record Qualifier (only one of this type)
    assert_eq!(reordered[3], "AESTDTC"); // Timing last
}

#[test]
fn reorder_columns_preserves_unknown_columns_at_end() {
    let domain = make_ae_domain();

    let with_extra = vec![
        "CUSTOM".to_string(),
        "AETERM".to_string(),
        "STUDYID".to_string(),
    ];

    let reordered = reorder_columns_by_role(&with_extra, &domain);

    assert_eq!(reordered[0], "STUDYID");
    assert_eq!(reordered[1], "AETERM");
    assert_eq!(reordered[2], "CUSTOM"); // Unknown at end
}

#[test]
fn reorder_columns_preserves_case() {
    let domain = make_ae_domain();

    let mixed_case = vec!["studyid".to_string(), "AeTerm".to_string()];

    let reordered = reorder_columns_by_role(&mixed_case, &domain);

    // Should preserve original casing
    assert_eq!(reordered[0], "studyid");
    assert_eq!(reordered[1], "AeTerm");
}
