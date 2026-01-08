//! Validation error types for XPT data validation.
//!
//! This module provides comprehensive validation error tracking with:
//! - Location information (dataset, column, row level)
//! - Severity levels (Error vs Warning)
//! - Error codes for programmatic handling
//! - A result type that collects all errors

use std::fmt;

/// Severity level for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Severity {
    /// Must be fixed before writing - file would be invalid or rejected
    Error,
    /// FDA compliance warning - technically valid but may cause issues
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
        }
    }
}

/// Location of a validation error within the dataset.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorLocation {
    /// Error at the dataset level (e.g., dataset name)
    Dataset {
        /// Dataset name
        name: String,
    },
    /// Error at the column/variable level
    Column {
        /// Dataset name
        dataset: String,
        /// Column name
        column: String,
        /// Column index (0-based)
        index: usize,
    },
    /// Error at the observation/row level
    Observation {
        /// Dataset name
        dataset: String,
        /// Row number (0-based)
        row: usize,
    },
    /// Error at a specific cell value
    Value {
        /// Dataset name
        dataset: String,
        /// Column name
        column: String,
        /// Row number (0-based)
        row: usize,
    },
    /// Error related to a filename
    File {
        /// Filename
        filename: String,
    },
}

impl fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dataset { name } => write!(f, "dataset '{name}'"),
            Self::Column {
                dataset,
                column,
                index,
            } => {
                write!(
                    f,
                    "column '{column}' (index {index}) in dataset '{dataset}'"
                )
            }
            Self::Observation { dataset, row } => {
                write!(f, "row {row} in dataset '{dataset}'")
            }
            Self::Value {
                dataset,
                column,
                row,
            } => {
                write!(
                    f,
                    "value at column '{column}', row {row} in dataset '{dataset}'"
                )
            }
            Self::File { filename } => write!(f, "file '{filename}'"),
        }
    }
}

/// Error codes for programmatic handling of validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ValidationErrorCode {
    // ============ Name Validation ============
    /// Name is empty
    EmptyName,
    /// Name exceeds version limit (8 for V5, 32 for V8)
    NameTooLong,
    /// Name contains invalid characters (only A-Z, 0-9, _ allowed)
    InvalidNameCharacter,
    /// Name starts with a number (must start with letter or underscore)
    NameStartsWithNumber,
    /// Name contains non-ASCII characters
    NonAsciiName,
    /// Name is not uppercase (warning)
    LowercaseName,

    // ============ Label Validation ============
    /// Label exceeds version limit (40 for V5, 256 for V8)
    LabelTooLong,
    /// Label contains non-ASCII characters
    NonAsciiLabel,
    /// Label contains non-printable characters
    NonPrintableLabel,

    // ============ Format Validation ============
    /// Format name exceeds version limit
    FormatNameTooLong,
    /// Invalid format name pattern
    InvalidFormatName,
    /// Informat name exceeds version limit
    InformatNameTooLong,
    /// Invalid informat name pattern
    InvalidInformatName,
    /// Non-standard SAS format (warning)
    CustomFormat,

    // ============ Column/Variable Validation ============
    /// Duplicate column name in dataset
    DuplicateColumnName,
    /// Column length is zero
    ZeroLengthColumn,
    /// Column length exceeds maximum
    ColumnLengthTooLong,

    // ============ Dataset Validation ============
    /// Dataset has no columns
    EmptyDataset,
    /// Too many columns in dataset
    TooManyColumns,

    // ============ Observation/Row Validation ============
    /// Row has wrong number of values
    RowLengthMismatch,
    /// Character value exceeds column length
    CharacterValueTooLong,
    /// Non-ASCII character in character value
    NonAsciiValue,

    // ============ FDA-Specific ============
    /// FDA requires V5 format
    WrongVersion,
    /// Dataset name doesn't match filename
    DatasetNameMismatch,
    /// Multiple datasets in file (FDA requires single dataset)
    MultipleDatasets,
    /// Compressed data (not allowed by FDA)
    CompressedData,
}

impl fmt::Display for ValidationErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::EmptyName => "EMPTY_NAME",
            Self::NameTooLong => "NAME_TOO_LONG",
            Self::InvalidNameCharacter => "INVALID_NAME_CHAR",
            Self::NameStartsWithNumber => "NAME_STARTS_WITH_NUMBER",
            Self::NonAsciiName => "NON_ASCII_NAME",
            Self::LowercaseName => "LOWERCASE_NAME",
            Self::LabelTooLong => "LABEL_TOO_LONG",
            Self::NonAsciiLabel => "NON_ASCII_LABEL",
            Self::NonPrintableLabel => "NON_PRINTABLE_LABEL",
            Self::FormatNameTooLong => "FORMAT_NAME_TOO_LONG",
            Self::InvalidFormatName => "INVALID_FORMAT_NAME",
            Self::InformatNameTooLong => "INFORMAT_NAME_TOO_LONG",
            Self::InvalidInformatName => "INVALID_INFORMAT_NAME",
            Self::CustomFormat => "CUSTOM_FORMAT",
            Self::DuplicateColumnName => "DUPLICATE_COLUMN",
            Self::ZeroLengthColumn => "ZERO_LENGTH_COLUMN",
            Self::ColumnLengthTooLong => "COLUMN_LENGTH_TOO_LONG",
            Self::EmptyDataset => "EMPTY_DATASET",
            Self::TooManyColumns => "TOO_MANY_COLUMNS",
            Self::RowLengthMismatch => "ROW_LENGTH_MISMATCH",
            Self::CharacterValueTooLong => "CHAR_VALUE_TOO_LONG",
            Self::NonAsciiValue => "NON_ASCII_VALUE",
            Self::WrongVersion => "WRONG_VERSION",
            Self::DatasetNameMismatch => "DATASET_NAME_MISMATCH",
            Self::MultipleDatasets => "MULTIPLE_DATASETS",
            Self::CompressedData => "COMPRESSED_DATA",
        };
        write!(f, "{code}")
    }
}

/// A single validation error with location and context.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// Error code for programmatic handling
    pub code: ValidationErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Location where the error occurred
    pub location: ErrorLocation,
    /// Severity level (Error or Warning)
    pub severity: Severity,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} at {}: {}",
            self.code, self.severity, self.location, self.message
        )
    }
}

impl ValidationError {
    /// Create a new validation error.
    #[must_use]
    pub fn new(
        code: ValidationErrorCode,
        message: impl Into<String>,
        location: ErrorLocation,
        severity: Severity,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            location,
            severity,
        }
    }

    /// Create a new error (severity = Error).
    #[must_use]
    pub fn error(
        code: ValidationErrorCode,
        message: impl Into<String>,
        location: ErrorLocation,
    ) -> Self {
        Self::new(code, message, location, Severity::Error)
    }

    /// Create a new warning (severity = Warning).
    #[must_use]
    pub fn warning(
        code: ValidationErrorCode,
        message: impl Into<String>,
        location: ErrorLocation,
    ) -> Self {
        Self::new(code, message, location, Severity::Warning)
    }

    /// Check if this is an error (not a warning).
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Check if this is a warning.
    #[must_use]
    pub fn is_warning(&self) -> bool {
        self.severity == Severity::Warning
    }
}

/// Result of validation containing all errors and warnings.
///
/// This implements a collect-all-errors pattern, allowing all validation
/// issues to be reported at once rather than failing on the first error.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Validation errors (must be fixed)
    pub errors: Vec<ValidationError>,
    /// Validation warnings (FDA compliance issues)
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    /// Create a new empty validation result.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a successful validation result (no errors or warnings).
    #[must_use]
    pub fn success() -> Self {
        Self::default()
    }

    /// Add an error to the result.
    pub fn add_error(&mut self, error: ValidationError) {
        debug_assert!(error.is_error());
        self.errors.push(error);
    }

    /// Add a warning to the result.
    pub fn add_warning(&mut self, warning: ValidationError) {
        debug_assert!(warning.is_warning());
        self.warnings.push(warning);
    }

    /// Add a validation error (routes to errors or warnings based on severity).
    pub fn add(&mut self, error: ValidationError) {
        match error.severity {
            Severity::Error => self.errors.push(error),
            Severity::Warning => self.warnings.push(error),
        }
    }

    /// Merge another validation result into this one.
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }

    /// Check if validation passed (no errors, warnings allowed).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if validation passed with no issues at all.
    #[must_use]
    pub fn is_fda_compliant(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    /// Get total number of issues (errors + warnings).
    #[must_use]
    pub fn issue_count(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }

    /// Check if there are any issues.
    #[must_use]
    pub fn has_issues(&self) -> bool {
        self.issue_count() > 0
    }

    /// Convert to a Result, failing if there are any errors.
    pub fn into_result(self) -> std::result::Result<(), Vec<ValidationError>> {
        if self.is_valid() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    /// Get all issues (errors first, then warnings).
    pub fn all_issues(&self) -> impl Iterator<Item = &ValidationError> {
        self.errors.iter().chain(self.warnings.iter())
    }

    /// Get errors with a specific code.
    pub fn errors_with_code(
        &self,
        code: ValidationErrorCode,
    ) -> impl Iterator<Item = &ValidationError> {
        self.errors.iter().filter(move |e| e.code == code)
    }

    /// Get warnings with a specific code.
    pub fn warnings_with_code(
        &self,
        code: ValidationErrorCode,
    ) -> impl Iterator<Item = &ValidationError> {
        self.warnings.iter().filter(move |e| e.code == code)
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_fda_compliant() {
            write!(f, "Validation passed (FDA compliant)")
        } else if self.is_valid() {
            write!(
                f,
                "Validation passed with {} warning(s)",
                self.warnings.len()
            )
        } else {
            write!(
                f,
                "Validation failed: {} error(s), {} warning(s)",
                self.errors.len(),
                self.warnings.len()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_empty_is_valid() {
        let result = ValidationResult::new();
        assert!(result.is_valid());
        assert!(result.is_fda_compliant());
        assert_eq!(result.issue_count(), 0);
    }

    #[test]
    fn test_validation_result_with_warning_is_valid() {
        let mut result = ValidationResult::new();
        result.add_warning(ValidationError::warning(
            ValidationErrorCode::LowercaseName,
            "Name should be uppercase",
            ErrorLocation::Dataset { name: "dm".into() },
        ));

        assert!(result.is_valid());
        assert!(!result.is_fda_compliant());
        assert_eq!(result.issue_count(), 1);
    }

    #[test]
    fn test_validation_result_with_error_is_invalid() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError::error(
            ValidationErrorCode::EmptyName,
            "Name cannot be empty",
            ErrorLocation::Dataset {
                name: String::new(),
            },
        ));

        assert!(!result.is_valid());
        assert!(!result.is_fda_compliant());
        assert_eq!(result.issue_count(), 1);
    }

    #[test]
    fn test_validation_result_into_result() {
        let result = ValidationResult::new();
        assert!(result.into_result().is_ok());

        let mut result = ValidationResult::new();
        result.add_error(ValidationError::error(
            ValidationErrorCode::EmptyName,
            "test",
            ErrorLocation::Dataset {
                name: String::new(),
            },
        ));
        let err = result.into_result().unwrap_err();
        assert_eq!(err.len(), 1);
    }

    #[test]
    fn test_error_location_display() {
        let loc = ErrorLocation::Column {
            dataset: "DM".into(),
            column: "USUBJID".into(),
            index: 0,
        };
        assert_eq!(
            format!("{loc}"),
            "column 'USUBJID' (index 0) in dataset 'DM'"
        );
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::error(
            ValidationErrorCode::NameTooLong,
            "Name 'VERYLONGNAME' exceeds 8 character limit",
            ErrorLocation::Column {
                dataset: "DM".into(),
                column: "VERYLONGNAME".into(),
                index: 0,
            },
        );
        let display = format!("{err}");
        assert!(display.contains("NAME_TOO_LONG"));
        assert!(display.contains("error"));
        assert!(display.contains("VERYLONGNAME"));
    }
}
