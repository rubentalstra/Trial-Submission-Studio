#![allow(missing_docs)]

use polars::prelude::{Column, DataFrame};

use sdtm_standards::{load_default_ct_registry, load_default_sdtm_ig_domains};
use sdtm_validate::{Severity, rule_ids, validate_domain};

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

    // P21 uses CT2001 for non-extensible, CT2002 for extensible codelists
    let (expected_code, expected_severity) = if ct.extensible {
        (rule_ids::CT2002, Severity::Warning)
    } else {
        (rule_ids::CT2001, Severity::Error)
    };
    assert_eq!(issue.code, expected_code);
    assert_eq!(issue.severity, expected_severity);
}
