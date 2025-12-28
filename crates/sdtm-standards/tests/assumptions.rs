//! Tests for assumption types and dynamic rule generation.

use sdtm_model::{Domain, Variable, VariableType};
use sdtm_standards::{CoreDesignation, RuleContext, RuleGenerator};

// ============================================================================
// CoreDesignation Tests
// ============================================================================

#[test]
fn parses_core_designations() {
    assert_eq!(CoreDesignation::parse("Req"), Some(CoreDesignation::Req));
    assert_eq!(CoreDesignation::parse("REQ"), Some(CoreDesignation::Req));
    assert_eq!(
        CoreDesignation::parse("required"),
        Some(CoreDesignation::Req)
    );
    assert_eq!(CoreDesignation::parse("Exp"), Some(CoreDesignation::Exp));
    assert_eq!(
        CoreDesignation::parse("expected"),
        Some(CoreDesignation::Exp)
    );
    assert_eq!(CoreDesignation::parse("Perm"), Some(CoreDesignation::Perm));
    assert_eq!(
        CoreDesignation::parse("permissible"),
        Some(CoreDesignation::Perm)
    );
    assert_eq!(CoreDesignation::parse("invalid"), None);
    assert_eq!(CoreDesignation::parse(""), None);
}

#[test]
fn core_designation_priority_ordering() {
    assert!(CoreDesignation::Req.priority() > CoreDesignation::Exp.priority());
    assert!(CoreDesignation::Exp.priority() > CoreDesignation::Perm.priority());
}

// ============================================================================
// RuleGenerator Tests
// ============================================================================

fn make_test_variable(name: &str, core: Option<&str>, codelist: Option<&str>) -> Variable {
    Variable {
        name: name.to_string(),
        label: Some(format!("{} Label", name)),
        data_type: VariableType::Char,
        length: None,
        role: Some("Identifier".to_string()),
        core: core.map(|s| s.to_string()),
        codelist_code: codelist.map(|s| s.to_string()),
        order: None,
    }
}

fn make_test_domain(code: &str, variables: Vec<Variable>) -> Domain {
    Domain {
        code: code.to_string(),
        description: None,
        class_name: None,
        dataset_class: None,
        label: None,
        structure: None,
        dataset_name: None,
        variables,
    }
}

#[test]
fn generates_required_variable_rule() {
    use sdtm_model::CtRegistry;

    let generator = RuleGenerator::new();
    let domain = make_test_domain("DM", vec![make_test_variable("STUDYID", Some("Req"), None)]);
    let ct = CtRegistry::default();

    let rules = generator.generate_rules_for_domain(&domain, &ct);

    // Required variables generate 2 rules: presence + null check
    assert_eq!(rules.len(), 2);
    let categories: Vec<_> = rules.iter().map(|r| r.category.as_str()).collect();
    assert!(categories.contains(&"SDTMIG_REQ"));
    assert!(categories.contains(&"SDTMIG_NULL"));

    let presence_rule = rules.iter().find(|r| r.category == "SDTMIG_REQ").unwrap();
    let null_rule = rules.iter().find(|r| r.category == "SDTMIG_NULL").unwrap();
    assert_eq!(presence_rule.variable, "STUDYID");
    assert_eq!(null_rule.variable, "STUDYID");
    assert!(matches!(
        presence_rule.context,
        RuleContext::RequiredPresence
    ));
    assert!(matches!(null_rule.context, RuleContext::RequiredVariable));
}

#[test]
fn generates_datetime_rule() {
    use sdtm_model::CtRegistry;

    let generator = RuleGenerator::new();
    let domain = make_test_domain(
        "AE",
        vec![make_test_variable("AESTDTC", Some("Perm"), None)],
    );
    let ct = CtRegistry::default();

    let rules = generator.generate_rules_for_domain(&domain, &ct);

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].category, "SDTMIG_DTC");
    assert_eq!(rules[0].variable, "AESTDTC");
    assert!(matches!(rules[0].context, RuleContext::DateTimeFormat));
}

#[test]
fn generates_sequence_rule() {
    use sdtm_model::CtRegistry;

    let generator = RuleGenerator::new();
    let domain = make_test_domain("AE", vec![make_test_variable("AESEQ", Some("Req"), None)]);
    let ct = CtRegistry::default();

    let rules = generator.generate_rules_for_domain(&domain, &ct);

    // Should have presence, completeness, and sequence uniqueness
    assert_eq!(rules.len(), 3);
    let categories: Vec<_> = rules.iter().map(|r| r.category.as_str()).collect();
    assert!(categories.contains(&"SDTMIG_REQ"));
    assert!(categories.contains(&"SDTMIG_NULL"));
    assert!(categories.contains(&"SDTMIG_SEQ"));
}
