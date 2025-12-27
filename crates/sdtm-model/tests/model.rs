//! Tests for sdtm-model types.

use sdtm_model::{ConformanceIssue, ConformanceReport, IssueSeverity, ProcessStudyResponse};

#[test]
fn conformance_report_counts() {
    let report = ConformanceReport {
        domain_code: "AE".to_string(),
        issues: vec![
            ConformanceIssue {
                code: "SD0002".to_string(),
                message: "Missing AE term".to_string(),
                severity: IssueSeverity::Error,
                variable: Some("AETERM".to_string()),
                count: Some(2),
                rule_id: None,
                category: None,
                codelist_code: None,
                ct_source: None,
            },
            ConformanceIssue {
                code: "SD0057".to_string(),
                message: "Unexpected value".to_string(),
                severity: IssueSeverity::Warning,
                variable: Some("AESEV".to_string()),
                count: Some(1),
                rule_id: None,
                category: None,
                codelist_code: None,
                ct_source: None,
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
        conformance_report_path: None,
    };
    let json = serde_json::to_string(&response).expect("serialize response");
    let round: ProcessStudyResponse = serde_json::from_str(&json).expect("deserialize response");
    assert_eq!(round.study_id, "STUDY");
}
