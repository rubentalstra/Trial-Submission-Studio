//! Tests for sdtm-model types.

use sdtm_model::{
    CheckType, Domain, ProcessStudyResponse, Severity, ValidationIssue, ValidationReport, Variable,
    VariableType,
};

#[test]
fn validation_report_counts() {
    let report = ValidationReport {
        domain_code: "AE".to_string(),
        issues: vec![
            ValidationIssue {
                check_type: Some(CheckType::RequiredVariableEmpty),
                code: "SD0002".to_string(),
                message: "Missing AE term".to_string(),
                severity: Severity::Error,
                variable: Some("AETERM".to_string()),
                count: Some(2),
                ct_source: None,
                observed_values: None,
                allowed_values: None,
                allowed_count: None,
                ct_examples: None,
            },
            ValidationIssue {
                check_type: Some(CheckType::ControlledTerminology),
                code: "SD0057".to_string(),
                message: "Unexpected value".to_string(),
                severity: Severity::Warning,
                variable: Some("AESEV".to_string()),
                count: Some(1),
                ct_source: None,
                observed_values: None,
                allowed_values: None,
                allowed_count: None,
                ct_examples: None,
            },
        ],
    };
    assert_eq!(report.error_count(), 1);
    assert_eq!(report.warning_count(), 1);
    assert!(report.has_errors());
}

#[test]
fn response_serializes() {
    let response = ProcessStudyResponse {
        success: true,
        study_id: "STUDY".to_string(),
        output_dir: "output".into(),
        domain_results: vec![],
        errors: vec![],
    };
    let json = serde_json::to_string(&response).expect("serialize response");
    let round: ProcessStudyResponse = serde_json::from_str(&json).expect("deserialize response");
    assert_eq!(round.study_id, "STUDY");
}

#[test]
fn validation_issue_serializes() {
    let issue = ValidationIssue {
        check_type: Some(CheckType::ControlledTerminology),
        code: "C66742".to_string(),
        message: "Invalid value".to_string(),
        severity: Severity::Error,
        variable: Some("SEX".to_string()),
        count: Some(3),
        ct_source: Some("SDTM CT".to_string()),
        observed_values: Some(vec!["OTHER".to_string()]),
        allowed_values: Some(vec!["F".to_string(), "M".to_string()]),
        allowed_count: None,
        ct_examples: None,
    };
    let json = serde_json::to_string(&issue).expect("serialize issue");
    let round: ValidationIssue = serde_json::from_str(&json).expect("deserialize issue");
    assert_eq!(round.code, "C66742");
    assert_eq!(round.ct_source.as_deref(), Some("SDTM CT"));
}

#[test]
fn validation_report_no_errors() {
    let report = ValidationReport {
        domain_code: "DM".to_string(),
        issues: vec![ValidationIssue {
            check_type: Some(CheckType::ControlledTerminology),
            code: "C66742".to_string(),
            message: "Warning only".to_string(),
            severity: Severity::Warning,
            variable: None,
            count: None,
            ct_source: None,
            observed_values: None,
            allowed_values: None,
            allowed_count: None,
            ct_examples: None,
        }],
    };
    assert_eq!(report.error_count(), 0);
    assert_eq!(report.warning_count(), 1);
    assert!(!report.has_errors());
}

#[test]
fn validation_report_with_reject() {
    let report = ValidationReport {
        domain_code: "AE".to_string(),
        issues: vec![ValidationIssue {
            check_type: Some(CheckType::RequiredVariableMissing),
            code: "FATAL".to_string(),
            message: "Critical error".to_string(),
            severity: Severity::Reject,
            variable: None,
            count: Some(1),
            ct_source: None,
            observed_values: None,
            allowed_values: None,
            allowed_count: None,
            ct_examples: None,
        }],
    };
    assert_eq!(report.error_count(), 1); // Reject counts as error
    assert!(report.has_errors());
}

fn make_variable(name: &str, role: &str, order: u32) -> Variable {
    Variable {
        name: name.to_string(),
        label: None,
        data_type: VariableType::Char,
        length: Some(200),
        role: Some(role.to_string()),
        core: Some("Req".to_string()),
        codelist_code: None,
        order: Some(order),
    }
}

#[test]
fn domain_variables_by_role_sorts_by_role_then_order() {
    let domain = Domain {
        code: "AE".to_string(),
        description: None,
        class_name: None,
        dataset_class: None,
        label: None,
        structure: None,
        dataset_name: None,
        variables: vec![
            make_variable("AESTDTC", "Timing", 53),
            make_variable("AETERM", "Topic", 9),
            make_variable("AESEV", "Record Qualifier", 27),
            make_variable("STUDYID", "Identifier", 1),
        ],
    };

    let ordered = domain.variables_by_role();
    let names: Vec<&str> = ordered.iter().map(|var| var.name.as_str()).collect();
    assert_eq!(names, vec!["STUDYID", "AETERM", "AESEV", "AESTDTC"]);
}
