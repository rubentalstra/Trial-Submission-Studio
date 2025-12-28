use std::fs;
use std::path::PathBuf;

use polars::prelude::{Column, DataFrame};

use sdtm_model::{ConformanceIssue, ConformanceReport, IssueSeverity, OutputFormat};
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
        .resolve(ct_code, None)
        .map(|resolved| resolved.codelist)
        .expect("ct lookup");
    let expected = if ct.extensible {
        IssueSeverity::Warning
    } else {
        IssueSeverity::Error
    };
    let _expected_rule = if ct.extensible { "CT2002" } else { "CT2001" };
    // rule_id check removed - check code/category instead
    assert_eq!(issue.code, ct.code);
    assert_eq!(issue.severity, expected);
    assert_eq!(issue.codelist_code.as_deref(), Some(ct.code.as_str()));
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

    // Should find Required Variable Missing issue
    let studyid_issue = report.issues.iter().find(|i| {
        i.variable.as_deref() == Some("STUDYID") && i.code == "Required Variable Missing"
    });
    assert!(
        studyid_issue.is_some(),
        "Should emit Required Variable Missing issue for missing STUDYID"
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

    // Should find Required Value Missing issue
    let null_issue = report.issues.iter().find(|i| {
        i.variable.as_deref() == Some("STUDYID")
            && i.category.as_deref() == Some("Required Value Missing")
    });
    assert!(
        null_issue.is_some(),
        "Should emit Required Value Missing issue for null STUDYID values"
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

    // Should have Required Value Missing rules
    assert!(
        dm_rules
            .iter()
            .any(|r| r.category == "Required Value Missing"),
        "Should have Required Value Missing rules"
    );
}
