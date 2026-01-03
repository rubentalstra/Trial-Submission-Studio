//! Name validation rules for datasets and variables.

use crate::error::{ErrorLocation, Severity, ValidationError, ValidationErrorCode};
use crate::header::normalize_name;
use crate::types::{XptColumn, XptDataset};
use crate::validation::{ValidationContext, ValidationMode, ValidationRule};

use super::is_valid_sas_name;

/// Validates dataset names.
pub struct DatasetNameRule;

impl ValidationRule for DatasetNameRule {
    fn name(&self) -> &'static str {
        "DatasetName"
    }

    fn applies_to(&self, _mode: ValidationMode) -> bool {
        true // Always applies
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let normalized = normalize_name(&dataset.name);

        // Check for empty name
        if normalized.is_empty() {
            errors.push(ValidationError::new(
                ValidationErrorCode::EmptyName,
                "Dataset name cannot be empty".to_string(),
                ErrorLocation::Dataset {
                    name: dataset.name.clone(),
                },
                Severity::Error,
            ));
            return errors;
        }

        // Check length limit
        let limit = ctx.dataset_name_limit();
        if normalized.len() > limit {
            errors.push(ValidationError::new(
                ValidationErrorCode::NameTooLong,
                format!(
                    "Dataset name '{}' exceeds {} character limit ({} chars)",
                    dataset.name,
                    limit,
                    normalized.len()
                ),
                ErrorLocation::Dataset {
                    name: dataset.name.clone(),
                },
                Severity::Error,
            ));
        }

        // Check valid characters
        if !is_valid_sas_name(&normalized) {
            errors.push(ValidationError::new(
                ValidationErrorCode::InvalidNameCharacter,
                format!(
                    "Dataset name '{}' contains invalid characters (must be A-Z, 0-9, _ and start with letter)",
                    dataset.name
                ),
                ErrorLocation::Dataset {
                    name: dataset.name.clone(),
                },
                Severity::Error,
            ));
        }

        // Warning for lowercase (SAS convention is uppercase)
        if dataset.name != normalized {
            errors.push(ValidationError::new(
                ValidationErrorCode::LowercaseName,
                format!(
                    "Dataset name '{}' will be converted to uppercase '{}'",
                    dataset.name, normalized
                ),
                ErrorLocation::Dataset {
                    name: dataset.name.clone(),
                },
                Severity::Warning,
            ));
        }

        errors
    }
}

/// Validates variable (column) names.
pub struct VariableNameRule;

impl ValidationRule for VariableNameRule {
    fn name(&self) -> &'static str {
        "VariableName"
    }

    fn applies_to(&self, _mode: ValidationMode) -> bool {
        true // Always applies
    }

    fn validate_column(
        &self,
        column: &XptColumn,
        index: usize,
        dataset_name: &str,
        ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let normalized = normalize_name(&column.name);

        let location = ErrorLocation::Column {
            dataset: dataset_name.to_string(),
            column: column.name.clone(),
            index,
        };

        // Check for empty name
        if normalized.is_empty() {
            errors.push(ValidationError::new(
                ValidationErrorCode::EmptyName,
                format!("Variable name at index {} cannot be empty", index),
                location.clone(),
                Severity::Error,
            ));
            return errors;
        }

        // Check length limit
        let limit = ctx.variable_name_limit();
        if normalized.len() > limit {
            errors.push(ValidationError::new(
                ValidationErrorCode::NameTooLong,
                format!(
                    "Variable name '{}' exceeds {} character limit ({} chars)",
                    column.name,
                    limit,
                    normalized.len()
                ),
                location.clone(),
                Severity::Error,
            ));
        }

        // Check valid characters
        if !is_valid_sas_name(&normalized) {
            errors.push(ValidationError::new(
                ValidationErrorCode::InvalidNameCharacter,
                format!(
                    "Variable name '{}' contains invalid characters (must be A-Z, 0-9, _ and start with letter)",
                    column.name
                ),
                location.clone(),
                Severity::Error,
            ));
        }

        // Warning for lowercase
        if column.name != normalized {
            errors.push(ValidationError::new(
                ValidationErrorCode::LowercaseName,
                format!(
                    "Variable name '{}' will be converted to uppercase '{}'",
                    column.name, normalized
                ),
                location,
                Severity::Warning,
            ));
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::XptVersion;

    fn make_context(version: XptVersion) -> ValidationContext {
        ValidationContext::new(version, ValidationMode::Basic)
    }

    #[test]
    fn test_dataset_name_valid() {
        let rule = DatasetNameRule;
        let dataset = XptDataset::new("DM");
        let ctx = make_context(XptVersion::V5);

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(errors.iter().all(|e| e.severity != Severity::Error));
    }

    #[test]
    fn test_dataset_name_too_long_v5() {
        let rule = DatasetNameRule;
        let dataset = XptDataset::new("TOOLONGNAME");
        let ctx = make_context(XptVersion::V5);

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::NameTooLong)
        );
    }

    #[test]
    fn test_dataset_name_too_long_v8_ok() {
        let rule = DatasetNameRule;
        let dataset = XptDataset::new("TOOLONGNAME"); // 11 chars, < 32
        let ctx = make_context(XptVersion::V8);

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(
            errors
                .iter()
                .all(|e| e.code != ValidationErrorCode::NameTooLong)
        );
    }

    #[test]
    fn test_variable_name_valid() {
        let rule = VariableNameRule;
        let column = XptColumn::character("USUBJID", 20);
        let ctx = make_context(XptVersion::V5);

        let errors = rule.validate_column(&column, 0, "DM", &ctx);
        assert!(errors.iter().all(|e| e.severity != Severity::Error));
    }

    #[test]
    fn test_variable_name_invalid_chars() {
        let rule = VariableNameRule;
        let column = XptColumn::character("VAR-1", 20);
        let ctx = make_context(XptVersion::V5);

        let errors = rule.validate_column(&column, 0, "DM", &ctx);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::InvalidNameCharacter)
        );
    }
}
