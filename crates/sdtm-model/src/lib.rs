pub mod conformance;
pub mod domain;
pub mod error;
pub mod mapping;
pub mod processing;
pub mod terminology;

pub use conformance::{ConformanceIssue, ConformanceReport, IssueSeverity};
pub use domain::{DatasetMetadata, Domain, Variable, VariableType};
pub use error::{Result, SdtmError};
pub use mapping::{ColumnHint, MappingConfig, MappingSuggestion};
pub use processing::{
    DomainResult, OutputFormat, OutputPaths, ProcessStudyRequest, ProcessStudyResponse,
    StudyError,
};
pub use terminology::{ControlledTerminology, CtRegistry};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conformance_report_counts() {
        let report = ConformanceReport {
            domain_code: "AE".to_string(),
            issues: vec![
                ConformanceIssue {
                    code: "AE001".to_string(),
                    message: "Missing AE term".to_string(),
                    severity: IssueSeverity::Error,
                    variable: Some("AETERM".to_string()),
                    count: Some(2),
                },
                ConformanceIssue {
                    code: "AE002".to_string(),
                    message: "Unexpected value".to_string(),
                    severity: IssueSeverity::Warning,
                    variable: Some("AESEV".to_string()),
                    count: Some(1),
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
        let round: ProcessStudyResponse =
            serde_json::from_str(&json).expect("deserialize response");
        assert_eq!(round.study_id, "STUDY");
    }
}
