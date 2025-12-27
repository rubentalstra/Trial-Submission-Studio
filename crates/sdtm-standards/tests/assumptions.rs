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

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rule_id, "SD0002");
    assert_eq!(rules[0].variable, "STUDYID");
    assert!(matches!(rules[0].context, RuleContext::RequiredVariable));
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
    assert_eq!(rules[0].rule_id, "SD0003");
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

    // Should have both SD0002 (Required) and SD0005 (Sequence)
    assert_eq!(rules.len(), 2);
    let rule_ids: Vec<_> = rules.iter().map(|r| r.rule_id.as_str()).collect();
    assert!(rule_ids.contains(&"SD0002"));
    assert!(rule_ids.contains(&"SD0005"));
}
