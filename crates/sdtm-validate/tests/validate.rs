use std::fs;
use std::path::PathBuf;

use polars::prelude::{Column, DataFrame};

use sdtm_model::{ConformanceIssue, ConformanceReport, IssueSeverity, OutputFormat, VariableType};
use sdtm_standards::{load_default_ct_registry, load_default_sdtm_ig_domains};
use sdtm_validate::{
    ValidationContext, gate_strict_outputs, strict_outputs_requested, validate_domain,
    validate_domain_with_rules, write_conformance_report_json,
};

fn temp_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    dir.push(format!("sdtm_validate_{stamp}"));
    dir
}

#[test]
fn missing_required_column_emits_error() {
    let domains = load_default_sdtm_ig_domains().expect("standards");
    let domain = domains
        .iter()
        .find(|domain| domain.code == "DM")
        .expect("DM domain");
    let required = domain
        .variables
        .iter()
        .find(|var| matches!(var.core.as_deref(), Some("Req") | Some("REQ")))
        .expect("required variable");
    let df = DataFrame::new(vec![]).expect("df");
    let report = validate_domain(domain, &df, &ValidationContext::new());
    assert!(report.issues.iter().any(
        |issue| issue.code == "SDTMIG_REQ" && issue.variable.as_deref() == Some(&required.name)
    ));
}

#[test]
fn missing_required_values_emits_error() {
    let domains = load_default_sdtm_ig_domains().expect("standards");
    let domain = domains
        .iter()
        .find(|domain| domain.code == "DM")
        .expect("DM domain");
    let required = domain
        .variables
        .iter()
        .find(|var| matches!(var.core.as_deref(), Some("Req") | Some("REQ")))
        .expect("required variable");
    let df =
        DataFrame::new(vec![Column::new(required.name.clone().into(), ["", " "])]).expect("df");
    let report = validate_domain(domain, &df, &ValidationContext::new());
    let issue = report
        .issues
        .iter()
        .find(|issue| issue.code == "SDTMIG_NULL")
        .expect("missing value issue");
    assert_eq!(issue.count, Some(2));
}

#[test]
fn numeric_type_issue_emits_error() {
    let domains = load_default_sdtm_ig_domains().expect("standards");
    let domain = domains
        .iter()
        .find(|domain| domain.code == "DM")
        .expect("DM domain");
    let numeric = domain
        .variables
        .iter()
        .find(|var| var.data_type == VariableType::Num)
        .expect("numeric variable");
    let df = DataFrame::new(vec![Column::new(
        numeric.name.clone().into(),
        ["BAD", "12"],
    )])
    .expect("df");
    let report = validate_domain(domain, &df, &ValidationContext::new());
    let issue = report
        .issues
        .iter()
        .find(|issue| issue.code == "SDTMIG_TYPE")
        .expect("type issue");
    assert_eq!(issue.count, Some(1));
}

#[test]
fn ct_invalid_value_emits_issue() {
    let domains = load_default_sdtm_ig_domains().expect("standards");
    let domain = domains
        .iter()
        .find(|domain| domain.code == "DM")
        .expect("DM domain");
    let variable = domain
        .variables
        .iter()
        .find(|var| var.codelist_code.is_some())
        .expect("ct variable");
    let df = DataFrame::new(vec![Column::new(
        variable.name.clone().into(),
        ["INVALID_CT"],
    )])
    .expect("df");
    let ct_registry = load_default_ct_registry().expect("ct");
    let ctx = ValidationContext::new().with_ct_registry(&ct_registry);
    let report = validate_domain(domain, &df, &ctx);
    let issue = report
        .issues
        .iter()
        .find(|issue| issue.codelist_code.is_some() && issue.ct_source.is_some())
        .expect("ct issue");
    let ct_code = variable.codelist_code.as_deref().unwrap_or("");
    let ct = ct_registry
        .resolve_by_code(ct_code, None)
        .or_else(|| ct_registry.resolve_for_variable(variable, None))
        .map(|resolved| resolved.ct)
        .expect("ct lookup");
    let expected = if ct.extensible {
        IssueSeverity::Warning
    } else {
        IssueSeverity::Error
    };
    let _expected_rule = if ct.extensible { "CT2002" } else { "CT2001" };
    // rule_id check removed - check code/category instead
    assert_eq!(issue.code, ct.codelist_code);
    assert_eq!(issue.severity, expected);
    assert_eq!(
        issue.codelist_code.as_deref(),
        Some(ct.codelist_code.as_str())
    );
}

#[test]
fn writes_conformance_report_json_payload() {
    let report = ConformanceReport {
        domain_code: "DM".to_string(),
        issues: Vec::new(),
    };
    let dir = temp_dir();
    let path = write_conformance_report_json(&dir, "STUDY1", &[report]).expect("write json");
    let contents = fs::read_to_string(&path).expect("read json");
    assert!(contents.contains("cdisc-transpiler.conformance-report"));
    assert!(contents.contains("STUDY1"));
    fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn strict_output_gate_blocks_on_errors() {
    let report = ConformanceReport {
        domain_code: "AE".to_string(),
        issues: vec![ConformanceIssue {
            code: "SD0002".to_string(),
            message: "missing".to_string(),
            severity: IssueSeverity::Error,
            variable: Some("AETERM".to_string()),
            count: Some(1),
            category: None,
            codelist_code: None,
            ct_source: None,
        }],
    };
    let decision = gate_strict_outputs(&[OutputFormat::Xpt], true, &[report]);
    assert!(decision.block_strict_outputs);
    assert_eq!(decision.blocking_domains, vec!["AE".to_string()]);
}

#[test]
fn strict_output_gate_ignored_without_strict_formats() {
    let report = ConformanceReport {
        domain_code: "AE".to_string(),
        issues: vec![ConformanceIssue {
            code: "SD0002".to_string(),
            message: "missing".to_string(),
            severity: IssueSeverity::Error,
            variable: Some("AETERM".to_string()),
            count: Some(1),
            category: None,
            codelist_code: None,
            ct_source: None,
        }],
    };
    let decision = gate_strict_outputs(&[OutputFormat::Xml], true, &[report]);
    assert!(!decision.block_strict_outputs);
    assert!(decision.blocking_domains.is_empty());
}

#[test]
fn strict_outputs_requested_only_for_xpt() {
    assert!(strict_outputs_requested(&[OutputFormat::Xpt]));
    assert!(!strict_outputs_requested(&[OutputFormat::Xml]));
}

// ============================================================================
// Rule Engine Tests (Dynamic Rule-Driven Validation)
// ============================================================================

#[test]
fn rule_engine_validates_required_variable() {
    let domains = load_default_sdtm_ig_domains().expect("standards");
    let domain = domains.iter().find(|d| d.code == "DM").expect("DM domain");
    let ct_registry = load_default_ct_registry().expect("ct");

    let ctx = ValidationContext::new().with_ct_registry(&ct_registry);

    // DataFrame missing required column STUDYID
    let df = DataFrame::new(vec![]).expect("df");

    let report = validate_domain_with_rules(domain, &df, &ctx);

    // Should find SDTMIG_REQ (Required variable not found)
    let studyid_issue = report
        .issues
        .iter()
        .find(|i| i.variable.as_deref() == Some("STUDYID") && i.code == "SDTMIG_REQ");
    assert!(
        studyid_issue.is_some(),
        "Should emit SDTMIG_REQ for missing STUDYID"
    );
}

#[test]
fn rule_engine_validates_required_null_values() {
    let domains = load_default_sdtm_ig_domains().expect("standards");
    let domain = domains.iter().find(|d| d.code == "DM").expect("DM domain");
    let ct_registry = load_default_ct_registry().expect("ct");

    let ctx = ValidationContext::new().with_ct_registry(&ct_registry);

    // DataFrame with STUDYID but null values
    let df = DataFrame::new(vec![Column::new("STUDYID".into(), ["", " "])]).expect("df");

    let report = validate_domain_with_rules(domain, &df, &ctx);

    // Should find SDTMIG_NULL (Required variable has null values)
    let null_issue = report.issues.iter().find(|i| {
        i.variable.as_deref() == Some("STUDYID") && i.category.as_deref() == Some("SDTMIG_NULL")
    });
    assert!(
        null_issue.is_some(),
        "Should emit SDTMIG_NULL for null STUDYID values"
    );
    assert_eq!(null_issue.unwrap().count, Some(2));
}

#[test]
fn rule_engine_validates_ct_values() {
    let domains = load_default_sdtm_ig_domains().expect("standards");
    let domain = domains.iter().find(|d| d.code == "DM").expect("DM domain");
    let ct_registry = load_default_ct_registry().expect("ct");

    let ctx = ValidationContext::new().with_ct_registry(&ct_registry);

    // SEX variable uses codelist C66731, valid values include M, F
    let df = DataFrame::new(vec![Column::new("SEX".into(), ["INVALID_SEX"])]).expect("df");

    let report = validate_domain_with_rules(domain, &df, &ctx);

    // Should find CT rule issue (code will be the CT codelist code)
    let ct_issue = report
        .issues
        .iter()
        .find(|i| i.variable.as_deref() == Some("SEX") && i.codelist_code.is_some());
    assert!(ct_issue.is_some(), "Should emit CT issue for invalid SEX");
}

#[test]
fn rule_engine_builds_from_context() {
    let domains = load_default_sdtm_ig_domains().expect("standards");
    let ct_registry = load_default_ct_registry().expect("ct");

    let ctx = ValidationContext::new().with_ct_registry(&ct_registry);

    let engine = ctx.build_rule_engine(&domains);

    // Should have rules for DM
    let dm_rules = engine.rules_for_domain("DM");
    assert!(!dm_rules.is_empty(), "Should have rules for DM domain");

    // Should have Required rules (SDTMIG_NULL)
    assert!(
        dm_rules.iter().any(|r| r.category == "SDTMIG_NULL"),
        "Should have SDTMIG_NULL rules"
    );
}
