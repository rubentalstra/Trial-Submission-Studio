use std::fs;
use std::path::PathBuf;

use polars::prelude::{Column, DataFrame};

use sdtm_model::{ConformanceIssue, ConformanceReport, IssueSeverity, OutputFormat, VariableType};
use sdtm_standards::{
    load_default_ct_registry, load_default_p21_rules, load_default_sdtm_ig_domains,
};
use sdtm_validate::{
    ValidationContext, gate_strict_outputs, strict_outputs_requested, validate_domain,
    write_conformance_report_json,
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
    assert!(
        report.issues.iter().any(
            |issue| issue.code == "SD0056" && issue.variable.as_deref() == Some(&required.name)
        )
    );
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
        .find(|issue| issue.code == "SD0002")
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
        .find(|issue| issue.code == "SD1230")
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
    let p21_rules = load_default_p21_rules().expect("p21");
    let ctx = ValidationContext::new()
        .with_ct_registry(&ct_registry)
        .with_p21_rules(&p21_rules);
    let report = validate_domain(domain, &df, &ctx);
    let issue = report
        .issues
        .iter()
        .find(|issue| issue.code == "CT2001" || issue.code == "CT2002")
        .expect("ct issue");
    let ct_code = variable.codelist_code.as_deref().unwrap_or("");
    let ct = ct_registry
        .by_code
        .get(&ct_code.to_uppercase())
        .or_else(|| ct_registry.by_submission.get(&variable.name.to_uppercase()))
        .or_else(|| ct_registry.by_name.get(&variable.name.to_uppercase()))
        .expect("ct lookup");
    let expected = if ct.extensible {
        IssueSeverity::Warning
    } else {
        IssueSeverity::Error
    };
    let expected_code = if ct.extensible { "CT2002" } else { "CT2001" };
    assert_eq!(issue.code, expected_code);
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
            rule_id: None,
            category: None,
            codelist_code: None,
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
            rule_id: None,
            category: None,
            codelist_code: None,
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
