//! Validation framework for XPT datasets.
//!
//! This module provides comprehensive validation for XPT datasets,
//! supporting both V5 and V8 formats with FDA compliance checking.
//!
//! # Architecture
//!
//! The validation framework uses a rule-based architecture:
//! - [`ValidationRule`] trait defines individual validation checks
//! - [`Validator`] orchestrates rule execution and collects results
//! - [`ValidationContext`] provides shared state during validation
//!
//! # Example
//!
//! ```
//! use xport::validation::{Validator, ValidationMode};
//! use xport::{XptDataset, XptVersion};
//!
//! let dataset = XptDataset::new("DM");
//! let validator = Validator::new(XptVersion::V5)
//!     .with_mode(ValidationMode::FdaCompliant);
//!
//! let result = validator.validate(&dataset);
//! if result.is_valid() {
//!     println!("Dataset is valid!");
//! } else {
//!     for error in &result.errors {
//!         eprintln!("Error: {}", error);
//!     }
//! }
//! ```

mod context;
pub mod rules;

pub use context::ValidationContext;

use crate::error::{ValidationError, ValidationResult};
use crate::types::{XptColumn, XptDataset};
use crate::version::XptVersion;

/// Validation mode determining which rules to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationMode {
    /// Basic XPT format validation only.
    #[default]
    Basic,
    /// Full FDA compliance validation (stricter).
    FdaCompliant,
    /// Custom validation with specific rules enabled.
    Custom,
}

/// A validation rule that can be applied to datasets or columns.
pub trait ValidationRule: Send + Sync {
    /// The name of this rule for reporting purposes.
    fn name(&self) -> &'static str;

    /// Whether this rule applies in the given mode.
    fn applies_to(&self, mode: ValidationMode) -> bool;

    /// Validate an entire dataset.
    fn validate_dataset(
        &self,
        _dataset: &XptDataset,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        Vec::new()
    }

    /// Validate a single column.
    fn validate_column(
        &self,
        _column: &XptColumn,
        _index: usize,
        _dataset_name: &str,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        Vec::new()
    }
}

/// Dataset validator that applies validation rules and collects errors.
///
/// The validator uses a collect-all-errors pattern, running all applicable
/// rules and returning all validation issues at once.
pub struct Validator {
    version: XptVersion,
    mode: ValidationMode,
    rules: Vec<Box<dyn ValidationRule>>,
}

impl Validator {
    /// Create a new validator for the specified XPT version.
    #[must_use]
    pub fn new(version: XptVersion) -> Self {
        let mut validator = Self {
            version,
            mode: ValidationMode::Basic,
            rules: Vec::new(),
        };
        validator.register_default_rules();
        validator
    }

    /// Create a validator with FDA compliance mode.
    #[must_use]
    pub fn fda_compliant(version: XptVersion) -> Self {
        Self::new(version).with_mode(ValidationMode::FdaCompliant)
    }

    /// Set the validation mode.
    #[must_use]
    pub fn with_mode(mut self, mode: ValidationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Add a custom validation rule.
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Get the XPT version being validated against.
    #[must_use]
    pub fn version(&self) -> XptVersion {
        self.version
    }

    /// Get the current validation mode.
    #[must_use]
    pub fn mode(&self) -> ValidationMode {
        self.mode
    }

    /// Validate a dataset and return all errors/warnings.
    #[must_use]
    pub fn validate(&self, dataset: &XptDataset) -> ValidationResult {
        let ctx = ValidationContext::new(self.version, self.mode);
        let mut result = ValidationResult::new();

        // Run dataset-level validation rules
        for rule in &self.rules {
            if rule.applies_to(self.mode) {
                let errors = rule.validate_dataset(dataset, &ctx);
                for error in errors {
                    result.add(error);
                }
            }
        }

        // Run column-level validation rules
        for (index, column) in dataset.columns.iter().enumerate() {
            for rule in &self.rules {
                if rule.applies_to(self.mode) {
                    let errors = rule.validate_column(column, index, &dataset.name, &ctx);
                    for error in errors {
                        result.add(error);
                    }
                }
            }
        }

        result
    }

    /// Validate and return an error if validation fails.
    ///
    /// This is a convenience method that returns `Ok(())` if validation passes,
    /// or `Err` with all validation errors if it fails.
    pub fn validate_or_error(&self, dataset: &XptDataset) -> Result<(), Vec<ValidationError>> {
        let result = self.validate(dataset);
        result.into_result()
    }

    /// Register the default set of validation rules.
    fn register_default_rules(&mut self) {
        use rules::{
            DatasetLabelRule, DatasetNameRule, DuplicateVariableRule, FdaAsciiRule, FdaVersionRule,
            FormatNameRule, VariableLabelRule, VariableLengthRule, VariableNameRule,
        };

        // Name validation rules
        self.rules.push(Box::new(DatasetNameRule));
        self.rules.push(Box::new(VariableNameRule));

        // Label validation rules
        self.rules.push(Box::new(DatasetLabelRule));
        self.rules.push(Box::new(VariableLabelRule));

        // Format validation rules
        self.rules.push(Box::new(FormatNameRule));

        // Dataset structure rules
        self.rules.push(Box::new(DuplicateVariableRule));
        self.rules.push(Box::new(VariableLengthRule));

        // FDA-specific rules
        self.rules.push(Box::new(FdaVersionRule));
        self.rules.push(Box::new(FdaAsciiRule));
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new(XptVersion::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::XptColumn;

    #[test]
    fn test_validator_new() {
        let validator = Validator::new(XptVersion::V5);
        assert_eq!(validator.version(), XptVersion::V5);
        assert_eq!(validator.mode(), ValidationMode::Basic);
    }

    #[test]
    fn test_validator_fda_compliant() {
        let validator = Validator::fda_compliant(XptVersion::V5);
        assert_eq!(validator.mode(), ValidationMode::FdaCompliant);
    }

    #[test]
    fn test_validate_empty_dataset() {
        let validator = Validator::new(XptVersion::V5);
        let dataset = XptDataset::new("DM");
        let result = validator.validate(&dataset);
        // Empty dataset should still be valid (no columns is allowed)
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_valid_dataset() {
        let validator = Validator::new(XptVersion::V5);
        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.columns.push(XptColumn::numeric("AGE"));

        let result = validator.validate(&dataset);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_invalid_name() {
        let validator = Validator::new(XptVersion::V5);
        let mut dataset = XptDataset::new("TOOLONGNAME"); // > 8 chars for V5
        dataset.columns.push(XptColumn::character("USUBJID", 20));

        let result = validator.validate(&dataset);
        assert!(!result.is_valid());
    }
}
