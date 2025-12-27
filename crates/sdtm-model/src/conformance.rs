use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Reject,
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceIssue {
    pub code: String,
    pub message: String,
    pub severity: IssueSeverity,
    pub variable: Option<String>,
    pub count: Option<u64>,
    pub rule_id: Option<String>,
    pub category: Option<String>,
    pub codelist_code: Option<String>,
    pub ct_source: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConformanceReport {
    #[serde(rename = "domain")]
    pub domain_code: String,
    pub issues: Vec<ConformanceIssue>,
}

impl ConformanceReport {
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| matches!(issue.severity, IssueSeverity::Error | IssueSeverity::Reject))
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == IssueSeverity::Warning)
            .count()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }
}
