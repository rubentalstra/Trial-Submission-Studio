//! Error types for SDTM processing.
//!
//! Provides a unified error type for SDTM operations including I/O errors
//! and processing errors.

use thiserror::Error;

/// Error type for SDTM processing operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SdtmError {
    /// I/O error (file operations, network, etc.).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// General error with message.
    #[error("{0}")]
    Message(String),
}

/// Result type alias using [`SdtmError`].
pub type Result<T> = std::result::Result<T, SdtmError>;
