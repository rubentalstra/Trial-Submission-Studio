//! Error types for XPT file operations.

use std::path::PathBuf;
use thiserror::Error;

use crate::types::XptVersion;

/// Errors that can occur when reading or writing XPT files.
#[derive(Debug, Error)]
pub enum XptError {
    /// File not found.
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Invalid XPT file format.
    #[error("invalid XPT file: {message}")]
    InvalidFormat { message: String },

    /// Missing required header record.
    #[error("missing header: expected {expected}")]
    MissingHeader { expected: &'static str },

    /// Invalid NAMESTR record.
    #[error("invalid NAMESTR at index {index}: {message}")]
    InvalidNamestr { index: usize, message: String },

    /// Float conversion error (IEEE/IBM).
    #[error("float conversion error: {message}")]
    FloatConversion { message: String },

    /// Invalid dataset name (empty).
    #[error("dataset name must not be empty")]
    InvalidDatasetName { name: String },

    /// Invalid variable name (empty).
    #[error("variable name must not be empty")]
    InvalidVariableName { name: String },

    /// Duplicate variable name.
    #[error("duplicate variable name: {name}")]
    DuplicateVariable { name: String },

    /// Row length mismatch.
    #[error("row length mismatch: expected {expected}, got {actual}")]
    RowLengthMismatch { expected: usize, actual: usize },

    /// Variable has zero length.
    #[error("variable {name} has zero length")]
    ZeroLength { name: String },

    /// Record out of bounds.
    #[error("record out of bounds at offset {offset}")]
    RecordOutOfBounds { offset: usize },

    /// Numeric field parsing error.
    #[error("failed to parse numeric field: {field}")]
    NumericParse { field: String },

    /// Observation data overflow.
    #[error("observation length overflow")]
    ObservationOverflow,

    /// Unexpected trailing bytes.
    #[error("unexpected trailing bytes in observations")]
    TrailingBytes,

    /// Dataset label exceeds 40 character limit.
    #[error("dataset label exceeds 40 character limit")]
    DatasetLabelTooLong { name: String },

    /// Dataset name exceeds version limit.
    #[error("dataset name '{name}' exceeds {version} limit of {limit} characters")]
    DatasetNameTooLong {
        name: String,
        version: XptVersion,
        limit: usize,
    },

    /// Variable name exceeds version limit.
    #[error("variable name '{name}' exceeds {version} limit of {limit} characters")]
    VariableNameTooLong {
        name: String,
        version: XptVersion,
        limit: usize,
    },

    /// Variable label exceeds version limit.
    #[error("variable label for '{name}' exceeds {version} limit of {limit} characters")]
    VariableLabelTooLong {
        name: String,
        version: XptVersion,
        limit: usize,
    },

    /// Format name exceeds version limit.
    #[error("format name '{format}' exceeds {version} limit of {limit} characters")]
    FormatNameTooLong {
        format: String,
        version: XptVersion,
        limit: usize,
    },

    /// Informat name exceeds version limit.
    #[error("informat name '{informat}' exceeds {version} limit of {limit} characters")]
    InformatNameTooLong {
        informat: String,
        version: XptVersion,
        limit: usize,
    },

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for XPT operations.
pub type Result<T> = std::result::Result<T, XptError>;

impl XptError {
    /// Create an InvalidFormat error.
    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self::InvalidFormat {
            message: message.into(),
        }
    }

    /// Create a MissingHeader error.
    pub fn missing_header(expected: &'static str) -> Self {
        Self::MissingHeader { expected }
    }

    /// Create an InvalidDatasetName error.
    pub fn invalid_dataset_name(name: impl Into<String>) -> Self {
        Self::InvalidDatasetName { name: name.into() }
    }

    /// Create an InvalidVariableName error.
    pub fn invalid_variable_name(name: impl Into<String>) -> Self {
        Self::InvalidVariableName { name: name.into() }
    }

    /// Create a DuplicateVariable error.
    pub fn duplicate_variable(name: impl Into<String>) -> Self {
        Self::DuplicateVariable { name: name.into() }
    }

    /// Create a ZeroLength error.
    pub fn zero_length(name: impl Into<String>) -> Self {
        Self::ZeroLength { name: name.into() }
    }

    /// Create a DatasetLabelTooLong error.
    pub fn dataset_label_too_long(name: impl Into<String>) -> Self {
        Self::DatasetLabelTooLong { name: name.into() }
    }

    /// Create a DatasetNameTooLong error.
    pub fn dataset_name_too_long(name: impl Into<String>, version: XptVersion) -> Self {
        Self::DatasetNameTooLong {
            name: name.into(),
            version,
            limit: version.dataset_name_limit(),
        }
    }

    /// Create a VariableNameTooLong error.
    pub fn variable_name_too_long(name: impl Into<String>, version: XptVersion) -> Self {
        Self::VariableNameTooLong {
            name: name.into(),
            version,
            limit: version.name_limit(),
        }
    }

    /// Create a VariableLabelTooLong error.
    pub fn variable_label_too_long(name: impl Into<String>, version: XptVersion) -> Self {
        Self::VariableLabelTooLong {
            name: name.into(),
            version,
            limit: version.label_limit(),
        }
    }

    /// Create a FormatNameTooLong error.
    pub fn format_name_too_long(format: impl Into<String>, version: XptVersion) -> Self {
        Self::FormatNameTooLong {
            format: format.into(),
            version,
            limit: version.format_limit(),
        }
    }

    /// Create an InformatNameTooLong error.
    pub fn informat_name_too_long(informat: impl Into<String>, version: XptVersion) -> Self {
        Self::InformatNameTooLong {
            informat: informat.into(),
            version,
            limit: version.format_limit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = XptError::invalid_format("test message");
        assert_eq!(format!("{err}"), "invalid XPT file: test message");

        let err = XptError::missing_header("LIBRARY");
        assert_eq!(format!("{err}"), "missing header: expected LIBRARY");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let xpt_err: XptError = io_err.into();
        assert!(matches!(xpt_err, XptError::Io(_)));
    }

    #[test]
    fn test_version_aware_errors() {
        let err = XptError::dataset_name_too_long("LONGNAME", XptVersion::V5);
        assert!(format!("{err}").contains("V5"));
        assert!(format!("{err}").contains("8 characters"));

        let err = XptError::variable_name_too_long("LONGNAME", XptVersion::V8);
        assert!(format!("{err}").contains("V8"));
        assert!(format!("{err}").contains("32 characters"));
    }
}
