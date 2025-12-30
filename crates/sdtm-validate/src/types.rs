//! Validation conformance types for SDTM compliance checking.
//!
//! This module provides types for representing validation issues and reports
//! generated during SDTM conformance checking using Pinnacle 21 rules.
//!
//! # Pinnacle 21 Integration
//!
//! All validation checks map to official P21 rule IDs (e.g., CT2001, SD0002).
//! See the `p21` module for rule definitions and the `p21::rule_ids` module
//! for compile-time rule ID constants.

use crate::p21::{rule_ids, P21Category};
use serde::{Deserialize, Serialize};

/// Severity level for validation issues.
///
/// Maps to P21 severity levels:
/// - `Reject` - Critical issue that prevents submission (P21: Reject)
/// - `Error` - Non-extensible CT violation or required field missing (P21: Error)
/// - `Warning` - Extensible CT deviation or best practice violation (P21: Warning)
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
///
/// Each variant maps to one or more Pinnacle 21 rule IDs.
/// Use `p21_rule_id()` to get the official P21 rule code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CheckType {
    /// Controlled terminology validation (CT2001/CT2002).
    ControlledTerminology,
    /// Required variable is missing from dataset (SD0056).
    RequiredVariableMissing,
    /// Required variable has null/empty values (SD0002).
    RequiredVariableEmpty,
    /// Expected variable is missing from dataset (SD0057).
    ExpectedVariableMissing,
    /// Data type mismatch - numeric vs character (SD0055).
    DataTypeMismatch,
    /// Invalid date/time format - not ISO 8601 (SD0003).
    InvalidDateFormat,
    /// Duplicate sequence number detected (SD0005).
    DuplicateSequence,
    /// Text value exceeds maximum length (SD0017).
    TextLengthExceeded,
    /// Identifier variable has null values (SD0002).
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

    /// Returns the Pinnacle 21 rule ID for this check type.
    ///
    /// For CT checks, returns the base rule - caller should use CT2001 or CT2002
    /// based on codelist extensibility.
    pub fn p21_rule_id(&self) -> &'static str {
        match self {
            CheckType::ControlledTerminology => rule_ids::CT2001, // or CT2002 if extensible
            CheckType::RequiredVariableMissing => rule_ids::SD0056,
            CheckType::RequiredVariableEmpty => rule_ids::SD0002,
            CheckType::ExpectedVariableMissing => rule_ids::SD0057,
            CheckType::DataTypeMismatch => rule_ids::SD0055,
            CheckType::InvalidDateFormat => rule_ids::SD0003,
            CheckType::DuplicateSequence => rule_ids::SD0005,
            CheckType::TextLengthExceeded => rule_ids::SD0017,
            CheckType::IdentifierNull => rule_ids::SD0002, // Identifiers are Required
        }
    }

    /// Returns the P21 category for this check type.
    pub fn p21_category(&self) -> P21Category {
        match self {
            CheckType::ControlledTerminology => P21Category::Terminology,
            CheckType::RequiredVariableMissing => P21Category::Metadata,
            CheckType::RequiredVariableEmpty => P21Category::Presence,
            CheckType::ExpectedVariableMissing => P21Category::Metadata,
            CheckType::DataTypeMismatch => P21Category::Metadata,
            CheckType::InvalidDateFormat => P21Category::Format,
            CheckType::DuplicateSequence => P21Category::Consistency,
            CheckType::TextLengthExceeded => P21Category::Format,
            CheckType::IdentifierNull => P21Category::Presence,
        }
    }

    /// Create CheckType from a P21 rule ID.
    pub fn from_p21_rule_id(id: &str) -> Option<Self> {
        match id {
            rule_ids::CT2001 | rule_ids::CT2002 => Some(CheckType::ControlledTerminology),
            rule_ids::SD0056 => Some(CheckType::RequiredVariableMissing),
            rule_ids::SD0002 => Some(CheckType::RequiredVariableEmpty),
            rule_ids::SD0057 => Some(CheckType::ExpectedVariableMissing),
            rule_ids::SD0055 => Some(CheckType::DataTypeMismatch),
            rule_ids::SD0003 => Some(CheckType::InvalidDateFormat),
            rule_ids::SD0005 => Some(CheckType::DuplicateSequence),
            rule_ids::SD0017 => Some(CheckType::TextLengthExceeded),
            _ => None,
        }
    }
}

/// A validation issue found during validation.
///
/// Issues are now identified by P21 rule IDs rather than custom codes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Type of validation check that found this issue.
    #[serde(default)]
    pub check_type: Option<CheckType>,
    /// Pinnacle 21 rule ID (e.g., "CT2001", "SD0002").
    /// This is the official P21 rule code.
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

impl ValidationIssue {
    /// Get the P21 category for this issue.
    pub fn p21_category(&self) -> Option<P21Category> {
        self.check_type.map(|ct| ct.p21_category())
    }

    /// Check if this is a controlled terminology issue.
    pub fn is_ct_issue(&self) -> bool {
        self.code.starts_with("CT") || self.check_type == Some(CheckType::ControlledTerminology)
    }
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

    /// Group issues by P21 category.
    pub fn issues_by_category(&self) -> std::collections::HashMap<P21Category, Vec<&ValidationIssue>> {
        let mut map = std::collections::HashMap::new();
        for issue in &self.issues {
            if let Some(cat) = issue.p21_category() {
                map.entry(cat).or_insert_with(Vec::new).push(issue);
            }
        }
        map
    }
}
