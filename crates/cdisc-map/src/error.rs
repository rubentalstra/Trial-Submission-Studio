//! Error types for mapping operations.

use std::fmt;

/// Errors from mapping operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappingError {
    /// Variable not found in domain.
    VariableNotFound(String),
    /// Column not found in source data.
    ColumnNotFound(String),
    /// Column already mapped to another variable.
    ColumnAlreadyUsed { column: String, variable: String },
    /// Cannot mark Required variable as not collected.
    CannotSetNullOnRequired(String),
    /// Cannot omit non-Permissible variable (only Permissible vars can be omitted).
    CannotOmitNonPermissible(String),
}

impl fmt::Display for MappingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VariableNotFound(v) => write!(f, "Variable not found: {v}"),
            Self::ColumnNotFound(c) => write!(f, "Column not found: {c}"),
            Self::ColumnAlreadyUsed { column, variable } => {
                write!(f, "Column '{column}' already mapped to '{variable}'")
            }
            Self::CannotSetNullOnRequired(v) => {
                write!(f, "Cannot mark Required variable '{v}' as not collected")
            }
            Self::CannotOmitNonPermissible(v) => {
                write!(
                    f,
                    "Cannot omit variable '{v}': only Permissible variables can be omitted"
                )
            }
        }
    }
}

impl std::error::Error for MappingError {}
