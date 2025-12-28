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
                // rule_id removed
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
                // rule_id removed
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

// --- IssueSummary tests ---

#[test]
fn issue_summary_from_empty_reports() {
    use sdtm_model::IssueSummary;

    let reports: Vec<ConformanceReport> = vec![];
    let summary = IssueSummary::from_reports(&reports);

    assert_eq!(summary.total_errors, 0);
    assert_eq!(summary.total_warnings, 0);
    assert_eq!(summary.total_rejects, 0);
    assert!(summary.by_domain.is_empty());
    assert!(summary.by_category.is_empty());
}

#[test]
fn issue_summary_counts_severities() {
    use sdtm_model::IssueSummary;

    let reports = vec![
        ConformanceReport {
            domain_code: "AE".to_string(),
            issues: vec![
                ConformanceIssue {
                    code: "SD0002".to_string(),
                    message: "Error 1".to_string(),
                    severity: IssueSeverity::Error,
                    variable: None,
                    count: Some(2),
                    // rule_id removed
                    category: None,
                    codelist_code: None,
                    ct_source: None,
                },
                ConformanceIssue {
                    code: "SD0057".to_string(),
                    message: "Warning 1".to_string(),
                    severity: IssueSeverity::Warning,
                    variable: None,
                    count: Some(3),
                    // rule_id removed
                    category: None,
                    codelist_code: None,
                    ct_source: None,
                },
            ],
        },
        ConformanceReport {
            domain_code: "DM".to_string(),
            issues: vec![ConformanceIssue {
                code: "SD0056".to_string(),
                message: "Reject 1".to_string(),
                severity: IssueSeverity::Reject,
                variable: None,
                count: Some(1),
                // rule_id removed
                category: None,
                codelist_code: None,
                ct_source: None,
            }],
        },
    ];

    let summary = IssueSummary::from_reports(&reports);

    assert_eq!(summary.total_errors, 1);
    assert_eq!(summary.total_warnings, 1);
    assert_eq!(summary.total_rejects, 1);
}

#[test]
fn issue_summary_groups_by_domain() {
    use sdtm_model::IssueSummary;

    let reports = vec![
        ConformanceReport {
            domain_code: "AE".to_string(),
            issues: vec![ConformanceIssue {
                code: "SD0002".to_string(),
                message: "AE issue".to_string(),
                severity: IssueSeverity::Error,
                variable: None,
                count: Some(1),
                // rule_id removed
                category: None,
                codelist_code: None,
                ct_source: None,
            }],
        },
        ConformanceReport {
            domain_code: "DM".to_string(),
            issues: vec![ConformanceIssue {
                code: "SD0056".to_string(),
                message: "DM issue".to_string(),
                severity: IssueSeverity::Warning,
                variable: None,
                count: Some(1),
                // rule_id removed
                category: None,
                codelist_code: None,
                ct_source: None,
            }],
        },
    ];

    let summary = IssueSummary::from_reports(&reports);

    assert!(summary.by_domain.contains_key("AE"));
    assert!(summary.by_domain.contains_key("DM"));
    assert_eq!(summary.by_domain.get("AE").unwrap().error_count, 1);
    assert_eq!(summary.by_domain.get("DM").unwrap().warning_count, 1);
}

#[test]
fn issue_summary_groups_by_rule() {
    use sdtm_model::IssueSummary;

    let reports = vec![ConformanceReport {
        domain_code: "AE".to_string(),
        issues: vec![
            ConformanceIssue {
                code: "SD0002".to_string(),
                message: "First SD0002".to_string(),
                severity: IssueSeverity::Error,
                variable: None,
                count: Some(1),
                category: Some("Completeness".to_string()),
                codelist_code: None,
                ct_source: None,
            },
            ConformanceIssue {
                code: "SD0002".to_string(),
                message: "Second SD0002".to_string(),
                severity: IssueSeverity::Error,
                variable: None,
                count: Some(2),
                category: Some("Completeness".to_string()),
                codelist_code: None,
                ct_source: None,
            },
        ],
    }];

    let summary = IssueSummary::from_reports(&reports);

    assert!(summary.by_category.contains_key("Completeness"));
    let rule_summary = summary.by_category.get("Completeness").unwrap();
    assert_eq!(rule_summary.error_count, 2); // 2 issues of this category
}

#[test]
fn issue_summary_extracts_samples() {
    use sdtm_model::IssueSummary;

    let reports = vec![ConformanceReport {
        domain_code: "AE".to_string(),
        issues: vec![ConformanceIssue {
            code: "CT2001".to_string(),
            message: "Invalid CT value 'BADVAL' for AESEV; values: VAL1, VAL2, VAL3".to_string(),
            severity: IssueSeverity::Error,
            variable: Some("AESEV".to_string()),
            count: Some(1),
            category: Some("Terminology".to_string()),
            codelist_code: None,
            ct_source: None,
        }],
    }];

    let summary = IssueSummary::from_reports(&reports);

    // Check if category is tracked and samples may be extracted
    assert!(summary.by_category.contains_key("Terminology"));
}

#[test]
fn issue_summary_serializes() {
    use sdtm_model::IssueSummary;

    let reports = vec![ConformanceReport {
        domain_code: "AE".to_string(),
        issues: vec![ConformanceIssue {
            code: "SD0002".to_string(),
            message: "Test issue".to_string(),
            severity: IssueSeverity::Error,
            variable: None,
            count: Some(1),
            // rule_id removed
            category: None,
            codelist_code: None,
            ct_source: None,
        }],
    }];

    let summary = IssueSummary::from_reports(&reports);
    let json = serde_json::to_string(&summary).expect("serialize summary");
    let round: IssueSummary = serde_json::from_str(&json).expect("deserialize summary");

    assert_eq!(round.total_errors, summary.total_errors);
    assert_eq!(round.total_warnings, summary.total_warnings);
}
