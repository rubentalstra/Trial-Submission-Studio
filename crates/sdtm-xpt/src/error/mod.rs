//! Error types for the sdtm-xpt crate.
//!
//! This module provides a unified error type [`XptError`] that covers:
//! - I/O and format parsing errors
//! - Validation errors with location tracking
//!
//! The validation system uses a collect-all-errors pattern, allowing
//! all validation issues to be reported at once.

mod validation;

use std::path::PathBuf;
use thiserror::Error;

pub use validation::{
    ErrorLocation, Severity, ValidationError, ValidationErrorCode, ValidationResult,
};

/// Unified error type for all XPT operations.
///
/// This enum covers I/O errors, format parsing errors, and validation errors.
#[derive(Debug, Error)]
pub enum XptError {
    // === I/O Errors ===
    
    /// File not found at the specified path.
    #[error("file not found: {}", path.display())]
    FileNotFound {
        /// Path that was not found
        path: PathBuf,
    },

    /// Underlying I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid XPT header format.
    #[error("invalid XPT header: expected {expected}")]
    InvalidHeader {
        /// Expected header type
        expected: &'static str,
    },

    /// Unexpected end of file.
    #[error("unexpected end of file at offset {offset}")]
    UnexpectedEof {
        /// Byte offset where EOF occurred
        offset: usize,
    },

    /// Invalid NAMESTR record.
    #[error("invalid NAMESTR record at index {index}: {message}")]
    InvalidNamestr {
        /// Index of the invalid record
        index: usize,
        /// Description of the error
        message: String,
    },

    /// Float conversion error.
    #[error("float conversion error: {message}")]
    FloatConversion {
        /// Description of the error
        message: String,
    },

    /// Record alignment error (XPT uses 80-byte records).
    #[error("record alignment error at offset {offset}")]
    AlignmentError {
        /// Byte offset where misalignment occurred
        offset: usize,
    },

    /// Invalid format in header or data.
    #[error("invalid format: {message}")]
    InvalidFormat {
        /// Description of the format error
        message: String,
    },

    /// Missing required header section.
    #[error("missing header section: {expected}")]
    MissingHeader {
        /// Name of the missing header
        expected: &'static str,
    },

    /// Numeric field parsing error.
    #[error("failed to parse numeric field: {field}")]
    NumericParse {
        /// Name of the field that failed to parse
        field: String,
    },

    /// Observation data overflow.
    #[error("observation data overflow: more data than expected")]
    ObservationOverflow,

    /// Trailing non-space bytes in observation data.
    #[error("trailing non-space bytes in observation data")]
    TrailingBytes,

    /// Record out of bounds.
    #[error("record access out of bounds at offset {offset}")]
    RecordOutOfBounds {
        /// Byte offset of out-of-bounds access
        offset: usize,
    },

    // === Data Validation Errors ===

    /// Invalid dataset name.
    #[error("invalid dataset name: '{name}'")]
    InvalidDatasetName {
        /// The invalid name
        name: String,
    },

    /// Dataset name too long.
    #[error("dataset name '{name}' exceeds {limit} character limit")]
    DatasetNameTooLong {
        /// The name that was too long
        name: String,
        /// Maximum allowed length
        limit: usize,
    },

    /// Dataset label too long.
    #[error("dataset label for '{dataset}' exceeds 40 character limit")]
    DatasetLabelTooLong {
        /// Dataset name
        dataset: String,
    },

    /// Invalid variable name.
    #[error("invalid variable name: '{name}'")]
    InvalidVariableName {
        /// The invalid name
        name: String,
    },

    /// Variable name too long.
    #[error("variable name '{name}' exceeds {limit} character limit")]
    VariableNameTooLong {
        /// The name that was too long
        name: String,
        /// Maximum allowed length
        limit: usize,
    },

    /// Duplicate variable name.
    #[error("duplicate variable name: '{name}'")]
    DuplicateVariable {
        /// The duplicate name
        name: String,
    },

    /// Zero length variable.
    #[error("variable '{name}' has zero length")]
    ZeroLength {
        /// Variable name
        name: String,
    },

    /// Variable label too long.
    #[error("variable label for '{name}' exceeds {limit} character limit")]
    VariableLabelTooLong {
        /// Variable name
        name: String,
        /// Maximum allowed length
        limit: usize,
    },

    /// Format name too long.
    #[error("format name '{format}' exceeds {limit} character limit")]
    FormatNameTooLong {
        /// Format name
        format: String,
        /// Maximum allowed length
        limit: usize,
    },

    /// Row length mismatch.
    #[error("row {row} has {actual} values but expected {expected}")]
    RowLengthMismatch {
        /// Row index
        row: usize,
        /// Expected number of values
        expected: usize,
        /// Actual number of values
        actual: usize,
    },

    // === Collected Validation Errors ===

    /// Multiple validation errors (collected).
    #[error("{} validation error(s)", .0.len())]
    Validation(Vec<ValidationError>),
}

impl XptError {
    /// Create an invalid format error.
    #[must_use]
    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self::InvalidFormat {
            message: message.into(),
        }
    }

    /// Create a missing header error.
    #[must_use]
    pub fn missing_header(expected: &'static str) -> Self {
        Self::MissingHeader { expected }
    }

    /// Create an invalid NAMESTR error.
    #[must_use]
    pub fn invalid_namestr(index: usize, message: impl Into<String>) -> Self {
        Self::InvalidNamestr {
            index,
            message: message.into(),
        }
    }

    /// Create a float conversion error.
    #[must_use]
    pub fn float_conversion(message: impl Into<String>) -> Self {
        Self::FloatConversion {
            message: message.into(),
        }
    }

    /// Create a file not found error.
    #[must_use]
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Create an invalid dataset name error.
    #[must_use]
    pub fn invalid_dataset_name(name: impl Into<String>) -> Self {
        Self::InvalidDatasetName { name: name.into() }
    }

    /// Create a dataset name too long error.
    #[must_use]
    pub fn dataset_name_too_long(name: impl Into<String>, limit: usize) -> Self {
        Self::DatasetNameTooLong {
            name: name.into(),
            limit,
        }
    }

    /// Create a dataset label too long error.
    #[must_use]
    pub fn dataset_label_too_long(dataset: impl Into<String>) -> Self {
        Self::DatasetLabelTooLong {
            dataset: dataset.into(),
        }
    }

    /// Create an invalid variable name error.
    #[must_use]
    pub fn invalid_variable_name(name: impl Into<String>) -> Self {
        Self::InvalidVariableName { name: name.into() }
    }

    /// Create a variable name too long error.
    #[must_use]
    pub fn variable_name_too_long(name: impl Into<String>, limit: usize) -> Self {
        Self::VariableNameTooLong {
            name: name.into(),
            limit,
        }
    }

    /// Create a duplicate variable error.
    #[must_use]
    pub fn duplicate_variable(name: impl Into<String>) -> Self {
        Self::DuplicateVariable { name: name.into() }
    }

    /// Create a zero length error.
    #[must_use]
    pub fn zero_length(name: impl Into<String>) -> Self {
        Self::ZeroLength { name: name.into() }
    }

    /// Create a variable label too long error.
    #[must_use]
    pub fn variable_label_too_long(name: impl Into<String>, limit: usize) -> Self {
        Self::VariableLabelTooLong {
            name: name.into(),
            limit,
        }
    }

    /// Create a format name too long error.
    #[must_use]
    pub fn format_name_too_long(format: impl Into<String>, limit: usize) -> Self {
        Self::FormatNameTooLong {
            format: format.into(),
            limit,
        }
    }

    /// Create a row length mismatch error.
    #[must_use]
    pub fn row_length_mismatch(row: usize, expected: usize, actual: usize) -> Self {
        Self::RowLengthMismatch {
            row,
            expected,
            actual,
        }
    }
}

impl From<Vec<ValidationError>> for XptError {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self::Validation(errors)
    }
}

/// Result type for XPT operations.
pub type Result<T> = std::result::Result<T, XptError>;
