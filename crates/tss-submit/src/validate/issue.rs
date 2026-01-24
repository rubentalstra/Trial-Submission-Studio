//! Validation issue types.
//!
//! The Issue enum provides type-safe validation issue creation where
//! each variant carries only its needed data.

use serde::{Deserialize, Serialize};

use super::rules::Category;

/// Issue severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Critical - submission will be rejected
    Reject,
    /// Must fix before submission
    Error,
    /// Should review
    Warning,
}

impl Severity {
    /// Parse severity from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "reject" => Some(Self::Reject),
            "error" => Some(Self::Error),
            "warning" => Some(Self::Warning),
            _ => None,
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Reject => "Reject",
            Self::Error => "Error",
            Self::Warning => "Warning",
        }
    }
}

/// Validation issue - each variant carries only its needed data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Issue {
    // Presence checks
    /// Required variable is missing from the dataset
    RequiredMissing { variable: String },
    /// Required variable exists but has null values
    RequiredEmpty { variable: String, null_count: u64 },
    /// Expected variable is missing from the dataset
    ExpectedMissing { variable: String },
    /// Identifier variable has null values
    IdentifierNull { variable: String, null_count: u64 },

    // Format checks
    /// Date values have invalid format
    InvalidDate {
        variable: String,
        invalid_count: u64,
        samples: Vec<String>,
    },
    /// Text values exceed maximum length
    TextTooLong {
        variable: String,
        exceeded_count: u64,
        max_found: usize,
        max_allowed: u32,
    },

    // Type checks
    /// Numeric variable contains non-numeric values
    DataTypeMismatch {
        variable: String,
        non_numeric_count: u64,
        samples: Vec<String>,
    },

    // Consistency checks
    /// Sequence variable has duplicate values within subjects
    DuplicateSequence {
        variable: String,
        duplicate_count: u64,
    },

    // Terminology checks
    /// Values not found in controlled terminology
    CtViolation {
        variable: String,
        codelist_code: String,
        codelist_name: String,
        extensible: bool,
        /// Total count of distinct invalid values (not truncated)
        total_invalid: u64,
        /// Sample of invalid values (up to 5)
        invalid_values: Vec<String>,
        allowed_count: usize,
    },

    // Cross-domain reference checks (#114)
    /// USUBJID values not found in DM domain
    UsubjidNotInDm {
        domain: String,
        missing_count: u64,
        samples: Vec<String>,
    },
    /// Parent record reference not found (e.g., --SPID references)
    ParentNotFound {
        variable: String,
        parent_domain: String,
        missing_count: u64,
        samples: Vec<String>,
    },
}

impl Issue {
    /// Variable name (all issues have one).
    pub fn variable(&self) -> &str {
        match self {
            Issue::RequiredMissing { variable } => variable,
            Issue::RequiredEmpty { variable, .. } => variable,
            Issue::ExpectedMissing { variable } => variable,
            Issue::IdentifierNull { variable, .. } => variable,
            Issue::InvalidDate { variable, .. } => variable,
            Issue::TextTooLong { variable, .. } => variable,
            Issue::DataTypeMismatch { variable, .. } => variable,
            Issue::DuplicateSequence { variable, .. } => variable,
            Issue::CtViolation { variable, .. } => variable,
            // Cross-domain issues use USUBJID or the specific variable
            Issue::UsubjidNotInDm { .. } => "USUBJID",
            Issue::ParentNotFound { variable, .. } => variable,
        }
    }

    /// Count of occurrences (if applicable).
    pub fn count(&self) -> Option<u64> {
        match self {
            Issue::RequiredMissing { .. } => None,
            Issue::ExpectedMissing { .. } => None,
            Issue::RequiredEmpty { null_count, .. } => Some(*null_count),
            Issue::IdentifierNull { null_count, .. } => Some(*null_count),
            Issue::InvalidDate { invalid_count, .. } => Some(*invalid_count),
            Issue::TextTooLong { exceeded_count, .. } => Some(*exceeded_count),
            Issue::DataTypeMismatch {
                non_numeric_count, ..
            } => Some(*non_numeric_count),
            Issue::DuplicateSequence {
                duplicate_count, ..
            } => Some(*duplicate_count),
            Issue::CtViolation { total_invalid, .. } => Some(*total_invalid),
            Issue::UsubjidNotInDm { missing_count, .. } => Some(*missing_count),
            Issue::ParentNotFound { missing_count, .. } => Some(*missing_count),
        }
    }

    /// Category for this issue type.
    pub fn category(&self) -> Category {
        match self {
            // Presence checks
            Issue::RequiredMissing { .. } => Category::Presence,
            Issue::RequiredEmpty { .. } => Category::Presence,
            Issue::ExpectedMissing { .. } => Category::Presence,
            Issue::IdentifierNull { .. } => Category::Presence,
            // Format checks
            Issue::InvalidDate { .. } => Category::Format,
            Issue::TextTooLong { .. } => Category::Limit,
            // Type checks
            Issue::DataTypeMismatch { .. } => Category::Format,
            // Consistency checks
            Issue::DuplicateSequence { .. } => Category::Consistency,
            // Terminology checks
            Issue::CtViolation { .. } => Category::Terminology,
            // Cross-domain reference checks
            Issue::UsubjidNotInDm { .. } => Category::CrossReference,
            Issue::ParentNotFound { .. } => Category::CrossReference,
        }
    }

    /// Severity for this issue type.
    pub fn severity(&self) -> Severity {
        match self {
            Issue::ExpectedMissing { .. } => Severity::Warning,
            Issue::TextTooLong { .. } => Severity::Warning,
            Issue::CtViolation {
                extensible: true, ..
            } => Severity::Warning,
            // Cross-domain reference issues are errors (data integrity)
            Issue::UsubjidNotInDm { .. } => Severity::Error,
            Issue::ParentNotFound { .. } => Severity::Error,
            _ => Severity::Error,
        }
    }

    /// Format message with issue-specific data.
    pub fn message(&self) -> String {
        match self {
            Issue::RequiredMissing { variable } => {
                format!("Required variable {} is missing", variable)
            }

            Issue::RequiredEmpty {
                variable,
                null_count,
            } => {
                format!(
                    "Required variable {} has {} null values",
                    variable, null_count
                )
            }

            Issue::ExpectedMissing { variable } => {
                format!("Expected variable {} is missing", variable)
            }

            Issue::IdentifierNull {
                variable,
                null_count,
            } => {
                format!(
                    "Identifier variable {} has {} null values",
                    variable, null_count
                )
            }

            Issue::InvalidDate {
                variable,
                invalid_count,
                samples,
            } => {
                let sample_str = if samples.is_empty() {
                    String::new()
                } else {
                    format!(" (e.g., {})", samples.join(", "))
                };
                format!(
                    "Variable {} has {} invalid date values{}",
                    variable, invalid_count, sample_str
                )
            }

            Issue::TextTooLong {
                variable,
                exceeded_count,
                max_found,
                max_allowed,
            } => {
                format!(
                    "Variable {} has {} values exceeding max length {} (found up to {})",
                    variable, exceeded_count, max_allowed, max_found
                )
            }

            Issue::DataTypeMismatch {
                variable,
                non_numeric_count,
                samples,
            } => {
                let sample_str = if samples.is_empty() {
                    String::new()
                } else {
                    format!(" (e.g., {})", samples.join(", "))
                };
                format!(
                    "Numeric variable {} has {} non-numeric values{}",
                    variable, non_numeric_count, sample_str
                )
            }

            Issue::DuplicateSequence {
                variable,
                duplicate_count,
            } => {
                format!(
                    "Sequence variable {} has {} duplicate values",
                    variable, duplicate_count
                )
            }

            Issue::CtViolation {
                variable,
                codelist_name,
                extensible,
                total_invalid,
                invalid_values,
                ..
            } => {
                let ext_str = if *extensible {
                    " (extensible)"
                } else {
                    " (non-extensible)"
                };
                let sample_count = invalid_values.len() as u64;
                let values_str = if invalid_values.is_empty() {
                    String::new()
                } else if sample_count < *total_invalid {
                    // Show truncation info when samples are limited
                    format!(
                        " (showing {} of {}): {}",
                        sample_count,
                        total_invalid,
                        invalid_values.join(", ")
                    )
                } else {
                    format!(": {}", invalid_values.join(", "))
                };
                format!(
                    "Variable {} has {} distinct invalid values not in codelist {}{}{}",
                    variable, total_invalid, codelist_name, ext_str, values_str
                )
            }

            Issue::UsubjidNotInDm {
                domain,
                missing_count,
                samples,
            } => {
                let sample_str = if samples.is_empty() {
                    String::new()
                } else {
                    format!(" (e.g., {})", samples.join(", "))
                };
                format!(
                    "Domain {} has {} USUBJID values not found in DM{}",
                    domain, missing_count, sample_str
                )
            }

            Issue::ParentNotFound {
                variable,
                parent_domain,
                missing_count,
                samples,
            } => {
                let sample_str = if samples.is_empty() {
                    String::new()
                } else {
                    format!(" (e.g., {})", samples.join(", "))
                };
                format!(
                    "Variable {} has {} references not found in {}{}",
                    variable, missing_count, parent_domain, sample_str
                )
            }
        }
    }
}
