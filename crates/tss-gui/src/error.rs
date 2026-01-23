//! GUI-specific error types.
//!
//! This module provides a unified error type for GUI operations, designed to
//! integrate with Iced's message-based architecture while providing user-friendly
//! error messages and suggestions.

use thiserror::Error;

/// GUI-specific errors.
///
/// These errors are designed to be displayed to users and include actionable
/// information about how to resolve them.
///
/// # Display Behavior
///
/// Errors can be categorized for different display treatments:
/// - **Transient**: Brief errors shown as toasts (auto-dismiss)
/// - **Modal**: Important errors requiring user acknowledgment
/// - **Blocking**: Critical errors that prevent further operation
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GuiError {
    // =========================================================================
    // STUDY OPERATIONS
    // =========================================================================
    /// Failed to load study from disk.
    #[error("Failed to load study: {reason}")]
    StudyLoad {
        /// Description of what went wrong.
        reason: String,
    },

    /// Failed to save study metadata.
    #[error("Failed to save study: {reason}")]
    StudySave {
        /// Description of what went wrong.
        reason: String,
    },

    /// Study folder not found or inaccessible.
    #[error("Study folder not found: {path}")]
    StudyNotFound {
        /// Path that was not found.
        path: String,
    },

    /// No study is currently loaded.
    #[error("No study is currently open")]
    NoStudyLoaded,

    // =========================================================================
    // DOMAIN OPERATIONS
    // =========================================================================
    /// Domain not found in study.
    #[error("Domain not found: {domain}")]
    DomainNotFound {
        /// Domain code that was not found.
        domain: String,
    },

    /// Failed to load domain data.
    #[error("Failed to load domain '{domain}': {reason}")]
    DomainLoad {
        /// Domain code.
        domain: String,
        /// Description of what went wrong.
        reason: String,
    },

    // =========================================================================
    // MAPPING OPERATIONS
    // =========================================================================
    /// Mapping operation failed.
    #[error("Mapping error: {message}")]
    Mapping {
        /// Description of the mapping error.
        message: String,
    },

    // =========================================================================
    // VALIDATION
    // =========================================================================
    /// Validation failed with critical issues.
    #[error("Validation failed for {domain}: {issue_count} issues found")]
    ValidationFailed {
        /// Domain that failed validation.
        domain: String,
        /// Number of issues found.
        issue_count: usize,
    },

    // =========================================================================
    // EXPORT OPERATIONS
    // =========================================================================
    /// Export operation failed.
    #[error("Export failed: {reason}")]
    Export {
        /// Domain being exported (if applicable).
        domain: Option<String>,
        /// Description of what went wrong.
        reason: String,
    },

    /// Export directory could not be created or accessed.
    #[error("Cannot access export directory: {path}")]
    ExportDirectory {
        /// Path that was inaccessible.
        path: String,
    },

    // =========================================================================
    // SETTINGS
    // =========================================================================
    /// Failed to load settings.
    #[error("Failed to load settings: {reason}")]
    SettingsLoad {
        /// Description of what went wrong.
        reason: String,
    },

    /// Failed to save settings.
    #[error("Failed to save settings: {reason}")]
    SettingsSave {
        /// Description of what went wrong.
        reason: String,
    },

    // =========================================================================
    // GENERAL OPERATIONS
    // =========================================================================
    /// File operation failed.
    #[error("File operation failed: {reason}")]
    FileOperation {
        /// Description of what went wrong.
        reason: String,
    },

    /// Generic operation error with context.
    #[error("{operation} failed: {reason}")]
    Operation {
        /// Name of the operation that failed.
        operation: String,
        /// Description of what went wrong.
        reason: String,
    },

    /// Internal error (should not normally occur).
    #[error("Internal error: {message}")]
    Internal {
        /// Description of the internal error.
        message: String,
    },
}

impl GuiError {
    /// Check if this error should be shown as a transient toast notification.
    ///
    /// Transient errors are minor issues that don't require user acknowledgment.
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::Mapping { .. } | Self::ValidationFailed { .. })
    }

    /// Check if this error is critical and blocks further operation.
    pub fn is_blocking(&self) -> bool {
        matches!(
            self,
            Self::StudyNotFound { .. } | Self::NoStudyLoaded | Self::Internal { .. }
        )
    }

    /// Get a user-friendly suggestion for resolving this error.
    pub fn suggestion(&self) -> Option<&'static str> {
        match self {
            Self::StudyLoad { .. } => {
                Some("Ensure the study folder exists and contains valid data files.")
            }
            Self::StudySave { .. } => Some("Check file permissions and available disk space."),
            Self::StudyNotFound { .. } => Some("The study folder may have been moved or deleted."),
            Self::NoStudyLoaded => Some("Open a study folder to begin working."),
            Self::DomainNotFound { .. } => {
                Some("Check that the domain is supported in the current study configuration.")
            }
            Self::DomainLoad { .. } => {
                Some("Verify the source data file exists and is formatted correctly.")
            }
            Self::Mapping { .. } => Some("Review the mapping configuration and try again."),
            Self::ValidationFailed { .. } => {
                Some("Review the validation issues and correct the source data.")
            }
            Self::Export { .. } => {
                Some("Check file permissions, disk space, and that all required data is mapped.")
            }
            Self::ExportDirectory { .. } => {
                Some("Check that you have write permissions to the output directory.")
            }
            Self::SettingsLoad { .. } => {
                Some("Settings will be reset to defaults if the file is corrupted.")
            }
            Self::SettingsSave { .. } => {
                Some("Check file permissions for the application config directory.")
            }
            Self::FileOperation { .. } => {
                Some("Check file permissions and that the file is not in use.")
            }
            Self::Operation { .. } | Self::Internal { .. } => None,
        }
    }

    /// Get the error category for display purposes.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::StudyLoad { .. }
            | Self::StudySave { .. }
            | Self::StudyNotFound { .. }
            | Self::NoStudyLoaded => ErrorCategory::Study,

            Self::DomainNotFound { .. } | Self::DomainLoad { .. } => ErrorCategory::Domain,

            Self::Mapping { .. } => ErrorCategory::Mapping,

            Self::ValidationFailed { .. } => ErrorCategory::Validation,

            Self::Export { .. } | Self::ExportDirectory { .. } => ErrorCategory::Export,

            Self::SettingsLoad { .. } | Self::SettingsSave { .. } => ErrorCategory::Settings,

            Self::FileOperation { .. } | Self::Operation { .. } | Self::Internal { .. } => {
                ErrorCategory::General
            }
        }
    }

    // =========================================================================
    // FACTORY METHODS
    // =========================================================================

    /// Create a study load error from any error source.
    pub fn study_load(err: impl std::fmt::Display) -> Self {
        Self::StudyLoad {
            reason: err.to_string(),
        }
    }

    /// Create a domain load error.
    pub fn domain_load(domain: impl Into<String>, err: impl std::fmt::Display) -> Self {
        Self::DomainLoad {
            domain: domain.into(),
            reason: err.to_string(),
        }
    }

    /// Create an export error.
    pub fn export(domain: Option<String>, err: impl std::fmt::Display) -> Self {
        Self::Export {
            domain,
            reason: err.to_string(),
        }
    }

    /// Create a mapping error.
    pub fn mapping(err: impl std::fmt::Display) -> Self {
        Self::Mapping {
            message: err.to_string(),
        }
    }

    /// Create a general operation error.
    pub fn operation(operation: impl Into<String>, err: impl std::fmt::Display) -> Self {
        Self::Operation {
            operation: operation.into(),
            reason: err.to_string(),
        }
    }

    /// Create an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

/// Error category for grouping related errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Study-related errors (load, save, not found).
    Study,
    /// Domain-related errors (load, not found).
    Domain,
    /// Mapping operation errors.
    Mapping,
    /// Validation errors.
    Validation,
    /// Export operation errors.
    Export,
    /// Settings errors.
    Settings,
    /// General/uncategorized errors.
    General,
}

impl ErrorCategory {
    /// Get a human-readable label for this category.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Study => "Study",
            Self::Domain => "Domain",
            Self::Mapping => "Mapping",
            Self::Validation => "Validation",
            Self::Export => "Export",
            Self::Settings => "Settings",
            Self::General => "Error",
        }
    }
}

/// Convert from tss-submit errors.
impl From<tss_submit::SubmitError> for GuiError {
    fn from(err: tss_submit::SubmitError) -> Self {
        // Determine the appropriate GUI error based on the submit error type
        match &err {
            tss_submit::SubmitError::VariableNotFound { .. }
            | tss_submit::SubmitError::ColumnNotFound { .. }
            | tss_submit::SubmitError::ColumnAlreadyMapped { .. }
            | tss_submit::SubmitError::CannotSetNullOnRequired { .. }
            | tss_submit::SubmitError::CannotOmitNonPermissible { .. } => Self::Mapping {
                message: err.to_string(),
            },
            tss_submit::SubmitError::WriteError { .. }
            | tss_submit::SubmitError::MissingDomain { .. }
            | tss_submit::SubmitError::NoDatasets { .. }
            | tss_submit::SubmitError::XptValidation { .. } => Self::Export {
                domain: None,
                reason: err.to_string(),
            },
            _ => Self::Operation {
                operation: "Submit".to_string(),
                reason: err.to_string(),
            },
        }
    }
}

/// Convert from mapping errors.
impl From<tss_submit::MappingError> for GuiError {
    fn from(err: tss_submit::MappingError) -> Self {
        Self::Mapping {
            message: err.to_string(),
        }
    }
}
