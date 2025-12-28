use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Reject,
    Error,
    Warning,
}

/// A validation issue found during validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Codelist code (e.g., "C66742").
    pub code: String,
    /// Human-readable message describing the issue.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// Variable name (if applicable).
    pub variable: Option<String>,
    /// Count of occurrences.
    pub count: Option<u64>,
    /// CT source identifier (e.g., "SDTM CT").
    pub ct_source: Option<String>,
    /// Sample observed values from the source data that failed CT validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_values: Option<Vec<String>>,
    /// Allowed CT values (only populated for small codelists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<String>>,
    /// Count of allowed CT values (for large codelists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_count: Option<u64>,
    /// Sample CT values from the codelist (for large codelists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ct_examples: Option<Vec<String>>,
}

/// Validation report for a single domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    #[serde(rename = "domain")]
    pub domain_code: String,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| matches!(issue.severity, Severity::Error | Severity::Reject))
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Warning)
            .count()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }
}
