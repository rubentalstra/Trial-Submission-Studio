//! Tests for common output utilities.

use polars::prelude::{Column, DataFrame, IntoColumn, NamedFrom, Series};
use sdtm_model::{Domain, Variable, VariableType};
use sdtm_output::{
    SAS_NUMERIC_LEN, VariableTypeExt, dataset_name, has_collected_data, is_expected, is_identifier,
    is_reference_domain, is_required, normalize_study_id, should_upcase, variable_length,
};

fn test_df(columns: Vec<(&str, Vec<&str>)>) -> DataFrame {
    let cols: Vec<Column> = columns
        .into_iter()
        .map(|(name, values)| {
            Series::new(
                name.into(),
                values.iter().copied().map(String::from).collect::<Vec<_>>(),
            )
            .into_column()
        })
        .collect();
    DataFrame::new(cols).unwrap()
}

fn test_variable(name: &str, data_type: VariableType) -> Variable {
    Variable {
        name: name.to_string(),
        label: Some(format!("{} Label", name)),
        data_type,
        length: None,
        role: None,
        core: None,
        codelist_code: None,
        order: None,
        described_value_domain: None,
    }
}

fn test_domain(code: &str, class_name: Option<&str>) -> Domain {
    Domain {
        code: code.to_string(),
        description: Some(format!("{} Domain", code)),
        class_name: class_name.map(String::from),
        dataset_class: None,
        label: Some(format!("{} Label", code)),
        structure: None,
        dataset_name: None,
        variables: vec![],
    }
}

#[test]
fn test_dataset_name_uses_code_when_no_override() {
    let domain = test_domain("AE", None);
    assert_eq!(dataset_name(&domain), "AE");
}

#[test]
fn test_dataset_name_uses_override() {
    let mut domain = test_domain("FA", None);
    domain.dataset_name = Some("FACM".to_string());
    assert_eq!(dataset_name(&domain), "FACM");
}

#[test]
fn test_normalize_study_id_trims_whitespace() {
    assert_eq!(normalize_study_id("  STUDY01  "), "STUDY01");
}

#[test]
fn test_normalize_study_id_defaults_empty() {
    assert_eq!(normalize_study_id(""), "STUDY");
    assert_eq!(normalize_study_id("   "), "STUDY");
}

#[test]
fn test_is_reference_domain_trial_design() {
    let domain = test_domain("TA", Some("Trial Design"));
    assert!(is_reference_domain(&domain));
}

#[test]
fn test_is_reference_domain_study_reference() {
    let domain = test_domain("OI", Some("Study Reference"));
    assert!(is_reference_domain(&domain));
}

#[test]
fn test_is_reference_domain_findings() {
    let domain = test_domain("AE", Some("Events"));
    assert!(!is_reference_domain(&domain));
}

#[test]
fn test_is_reference_domain_none_class() {
    let domain = test_domain("AE", None);
    assert!(!is_reference_domain(&domain));
}

#[test]
fn test_is_reference_domain_normalizes_class_name() {
    let domain = test_domain("TA", Some("trial-design"));
    assert!(is_reference_domain(&domain));

    let domain2 = test_domain("TA", Some("TRIAL_DESIGN"));
    assert!(is_reference_domain(&domain2));
}

#[test]
fn test_variable_length_uses_explicit_length() {
    let df = test_df(vec![("AETERM", vec!["short"])]);
    let mut variable = test_variable("AETERM", VariableType::Char);
    variable.length = Some(200);

    let length = variable_length(&variable, &df).unwrap();
    assert_eq!(length, 200);
}

#[test]
fn test_variable_length_numeric_always_8() {
    let df = test_df(vec![("AESTDY", vec!["1"])]);
    let variable = test_variable("AESTDY", VariableType::Num);

    let length = variable_length(&variable, &df).unwrap();
    assert_eq!(length, SAS_NUMERIC_LEN);
}

#[test]
fn test_variable_length_computes_max_from_data() {
    let df = test_df(vec![("AETERM", vec!["short", "medium length", "x"])]);
    let variable = test_variable("AETERM", VariableType::Char);

    let length = variable_length(&variable, &df).unwrap();
    assert_eq!(length, 13); // "medium length" has 13 chars
}

#[test]
fn test_variable_length_minimum_one() {
    let df = test_df(vec![("AETERM", vec!["", "", ""])]);
    let variable = test_variable("AETERM", VariableType::Char);

    let length = variable_length(&variable, &df).unwrap();
    assert_eq!(length, 1);
}

#[test]
fn test_is_required() {
    let mut variable = test_variable("USUBJID", VariableType::Char);
    variable.core = Some("Req".to_string());
    assert!(is_required(&variable));

    variable.core = Some("REQ".to_string());
    assert!(is_required(&variable));

    variable.core = Some("Exp".to_string());
    assert!(!is_required(&variable));

    variable.core = None;
    assert!(!is_required(&variable));
}

#[test]
fn test_is_identifier() {
    let mut variable = test_variable("USUBJID", VariableType::Char);
    variable.role = Some("Identifier".to_string());
    assert!(is_identifier(&variable));

    variable.role = Some("IDENTIFIER".to_string());
    assert!(is_identifier(&variable));

    variable.role = Some("Topic".to_string());
    assert!(!is_identifier(&variable));

    variable.role = None;
    assert!(!is_identifier(&variable));
}

#[test]
fn test_should_upcase_identifier() {
    let mut variable = test_variable("USUBJID", VariableType::Char);
    variable.role = Some("Identifier".to_string());
    assert!(should_upcase(&variable));
}

#[test]
fn test_should_upcase_with_codelist() {
    let mut variable = test_variable("SEX", VariableType::Char);
    variable.codelist_code = Some("C66731".to_string());
    assert!(should_upcase(&variable));
}

#[test]
fn test_should_upcase_neither() {
    let variable = test_variable("AETERM", VariableType::Char);
    assert!(!should_upcase(&variable));
}

#[test]
fn test_is_expected() {
    assert!(is_expected(Some("Exp")));
    assert!(is_expected(Some("EXP")));
    assert!(is_expected(Some("  exp  ")));
    assert!(!is_expected(Some("Req")));
    assert!(!is_expected(None));
}

#[test]
fn test_has_collected_data_with_values() {
    let df = test_df(vec![("AETERM", vec!["Headache", "", "Nausea"])]);
    assert!(has_collected_data(&df, "AETERM"));
}

#[test]
fn test_has_collected_data_all_empty() {
    let df = test_df(vec![("AETERM", vec!["", "  ", ""])]);
    assert!(!has_collected_data(&df, "AETERM"));
}

#[test]
fn test_has_collected_data_missing_column() {
    let df = test_df(vec![("AETERM", vec!["Headache"])]);
    assert!(!has_collected_data(&df, "AEOTHER"));
}

#[test]
fn test_variable_type_ext() {
    assert_eq!(VariableType::Char.as_define_type(), "text");
    assert_eq!(VariableType::Num.as_define_type(), "float");
}
