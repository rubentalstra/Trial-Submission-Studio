//! Error types for SDTM data ingestion.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during data ingestion operations.
#[derive(Debug, Error)]
pub enum IngestError {
    // === File System Errors ===
    /// Directory not found or not readable.
    #[error("directory not found: {path}")]
    DirectoryNotFound { path: PathBuf },

    /// CSV file not found.
    #[error("CSV file not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Failed to read directory entries.
    #[error("failed to read directory {path}: {source}")]
    DirectoryRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to read file.
    #[error("failed to read file {path}: {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    // === CSV Parsing Errors ===
    /// Failed to parse CSV with Polars.
    #[error("failed to parse CSV {path}: {message}")]
    CsvParse { path: PathBuf, message: String },

    /// CSV file is empty or has no valid rows.
    #[error("CSV file is empty: {path}")]
    EmptyCsv { path: PathBuf },

    /// Failed to detect valid header row.
    #[error("could not detect header row in {path}")]
    NoHeaderDetected { path: PathBuf },

    // === Metadata Errors ===
    /// Required column not found in metadata file.
    #[error("required column '{column}' not found in {path}")]
    MissingColumn { column: String, path: PathBuf },

    /// Invalid value in metadata field.
    #[error("invalid {field} value '{value}' in {path}")]
    InvalidValue {
        field: String,
        value: String,
        path: PathBuf,
    },

    /// Metadata file has unexpected format.
    #[error("unexpected metadata format in {path}: {reason}")]
    MetadataFormat { path: PathBuf, reason: String },

    // === DataFrame Errors ===
    /// Column not found in DataFrame.
    #[error("column '{column}' not found in DataFrame")]
    ColumnNotFound { column: String },

    /// Failed DataFrame operation.
    #[error("DataFrame operation failed: {message}")]
    DataFrame { message: String },

    // === Schema Detection Errors ===
    /// Failed to detect schema for metadata file.
    #[error("could not detect schema for {file_type} file {path}: {reason}")]
    SchemaDetection {
        file_type: String,
        path: PathBuf,
        reason: String,
    },
}

impl From<polars::prelude::PolarsError> for IngestError {
    fn from(err: polars::prelude::PolarsError) -> Self {
        Self::DataFrame {
            message: err.to_string(),
        }
    }
}

/// Result type for ingestion operations.
pub type Result<T> = std::result::Result<T, IngestError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = IngestError::FileNotFound {
            path: PathBuf::from("/path/to/file.csv"),
        };
        assert_eq!(err.to_string(), "CSV file not found: /path/to/file.csv");
    }

    #[test]
    fn test_error_from_polars() {
        let polars_err = polars::prelude::PolarsError::ColumnNotFound("test".into());
        let ingest_err: IngestError = polars_err.into();
        assert!(matches!(ingest_err, IngestError::DataFrame { .. }));
    }
}
