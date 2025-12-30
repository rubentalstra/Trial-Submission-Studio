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
#[non_exhaustive]
pub enum Severity {
    /// Critical issue - submission will be rejected.
    Reject,
    /// Error - requires correction before submission.
    Error,
    /// Warning - should be reviewed but may be acceptable.
    Warning,
}

/// Type of validation check that generated the issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CheckType {
    /// Controlled terminology validation.
    ControlledTerminology,
    /// Required variable is missing from dataset.
    RequiredVariableMissing,
    /// Required variable has null/empty values.
    RequiredVariableEmpty,
    /// Expected variable is missing from dataset.
    ExpectedVariableMissing,
    /// Data type mismatch (numeric vs character).
    DataTypeMismatch,
    /// Invalid date/time format (not ISO 8601).
    InvalidDateFormat,
    /// Duplicate sequence number detected.
    DuplicateSequence,
    /// Text value exceeds maximum length.
    TextLengthExceeded,
    /// Identifier variable has null values.
    IdentifierNull,
}

impl CheckType {
    /// Returns a human-readable label for the check type.
    pub fn label(&self) -> &'static str {
        match self {
            CheckType::ControlledTerminology => "Controlled Terminology",
            CheckType::RequiredVariableMissing => "Required Variable Missing",
            CheckType::RequiredVariableEmpty => "Required Variable Empty",
            CheckType::ExpectedVariableMissing => "Expected Variable Missing",
            CheckType::DataTypeMismatch => "Data Type Mismatch",
            CheckType::InvalidDateFormat => "Invalid Date Format",
            CheckType::DuplicateSequence => "Duplicate Sequence",
            CheckType::TextLengthExceeded => "Text Length Exceeded",
            CheckType::IdentifierNull => "Identifier Null",
        }
    }
}

/// A validation issue found during validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Type of validation check that found this issue.
    #[serde(default)]
    pub check_type: Option<CheckType>,
    /// Issue code (codelist code for CT, rule code for others).
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
    /// Sample observed values from the source data that failed validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_values: Option<Vec<String>>,
    /// Allowed values (for CT or enum validation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<String>>,
    /// Count of allowed values (for large value sets).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_count: Option<u64>,
    /// Sample allowed values (for large value sets).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ct_examples: Option<Vec<String>>,
}

/// Validation report for a single domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    /// The domain code (e.g., "AE", "DM") this report covers.
    #[serde(rename = "domain")]
    pub domain_code: String,
    /// List of validation issues found in this domain.
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// Returns the count of error-level issues (Error or Reject severity).
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| matches!(issue.severity, Severity::Error | Severity::Reject))
            .count()
    }

    /// Returns the count of warning-level issues.
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Warning)
            .count()
    }

    /// Returns true if there are any error-level issues.
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| matches!(issue.severity, Severity::Error | Severity::Reject))
    }
}
