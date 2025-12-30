//! Unit tests for validation functions.

use polars::prelude::*;
use sdtm_model::{CoreDesignation, Domain, Variable, VariableType};
use sdtm_validate::{CheckType, Severity, validate_domain};

fn make_domain(variables: Vec<Variable>) -> Domain {
    Domain {
        code: "AE".to_string(),
        description: None,
        class_name: None,
        dataset_class: None,
        label: None,
        structure: None,
        dataset_name: None,
        variables,
    }
}

fn make_variable(name: &str, core: Option<CoreDesignation>, data_type: VariableType) -> Variable {
    Variable {
        name: name.to_string(),
        label: None,
        data_type,
        length: None,
        role: None,
        core,
        codelist_code: None,
        order: None,
        described_value_domain: None,
    }
}

#[test]
fn test_required_variable_missing() {
    let domain = make_domain(vec![make_variable(
        "USUBJID",
        Some(CoreDesignation::Required),
        VariableType::Char,
    )]);

    let df = DataFrame::new(vec![Series::new("OTHER".into(), vec!["A"]).into()]).unwrap();

    let report = validate_domain(&domain, &df, None);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].check_type,
        Some(CheckType::RequiredVariableMissing)
    );
}

#[test]
fn test_required_variable_empty() {
    let domain = make_domain(vec![make_variable(
        "USUBJID",
        Some(CoreDesignation::Required),
        VariableType::Char,
    )]);

    let df = DataFrame::new(vec![Series::new("USUBJID".into(), vec!["A", ""]).into()]).unwrap();

    let report = validate_domain(&domain, &df, None);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].check_type,
        Some(CheckType::RequiredVariableEmpty)
    );
}

#[test]
fn test_expected_variable_missing() {
    let domain = make_domain(vec![make_variable(
        "AETERM",
        Some(CoreDesignation::Expected),
        VariableType::Char,
    )]);

    let df = DataFrame::new(vec![Series::new("OTHER".into(), vec!["A"]).into()]).unwrap();

    let report = validate_domain(&domain, &df, None);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].check_type,
        Some(CheckType::ExpectedVariableMissing)
    );
    assert_eq!(report.issues[0].severity, Severity::Warning);
}

#[test]
fn test_iso8601_date_validation() {
    use sdtm_validate::ISO8601_DATE_REGEX;

    // Test valid dates pass
    assert!(ISO8601_DATE_REGEX.is_match("2024"));
    assert!(ISO8601_DATE_REGEX.is_match("2024-01"));
    assert!(ISO8601_DATE_REGEX.is_match("2024-01-15"));
    assert!(ISO8601_DATE_REGEX.is_match("2024-01-15T10:30"));
    assert!(ISO8601_DATE_REGEX.is_match("2024-01-15T10:30:45"));

    // Test invalid dates fail
    assert!(!ISO8601_DATE_REGEX.is_match("01/15/2024"));
    assert!(!ISO8601_DATE_REGEX.is_match("15-01-2024"));
    assert!(!ISO8601_DATE_REGEX.is_match("2024/01/15"));
}

#[test]
fn test_date_variable_detection() {
    use sdtm_validate::is_date_variable;

    assert!(is_date_variable("AESTDTC"));
    assert!(is_date_variable("AEENDTC"));
    assert!(is_date_variable("DMDTC"));
    assert!(!is_date_variable("AETERM"));
    assert!(!is_date_variable("USUBJID"));
}
