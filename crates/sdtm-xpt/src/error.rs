//! Error types for XPT file operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when reading or writing XPT files.
#[derive(Debug, Error)]
pub enum XptError {
    /// File not found at specified path.
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Invalid XPT file format.
    #[error("invalid XPT file: {message}")]
    InvalidFormat { message: String },

    /// Missing required header record.
    #[error("missing header: expected {expected}")]
    MissingHeader { expected: &'static str },

    /// Invalid NAMESTR record at given index.
    #[error("invalid NAMESTR at index {index}: {message}")]
    InvalidNamestr { index: usize, message: String },

    /// Float conversion error (IEEE/IBM).
    #[error("float conversion error: {message}")]
    FloatConversion { message: String },

    /// Dataset name validation error.
    #[error("dataset name must be 1-8 characters: {name}")]
    InvalidDatasetName { name: String },

    /// Variable name validation error.
    #[error("variable name must be 1-8 characters: {name}")]
    InvalidVariableName { name: String },

    /// Duplicate variable name in dataset.
    #[error("duplicate variable name: {name}")]
    DuplicateVariable { name: String },

    /// Row value count doesn't match column count.
    #[error("row length mismatch: expected {expected}, got {actual}")]
    RowLengthMismatch { expected: usize, actual: usize },

    /// Variable length is zero.
    #[error("variable {name} has zero length")]
    ZeroLength { name: String },

    /// Label exceeds maximum length.
    #[error("label exceeds maximum length of {max} characters")]
    LabelTooLong { max: usize },

    /// Record out of bounds when reading.
    #[error("record out of bounds at offset {offset}")]
    RecordOutOfBounds { offset: usize },

    /// Numeric field parsing error.
    #[error("failed to parse numeric field: {field}")]
    NumericParse { field: String },

    /// Observation data overflow.
    #[error("observation length overflow")]
    ObservationOverflow,

    /// Unexpected trailing bytes in file.
    #[error("unexpected trailing bytes in observations")]
    TrailingBytes,

    /// I/O error from underlying operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for XPT operations.
pub type Result<T> = std::result::Result<T, XptError>;

impl XptError {
    /// Create an InvalidFormat error with a message.
    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self::InvalidFormat {
            message: message.into(),
        }
    }

    /// Create a MissingHeader error.
    pub fn missing_header(expected: &'static str) -> Self {
        Self::MissingHeader { expected }
    }

    /// Create an InvalidNamestr error.
    pub fn invalid_namestr(index: usize, message: impl Into<String>) -> Self {
        Self::InvalidNamestr {
            index,
            message: message.into(),
        }
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

        let err = XptError::InvalidNamestr {
            index: 5,
            message: "bad type".to_string(),
        };
        assert_eq!(format!("{err}"), "invalid NAMESTR at index 5: bad type");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let xpt_err: XptError = io_err.into();
        assert!(matches!(xpt_err, XptError::Io(_)));
    }
}
