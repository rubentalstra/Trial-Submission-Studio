use std::fs;
use std::path::PathBuf;

use polars::prelude::{Column, DataFrame};

use sdtm_model::{ConformanceIssue, ConformanceReport, IssueSeverity, OutputFormat};
use sdtm_standards::{load_default_ct_registry, load_default_sdtm_ig_domains};
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
        .find(|issue| issue.ct_source.is_some())
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
    assert_eq!(issue.code, ct.code);
    assert_eq!(issue.severity, expected);
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
