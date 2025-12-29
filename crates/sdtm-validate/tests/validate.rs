#![allow(missing_docs)]

use polars::prelude::{Column, DataFrame};

use sdtm_model::{OutputFormat, Severity, ValidationIssue, ValidationReport};
use sdtm_standards::{load_default_ct_registry, load_default_sdtm_ig_domains};
use sdtm_validate::{gate_strict_outputs, strict_outputs_requested, validate_domain};

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
    let report = validate_domain(domain, &df, Some(&ct_registry));
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
        Severity::Warning
    } else {
        Severity::Error
    };
    assert_eq!(issue.code, ct.code);
    assert_eq!(issue.severity, expected);
}

#[test]
fn strict_output_gate_blocks_on_errors() {
    let report = ValidationReport {
        domain_code: "AE".to_string(),
        issues: vec![ValidationIssue {
            code: "SD0002".to_string(),
            message: "missing".to_string(),
            severity: Severity::Error,
            variable: Some("AETERM".to_string()),
            count: Some(1),
            ct_source: None,
            observed_values: None,
            allowed_values: None,
            allowed_count: None,
            ct_examples: None,
        }],
    };
    let decision = gate_strict_outputs(&[OutputFormat::Xpt], true, &[report]);
    assert!(decision.block_strict_outputs);
    assert_eq!(decision.blocking_domains, vec!["AE".to_string()]);
}

#[test]
fn strict_output_gate_ignored_without_strict_formats() {
    let report = ValidationReport {
        domain_code: "AE".to_string(),
        issues: vec![ValidationIssue {
            code: "SD0002".to_string(),
            message: "missing".to_string(),
            severity: Severity::Error,
            variable: Some("AETERM".to_string()),
            count: Some(1),
            ct_source: None,
            observed_values: None,
            allowed_values: None,
            allowed_count: None,
            ct_examples: None,
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
