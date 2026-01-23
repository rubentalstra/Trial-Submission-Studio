//! Unified error types for the tss-submit crate.
//!
//! This module provides a standardized error hierarchy using `thiserror` for all
//! submission pipeline operations: mapping, normalization, validation, and export.

use thiserror::Error;

/// Unified error type for all submission operations.
///
/// This enum consolidates errors from all stages of the submission pipeline,
/// providing consistent error handling and user-friendly messages.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum SubmitError {
    // =========================================================================
    // MAPPING ERRORS
    // =========================================================================
    /// Variable not found in domain definition.
    #[error("Variable not found: {variable}")]
    VariableNotFound {
        /// The variable name that was not found.
        variable: String,
    },

    /// Column not found in source data.
    #[error("Column not found: {column}")]
    ColumnNotFound {
        /// The column name that was not found.
        column: String,
    },

    /// Column already mapped to another variable.
    #[error("Column '{column}' already mapped to '{variable}'")]
    ColumnAlreadyMapped {
        /// The column that was already used.
        column: String,
        /// The variable it was mapped to.
        variable: String,
    },

    /// Cannot mark a Required variable as not collected.
    #[error("Cannot mark Required variable '{variable}' as not collected")]
    CannotSetNullOnRequired {
        /// The Required variable name.
        variable: String,
    },

    /// Cannot omit a non-Permissible variable.
    #[error("Cannot omit variable '{variable}': only Permissible variables can be omitted")]
    CannotOmitNonPermissible {
        /// The non-Permissible variable name.
        variable: String,
    },

    // =========================================================================
    // NORMALIZATION ERRORS
    // =========================================================================
    /// Parse error during normalization.
    #[error("Parse error for {variable}: {message}")]
    ParseError {
        /// Variable name where parse failed.
        variable: String,
        /// Description of the parse failure.
        message: String,
    },

    /// Missing required context for normalization.
    #[error("Missing required context: {context}")]
    MissingContext {
        /// Description of the missing context.
        context: String,
    },

    /// Invalid normalization configuration.
    #[error("Invalid normalization configuration: {message}")]
    InvalidConfig {
        /// Description of the invalid configuration.
        message: String,
    },

    // =========================================================================
    // EXPORT ERRORS
    // =========================================================================
    /// Failed to write output file.
    #[error("Failed to write {format} file '{path}': {message}")]
    WriteError {
        /// Export format (XPT, Dataset-XML, Define-XML).
        format: String,
        /// File path that failed to write.
        path: String,
        /// Detailed error message.
        message: String,
    },

    /// Missing domain definition during export.
    #[error("Missing domain definition for '{domain}'")]
    MissingDomain {
        /// Domain code that was not found.
        domain: String,
    },

    /// No datasets provided for export.
    #[error("No datasets provided for {format} export")]
    NoDatasets {
        /// Export format that requires datasets.
        format: String,
    },

    /// Missing codelist definition.
    #[error("Missing codelist '{codelist}' for {domain}.{variable}")]
    MissingCodelist {
        /// Codelist code that was not found.
        codelist: String,
        /// Domain name.
        domain: String,
        /// Variable name.
        variable: String,
    },

    /// XPT dataset validation failed.
    #[error("XPT validation failed for '{dataset}': {message}")]
    XptValidation {
        /// Dataset name.
        dataset: String,
        /// Validation error message.
        message: String,
    },

    // =========================================================================
    // WRAPPED ERRORS
    // =========================================================================
    /// Polars DataFrame operation error.
    #[error("DataFrame error: {0}")]
    Polars(#[from] polars::error::PolarsError),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// XML writing error.
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// Standards loading error.
    #[error("Standards error: {0}")]
    Standards(#[from] tss_standards::StandardsError),
}

/// Result type alias for submit operations.
pub type Result<T> = std::result::Result<T, SubmitError>;

impl SubmitError {
    /// Create a write error with format and path context.
    pub fn write_error(
        format: impl Into<String>,
        path: impl Into<String>,
        source: impl std::fmt::Display,
    ) -> Self {
        Self::WriteError {
            format: format.into(),
            path: path.into(),
            message: source.to_string(),
        }
    }

    /// Check if this error is recoverable (user can fix and retry).
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::VariableNotFound { .. }
                | Self::ColumnNotFound { .. }
                | Self::ColumnAlreadyMapped { .. }
                | Self::CannotSetNullOnRequired { .. }
                | Self::CannotOmitNonPermissible { .. }
                | Self::ParseError { .. }
                | Self::MissingContext { .. }
                | Self::InvalidConfig { .. }
        )
    }

    /// Get a user-friendly suggestion for fixing this error.
    pub fn suggestion(&self) -> Option<&'static str> {
        match self {
            Self::VariableNotFound { .. } => {
                Some("Check that the variable name matches the SDTM domain specification.")
            }
            Self::ColumnNotFound { .. } => {
                Some("Verify that the source file contains the expected column.")
            }
            Self::ColumnAlreadyMapped { .. } => {
                Some("Unmap the column from its current variable before remapping.")
            }
            Self::CannotSetNullOnRequired { .. } => {
                Some("Required variables must have a mapped column or assigned value.")
            }
            Self::CannotOmitNonPermissible { .. } => {
                Some("Only Permissible (Perm) variables can be omitted from the output.")
            }
            Self::ParseError { .. } => {
                Some("Check the source data format matches the expected variable type.")
            }
            Self::MissingContext { .. } => Some("Ensure all required reference data is loaded."),
            Self::WriteError { .. } => Some("Check file permissions and available disk space."),
            Self::MissingDomain { .. } => {
                Some("Ensure the domain definition is loaded from standards.")
            }
            _ => None,
        }
    }
}
