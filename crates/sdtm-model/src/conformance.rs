//! Validation conformance types for SDTM compliance checking.
//!
//! This module provides types for representing validation issues and reports
//! generated during SDTM conformance checking.
//!
//! # SDTMIG Reference
//!
//! - Chapter 10: Controlled Terminology
//! - Appendix C: Validation Rules

use serde::{Deserialize, Serialize};

/// Severity level for validation issues.
///
/// Determines how validation findings should be handled:
/// - `Reject` - Critical issue that prevents submission
/// - `Error` - Non-extensible CT violation or required field missing
/// - `Warning` - Extensible CT deviation or best practice violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Critical issue - submission will be rejected.
    Reject,
    /// Error - requires correction before submission.
    Error,
    /// Warning - should be reviewed but may be acceptable.
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
