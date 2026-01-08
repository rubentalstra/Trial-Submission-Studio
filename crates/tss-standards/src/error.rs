//! Error types for standards loading operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when loading SDTM standards.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StandardsError {
    /// Standards directory not found.
    #[error("Standards directory not found: {path}")]
    DirectoryNotFound { path: PathBuf },

    /// CSV file not found.
    #[error("CSV file not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Failed to read or parse CSV file.
    #[error("Failed to read CSV {path}: {source}")]
    CsvRead {
        path: PathBuf,
        #[source]
        source: csv::Error,
    },

    /// Invalid value in CSV field.
    #[error("Invalid {field} value '{value}' in {file}")]
    InvalidValue {
        field: &'static str,
        value: String,
        file: PathBuf,
    },
}

/// Result type for standards loading operations.
pub type Result<T> = std::result::Result<T, StandardsError>;
