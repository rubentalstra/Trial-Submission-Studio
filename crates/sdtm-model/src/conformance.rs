use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Reject,
    Error,
    Warning,
}

/// A conformance issue found during validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceIssue {
    /// Codelist code (e.g., "C66742").
    pub code: String,
    /// Human-readable message describing the issue.
    pub message: String,
    /// Severity level.
    pub severity: IssueSeverity,
    /// Variable name (if applicable).
    pub variable: Option<String>,
    /// Count of occurrences.
    pub count: Option<u64>,
    /// CT source identifier (e.g., "SDTM CT").
    pub ct_source: Option<String>,
}

/// Conformance report for a single domain.
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
