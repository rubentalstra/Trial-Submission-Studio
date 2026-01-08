//! Error types for the auto-update system.

use thiserror::Error;

/// Errors that can occur during the update process.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UpdateError {
    /// Failed to parse version string.
    #[error("invalid version format: {0}")]
    InvalidVersion(String),

    /// Network request failed.
    #[error("network error: {0}")]
    Network(String),

    /// Self-update crate error (covers download, checksum, install, etc.).
    #[error("update error: {0}")]
    SelfUpdate(String),

    /// I/O error during file operations (e.g., restart).
    #[error("I/O error: {0}")]
    Io(String),
}

/// Result type alias for update operations.
pub type Result<T> = std::result::Result<T, UpdateError>;
