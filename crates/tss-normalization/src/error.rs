//! Error types for the SDTM normalization system.

use thiserror::Error;

/// Errors that can occur during normalization.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum NormalizationError {
    /// Column not found in source DataFrame.
    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    /// Polars DataFrame operation error.
    #[error("DataFrame error: {0}")]
    PolarsError(#[from] polars::error::PolarsError),

    /// Parse error for a specific variable.
    #[error("Parse error for {variable}: {message}")]
    ParseError {
        /// Variable name where parse failed.
        variable: String,
        /// Description of the parse failure.
        message: String,
    },

    /// Missing required context (e.g., CT registry, reference date).
    #[error("Missing required context: {0}")]
    MissingContext(String),

    /// Invalid normalization configuration.
    #[error("Invalid normalization configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for normalization operations.
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, NormalizationError>;
