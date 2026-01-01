//! Error types for the sdtm-xpt crate.
//!
//! This module provides two error types:
//! - [`XptIoError`]: I/O and format parsing errors  
//! - [`ValidationError`]: Data validation errors with location tracking
//!
//! The validation system uses a collect-all-errors pattern, allowing
//! all validation issues to be reported at once.

mod io;
mod validation;

pub use io::XptIoError;
pub use validation::{
    ErrorLocation, Severity, ValidationError, ValidationErrorCode, ValidationResult,
};

use thiserror::Error;

/// Result type for I/O operations that return [`XptIoError`].
pub type IoResult<T> = std::result::Result<T, XptIoError>;

/// Combined error type for XPT operations.
#[derive(Debug, Error)]
pub enum XptError {
    /// I/O or format error
    #[error("{0}")]
    Io(#[from] XptIoError),

    /// Validation errors (may contain multiple)
    #[error("{} validation error(s)", .0.len())]
    Validation(Vec<ValidationError>),
}

impl From<std::io::Error> for XptError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(XptIoError::Io(e))
    }
}

impl From<Vec<ValidationError>> for XptError {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self::Validation(errors)
    }
}

/// Result type for XPT operations.
pub type Result<T> = std::result::Result<T, XptError>;
