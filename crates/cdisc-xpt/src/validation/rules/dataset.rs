//! Dataset structure validation rules.

use std::collections::BTreeSet;

use crate::error::{ErrorLocation, Severity, ValidationError, ValidationErrorCode};
use crate::header::normalize_name;
use crate::types::{XptColumn, XptDataset};
use crate::validation::{ValidationContext, ValidationMode, ValidationRule};

/// Checks for duplicate variable names in a dataset.
pub struct DuplicateVariableRule;

impl ValidationRule for DuplicateVariableRule {
    fn name(&self) -> &'static str {
        "DuplicateVariable"
    }

    fn applies_to(&self, _mode: ValidationMode) -> bool {
        true
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let mut seen = BTreeSet::new();

        for (index, column) in dataset.columns.iter().enumerate() {
            let normalized = normalize_name(&column.name);

            if !seen.insert(normalized.clone()) {
                errors.push(ValidationError::new(
                    ValidationErrorCode::DuplicateColumnName,
                    format!(
                        "Duplicate variable name '{}' at index {}",
                        column.name, index
                    ),
                    ErrorLocation::Column {
                        dataset: dataset.name.clone(),
                        column: column.name.clone(),
                        index,
                    },
                    Severity::Error,
                ));
            }
        }

        errors
    }
}

/// Validates variable lengths are non-zero.
pub struct VariableLengthRule;

impl ValidationRule for VariableLengthRule {
    fn name(&self) -> &'static str {
        "VariableLength"
    }

    fn applies_to(&self, _mode: ValidationMode) -> bool {
        true
    }

    fn validate_column(
        &self,
        column: &XptColumn,
        index: usize,
        dataset_name: &str,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if column.length == 0 {
            errors.push(ValidationError::new(
                ValidationErrorCode::ZeroLengthColumn,
                format!("Variable '{}' has zero length", column.name),
                ErrorLocation::Column {
                    dataset: dataset_name.to_string(),
                    column: column.name.clone(),
                    index,
                },
                Severity::Error,
            ));
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::XptVersion;

    fn make_context() -> ValidationContext {
        ValidationContext::new(XptVersion::V5, ValidationMode::Basic)
    }

    #[test]
    fn test_no_duplicates() {
        let rule = DuplicateVariableRule;
        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.columns.push(XptColumn::numeric("AGE"));
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_duplicate_names() {
        let rule = DuplicateVariableRule;
        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.columns.push(XptColumn::character("USUBJID", 20)); // duplicate
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::DuplicateColumnName);
    }

    #[test]
    fn test_duplicate_case_insensitive() {
        let rule = DuplicateVariableRule;
        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.columns.push(XptColumn::character("usubjid", 20)); // same when normalized
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_valid_length() {
        let rule = VariableLengthRule;
        let column = XptColumn::character("USUBJID", 20);
        let ctx = make_context();

        let errors = rule.validate_column(&column, 0, "DM", &ctx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_zero_length() {
        let rule = VariableLengthRule;
        let column = XptColumn::character("USUBJID", 0);
        let ctx = make_context();

        let errors = rule.validate_column(&column, 0, "DM", &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::ZeroLengthColumn);
    }
}
