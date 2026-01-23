//! Persistence error types.
//!
//! All persistence operations return structured errors that provide
//! user-friendly messages and optional remediation hints.

use std::path::PathBuf;
use thiserror::Error;

/// Persistence operation error.
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// File I/O error.
    #[error("Failed to {operation} file: {path}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Invalid file format (not a .tss file).
    #[error("Invalid project file format")]
    InvalidFormat { path: PathBuf, reason: String },

    /// Unsupported schema version.
    #[error("Project file version {found} is not supported (maximum: {max_supported})")]
    UnsupportedVersion {
        found: u32,
        max_supported: u32,
        path: PathBuf,
    },

    /// Source CSV file has been modified since project was saved.
    #[error("Source file has been modified: {path}")]
    SourceFileChanged {
        path: PathBuf,
        expected_hash: String,
        actual_hash: String,
    },

    /// Source CSV file is missing.
    #[error("Source file not found: {path}")]
    SourceFileMissing { path: PathBuf },

    /// Serialization error.
    #[error("Failed to serialize project data")]
    Serialization {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Deserialization error.
    #[error("Failed to deserialize project data")]
    Deserialization {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Atomic write failed (temp file couldn't be renamed).
    #[error("Failed to complete save operation")]
    AtomicWriteFailed {
        temp_path: PathBuf,
        target_path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl PersistenceError {
    /// Get a user-friendly message for this error.
    pub fn user_message(&self) -> String {
        match self {
            Self::Io {
                operation, path, ..
            } => {
                format!("Could not {} the file at {}", operation, path.display())
            }
            Self::InvalidFormat { path, reason } => {
                format!(
                    "The file at {} is not a valid project file: {}",
                    path.display(),
                    reason
                )
            }
            Self::UnsupportedVersion {
                found,
                max_supported,
                ..
            } => {
                format!(
                    "This project file was created with a newer version of Trial Submission Studio \
                    (file version {}, your version supports up to {}). \
                    Please update the application.",
                    found, max_supported
                )
            }
            Self::SourceFileChanged { path, .. } => {
                format!(
                    "The source file '{}' has been modified since this project was last saved.",
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                )
            }
            Self::SourceFileMissing { path } => {
                format!(
                    "The source file '{}' could not be found. It may have been moved or deleted.",
                    path.display()
                )
            }
            Self::Serialization { .. } => {
                "An error occurred while saving the project data.".to_string()
            }
            Self::Deserialization { .. } => {
                "An error occurred while reading the project data. The file may be corrupted."
                    .to_string()
            }
            Self::AtomicWriteFailed { target_path, .. } => {
                format!(
                    "Could not save the file to {}. Please check disk space and permissions.",
                    target_path.display()
                )
            }
        }
    }

    /// Get a suggestion for how to resolve this error.
    pub fn suggestion(&self) -> Option<String> {
        match self {
            Self::Io { operation, .. } => {
                if *operation == "read" {
                    Some("Check that the file exists and you have permission to read it.".into())
                } else {
                    Some("Check that you have permission to write to this location.".into())
                }
            }
            Self::InvalidFormat { .. } => {
                Some("Make sure you selected a .tss project file.".into())
            }
            Self::UnsupportedVersion { .. } => {
                Some("Download the latest version from the Trial Submission Studio website.".into())
            }
            Self::SourceFileChanged { .. } => {
                Some(
                    "You can choose to reload mappings from the source files or continue with the saved mappings."
                        .into(),
                )
            }
            Self::SourceFileMissing { .. } => {
                Some("Locate the original source file or remove it from the project.".into())
            }
            Self::Serialization { .. } => None,
            Self::Deserialization { .. } => {
                Some("Try opening a backup if you have one.".into())
            }
            Self::AtomicWriteFailed { .. } => {
                Some("Free up disk space or try saving to a different location.".into())
            }
        }
    }
}

/// Result type alias for persistence operations.
pub type Result<T> = std::result::Result<T, PersistenceError>;
