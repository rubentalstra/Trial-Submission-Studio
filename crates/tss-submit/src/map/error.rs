//! Error types for mapping operations.

use thiserror::Error;

/// Errors from mapping operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MappingError {
    /// Variable not found in domain.
    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    /// Column not found in source data.
    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    /// Column already mapped to another variable.
    #[error("Column '{column}' already mapped to '{variable}'")]
    ColumnAlreadyUsed {
        /// The column that was already used.
        column: String,
        /// The variable it was mapped to.
        variable: String,
    },

    /// Cannot mark Required variable as not collected.
    #[error("Cannot mark Required variable '{0}' as not collected")]
    CannotSetNullOnRequired(String),

    /// Cannot omit non-Permissible variable (only Permissible vars can be omitted).
    #[error("Cannot omit variable '{0}': only Permissible variables can be omitted")]
    CannotOmitNonPermissible(String),
}

impl MappingError {
    /// Get the variable name associated with this error, if any.
    pub fn variable(&self) -> Option<&str> {
        match self {
            Self::VariableNotFound(v) => Some(v),
            Self::ColumnAlreadyUsed { variable, .. } => Some(variable),
            Self::CannotSetNullOnRequired(v) => Some(v),
            Self::CannotOmitNonPermissible(v) => Some(v),
            Self::ColumnNotFound(_) => None,
        }
    }

    /// Get the column name associated with this error, if any.
    pub fn column(&self) -> Option<&str> {
        match self {
            Self::ColumnNotFound(c) => Some(c),
            Self::ColumnAlreadyUsed { column, .. } => Some(column),
            _ => None,
        }
    }
}

/// Convert MappingError to the unified SubmitError.
impl From<MappingError> for crate::error::SubmitError {
    fn from(err: MappingError) -> Self {
        match err {
            MappingError::VariableNotFound(variable) => {
                crate::error::SubmitError::VariableNotFound { variable }
            }
            MappingError::ColumnNotFound(column) => {
                crate::error::SubmitError::ColumnNotFound { column }
            }
            MappingError::ColumnAlreadyUsed { column, variable } => {
                crate::error::SubmitError::ColumnAlreadyMapped { column, variable }
            }
            MappingError::CannotSetNullOnRequired(variable) => {
                crate::error::SubmitError::CannotSetNullOnRequired { variable }
            }
            MappingError::CannotOmitNonPermissible(variable) => {
                crate::error::SubmitError::CannotOmitNonPermissible { variable }
            }
        }
    }
}
