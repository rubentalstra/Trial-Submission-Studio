//! XPT writer builder with validation.
//!
//! This module provides a builder pattern for creating XPT writers
//! with explicit validation steps before writing.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::error::{Result, ValidationError, ValidationResult, XptError};
use crate::header::normalize_name;
use crate::types::{XptDataset, XptVersion, XptWriterOptions};
use crate::validation::{ValidationMode, Validator};

/// Builder for creating XPT writers with validation.
///
/// This builder provides a two-step approach:
/// 1. Configure options and validate the dataset
/// 2. Write to the output destination
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use tss_xpt::{XptDataset, XptColumn, XptWriterBuilder};
///
/// let dataset = XptDataset::with_columns("DM", vec![
///     XptColumn::character("USUBJID", 20),
///     XptColumn::numeric("AGE"),
/// ]);
///
/// let result = XptWriterBuilder::new()
///     .fda_compliant()
///     .validate(&dataset);
///
/// if result.is_valid() {
///     result.write_to_file(Path::new("dm.xpt"), &dataset).unwrap();
/// } else {
///     for error in result.errors() {
///         eprintln!("Error: {}", error);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct XptWriterBuilder {
    options: XptWriterOptions,
    mode: ValidationMode,
}

impl Default for XptWriterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl XptWriterBuilder {
    /// Create a new writer builder with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: XptWriterOptions::default(),
            mode: ValidationMode::Basic,
        }
    }

    /// Set the XPT version.
    #[must_use]
    pub fn version(mut self, version: XptVersion) -> Self {
        self.options = self.options.with_version(version);
        self
    }

    /// Enable FDA-compliant mode.
    ///
    /// This sets the version to V5 and enables FDA validation rules.
    #[must_use]
    pub fn fda_compliant(mut self) -> Self {
        self.options = self.options.with_version(XptVersion::V5);
        self.mode = ValidationMode::FdaCompliant;
        self
    }

    /// Use V5 format (default).
    #[must_use]
    pub fn v5(mut self) -> Self {
        self.options = self.options.with_version(XptVersion::V5);
        self
    }

    /// Use V8 format (longer names, labels, formats).
    #[must_use]
    pub fn v8(mut self) -> Self {
        self.options = self.options.with_version(XptVersion::V8);
        self
    }

    /// Set writer options.
    #[must_use]
    pub fn with_options(mut self, options: XptWriterOptions) -> Self {
        self.options = options;
        self
    }

    /// Validate the dataset and return a validated writer.
    ///
    /// This collects all validation errors/warnings before writing.
    #[must_use]
    pub fn validate(self, dataset: &XptDataset) -> ValidatedWriter {
        let version = self.options.version;

        // Use the validation framework
        let validator = Validator::new(version).with_mode(self.mode);
        let result = validator.validate(dataset);

        ValidatedWriter {
            options: self.options,
            result,
        }
    }

    /// Validate and write in one step, returning on first error.
    ///
    /// Use this for simple cases where you don't need to inspect all errors.
    pub fn write(self, path: &Path, dataset: &XptDataset) -> crate::Result<()> {
        let validated = self.validate(dataset);

        if !validated.is_valid() {
            // Return the first error
            if let Some(error) = validated.result.errors.first() {
                return Err(XptError::invalid_format(error.to_string()));
            }
        }

        validated.write_to_file(path, dataset)
    }
}

/// A writer that has been validated and is ready to write.
#[derive(Debug)]
pub struct ValidatedWriter {
    options: XptWriterOptions,
    result: ValidationResult,
}

impl ValidatedWriter {
    /// Check if validation passed (no errors, only warnings allowed).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.result.is_valid()
    }

    /// Get all validation errors.
    #[must_use]
    pub fn errors(&self) -> &[ValidationError] {
        &self.result.errors
    }

    /// Get all validation warnings.
    #[must_use]
    pub fn warnings(&self) -> &[ValidationError] {
        &self.result.warnings
    }

    /// Get the full validation result.
    #[must_use]
    pub fn validation_result(&self) -> &ValidationResult {
        &self.result
    }

    /// Get the configured options.
    #[must_use]
    pub fn options(&self) -> &XptWriterOptions {
        &self.options
    }

    /// Write the dataset to a file.
    ///
    /// Returns an error if validation failed or if writing fails.
    pub fn write_to_file(self, path: &Path, dataset: &XptDataset) -> crate::Result<()> {
        if !self.is_valid() {
            if let Some(error) = self.result.errors.first() {
                return Err(XptError::invalid_format(error.to_string()));
            }
            return Err(XptError::invalid_format("Validation failed"));
        }

        let file = File::create(path)?;
        self.write_to(file, dataset)
    }

    /// Write the dataset to any writer.
    pub fn write_to<W: Write>(self, writer: W, dataset: &XptDataset) -> crate::Result<()> {
        if !self.is_valid() {
            if let Some(error) = self.result.errors.first() {
                return Err(XptError::invalid_format(error.to_string()));
            }
            return Err(XptError::invalid_format("Validation failed"));
        }

        use super::XptWriter;
        let xpt_writer = XptWriter::with_options(writer, self.options);
        xpt_writer.write_dataset(dataset)
    }
}

/// Quick validation of a dataset without building a writer.
///
/// Performs basic validation using the validation framework.
///
/// # Arguments
/// * `dataset` - The dataset to validate
/// * `version` - The XPT version to validate against
///
/// # Returns
/// `Ok(())` if valid, `Err` with the first validation error otherwise.
pub fn validate_dataset(dataset: &XptDataset, version: XptVersion) -> Result<()> {
    validate_dataset_quick(dataset, version)
}

/// Quick validation that returns on first error (for backward compatibility).
pub(crate) fn validate_dataset_quick(dataset: &XptDataset, version: XptVersion) -> Result<()> {
    // Validate dataset name
    let name = normalize_name(&dataset.name);
    if name.is_empty() {
        return Err(XptError::invalid_dataset_name(&dataset.name));
    }
    if name.len() > version.dataset_name_limit() {
        return Err(XptError::dataset_name_too_long(
            &dataset.name,
            version.dataset_name_limit(),
        ));
    }

    // Validate dataset label (always max 40 chars)
    if let Some(label) = &dataset.label
        && label.len() > 40
    {
        return Err(XptError::dataset_label_too_long(&dataset.name));
    }

    // Check for duplicate column names and validate each column
    let mut seen = BTreeSet::new();
    for column in &dataset.columns {
        let col_name = normalize_name(&column.name);

        if col_name.is_empty() {
            return Err(XptError::invalid_variable_name(&column.name));
        }
        if col_name.len() > version.variable_name_limit() {
            return Err(XptError::variable_name_too_long(
                &column.name,
                version.variable_name_limit(),
            ));
        }

        if !seen.insert(col_name.clone()) {
            return Err(XptError::duplicate_variable(&column.name));
        }

        if column.length == 0 {
            return Err(XptError::zero_length(&column.name));
        }

        // Validate label length
        if let Some(label) = &column.label
            && label.len() > version.variable_label_limit()
        {
            return Err(XptError::variable_label_too_long(
                &column.name,
                version.variable_label_limit(),
            ));
        }

        // Validate format name length
        if let Some(format) = &column.format
            && format.len() > version.format_name_limit()
        {
            return Err(XptError::format_name_too_long(
                format,
                version.format_name_limit(),
            ));
        }

        // Validate informat name length
        if let Some(informat) = &column.informat
            && informat.len() > version.format_name_limit()
        {
            return Err(XptError::format_name_too_long(
                informat,
                version.format_name_limit(),
            ));
        }
    }

    // Validate row lengths
    for (row_idx, row) in dataset.rows.iter().enumerate() {
        if row.len() != dataset.columns.len() {
            return Err(XptError::row_length_mismatch(
                row_idx,
                dataset.columns.len(),
                row.len(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::XptColumn;

    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = XptWriterBuilder::new();
        assert_eq!(builder.options.version, XptVersion::V5);
    }

    #[test]
    fn test_builder_fda_compliant() {
        let builder = XptWriterBuilder::new().fda_compliant();
        assert_eq!(builder.options.version, XptVersion::V5);
        assert_eq!(builder.mode, ValidationMode::FdaCompliant);
    }

    #[test]
    fn test_builder_v8() {
        let builder = XptWriterBuilder::new().v8();
        assert_eq!(builder.options.version, XptVersion::V8);
    }

    #[test]
    fn test_validate_valid_dataset() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::character("NAME", 20)],
        );

        let result = XptWriterBuilder::new().validate(&dataset);
        assert!(result.is_valid());
        assert!(result.errors().is_empty());
    }

    #[test]
    fn test_validate_empty_name() {
        let dataset = XptDataset::new("");
        let result = XptWriterBuilder::new().validate(&dataset);
        assert!(!result.is_valid());
        assert!(!result.errors().is_empty());
    }

    #[test]
    fn test_validate_long_name_v5() {
        let dataset = XptDataset::new("VERYLONGNAME");
        let result = XptWriterBuilder::new().v5().validate(&dataset);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_validate_long_name_v8() {
        let dataset = XptDataset::new("VERYLONGNAME"); // 12 chars, OK for V8
        let result = XptWriterBuilder::new().v8().validate(&dataset);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_dataset_quick() {
        let dataset = XptDataset::with_columns("TEST", vec![XptColumn::numeric("AGE")]);
        assert!(validate_dataset(&dataset, XptVersion::V5).is_ok());
    }

    #[test]
    fn test_validate_duplicate_columns() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::numeric("AGE")],
        );
        assert!(validate_dataset(&dataset, XptVersion::V5).is_err());
    }
}
