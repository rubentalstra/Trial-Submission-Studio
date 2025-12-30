//! Tests for SAS output generation.

use sdtm_model::{Domain, Variable, VariableType};
use sdtm_output::sas::{default_assignment, keep_clause};

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

fn test_domain() -> Domain {
    Domain {
        code: "AE".to_string(),
        description: Some("Adverse Events".to_string()),
        class_name: Some("Events".to_string()),
        dataset_class: None,
        label: Some("Adverse Events".to_string()),
        structure: Some("One record per event".to_string()),
        dataset_name: None,
        variables: vec![
            {
                let mut v = test_variable("STUDYID", VariableType::Char);
                v.role = Some("Identifier".to_string());
                v.core = Some("Req".to_string());
                v
            },
            {
                let mut v = test_variable("USUBJID", VariableType::Char);
                v.role = Some("Identifier".to_string());
                v.core = Some("Req".to_string());
                v
            },
            {
                let mut v = test_variable("AETERM", VariableType::Char);
                v.role = Some("Topic".to_string());
                v.core = Some("Req".to_string());
                v
            },
            {
                let mut v = test_variable("AESTDTC", VariableType::Char);
                v.role = Some("Timing".to_string());
                v
            },
        ],
    }
}

#[test]
fn test_keep_clause() {
    let domain = test_domain();
    let clause = keep_clause(&domain);

    assert!(clause.contains("STUDYID"));
    assert!(clause.contains("USUBJID"));
    assert!(clause.contains("AETERM"));
    assert!(clause.contains("AESTDTC"));
}

#[test]
fn test_default_assignment_char() {
    let variable = test_variable("AETERM", VariableType::Char);
    assert_eq!(default_assignment(&variable), "AETERM = '';");
}

#[test]
fn test_default_assignment_num() {
    let variable = test_variable("AESEQ", VariableType::Num);
    assert_eq!(default_assignment(&variable), "AESEQ = .;");
}
