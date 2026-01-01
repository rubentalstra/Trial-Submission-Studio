//! Label validation rules for datasets and variables.

use crate::error::{ErrorLocation, Severity, ValidationError, ValidationErrorCode};
use crate::types::{XptColumn, XptDataset};
use crate::validation::{ValidationContext, ValidationMode, ValidationRule};

use super::is_ascii_printable;

/// Validates dataset labels.
pub struct DatasetLabelRule;

impl ValidationRule for DatasetLabelRule {
    fn name(&self) -> &'static str {
        "DatasetLabel"
    }

    fn applies_to(&self, _mode: ValidationMode) -> bool {
        true
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if let Some(label) = &dataset.label {
            let limit = ctx.dataset_label_limit();
            
            // Check length
            if label.len() > limit {
                errors.push(ValidationError::new(
                    ValidationErrorCode::LabelTooLong,
                    format!(
                        "Dataset label exceeds {} character limit ({} chars)",
                        limit,
                        label.len()
                    ),
                    ErrorLocation::Dataset {
                        name: dataset.name.clone(),
                    },
                    Severity::Error,
                ));
            }

            // Check for non-printable ASCII (FDA requirement)
            if !is_ascii_printable(label) {
                errors.push(ValidationError::new(
                    ValidationErrorCode::NonAsciiLabel,
                    "Dataset label contains non-printable or non-ASCII characters".to_string(),
                    ErrorLocation::Dataset {
                        name: dataset.name.clone(),
                    },
                    if ctx.is_fda_compliant() {
                        Severity::Error
                    } else {
                        Severity::Warning
                    },
                ));
            }
        }

        errors
    }
}

/// Validates variable (column) labels.
pub struct VariableLabelRule;

impl ValidationRule for VariableLabelRule {
    fn name(&self) -> &'static str {
        "VariableLabel"
    }

    fn applies_to(&self, _mode: ValidationMode) -> bool {
        true
    }

    fn validate_column(
        &self,
        column: &XptColumn,
        index: usize,
        dataset_name: &str,
        ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if let Some(label) = &column.label {
            let limit = ctx.variable_label_limit();
            let location = ErrorLocation::Column {
                dataset: dataset_name.to_string(),
                column: column.name.clone(),
                index,
            };

            // Check length
            if label.len() > limit {
                errors.push(ValidationError::new(
                    ValidationErrorCode::LabelTooLong,
                    format!(
                        "Variable label for '{}' exceeds {} character limit ({} chars)",
                        column.name,
                        limit,
                        label.len()
                    ),
                    location.clone(),
                    Severity::Error,
                ));
            }

            // Check for non-printable ASCII
            if !is_ascii_printable(label) {
                errors.push(ValidationError::new(
                    ValidationErrorCode::NonAsciiLabel,
                    format!(
                        "Variable label for '{}' contains non-printable or non-ASCII characters",
                        column.name
                    ),
                    location,
                    if ctx.is_fda_compliant() {
                        Severity::Error
                    } else {
                        Severity::Warning
                    },
                ));
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::XptVersion;

    fn make_context(version: XptVersion, fda: bool) -> ValidationContext {
        ValidationContext::new(
            version,
            if fda { ValidationMode::FdaCompliant } else { ValidationMode::Basic },
        )
    }

    #[test]
    fn test_dataset_label_valid() {
        let rule = DatasetLabelRule;
        let mut dataset = XptDataset::new("DM");
        dataset.label = Some("Demographics".to_string());
        let ctx = make_context(XptVersion::V5, false);

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_dataset_label_too_long() {
        let rule = DatasetLabelRule;
        let mut dataset = XptDataset::new("DM");
        dataset.label = Some("A".repeat(50)); // > 40 chars
        let ctx = make_context(XptVersion::V5, false);

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(errors.iter().any(|e| e.code == ValidationErrorCode::LabelTooLong));
    }

    #[test]
    fn test_variable_label_v8_longer() {
        let rule = VariableLabelRule;
        let mut column = XptColumn::character("USUBJID", 20);
        column.label = Some("A".repeat(100)); // > 40 but < 256
        
        let ctx_v5 = make_context(XptVersion::V5, false);
        let ctx_v8 = make_context(XptVersion::V8, false);

        let errors_v5 = rule.validate_column(&column, 0, "DM", &ctx_v5);
        let errors_v8 = rule.validate_column(&column, 0, "DM", &ctx_v8);

        assert!(errors_v5.iter().any(|e| e.code == ValidationErrorCode::LabelTooLong));
        assert!(errors_v8.iter().all(|e| e.code != ValidationErrorCode::LabelTooLong));
    }

    #[test]
    fn test_label_non_ascii_fda() {
        let rule = VariableLabelRule;
        let mut column = XptColumn::character("USUBJID", 20);
        column.label = Some("Subject Idéntifier".to_string()); // non-ASCII 'é'
        
        let ctx_fda = make_context(XptVersion::V5, true);
        let ctx_basic = make_context(XptVersion::V5, false);

        let errors_fda = rule.validate_column(&column, 0, "DM", &ctx_fda);
        let errors_basic = rule.validate_column(&column, 0, "DM", &ctx_basic);

        // FDA mode: error
        assert!(errors_fda.iter().any(|e| 
            e.code == ValidationErrorCode::NonAsciiLabel && e.severity == Severity::Error
        ));
        // Basic mode: warning
        assert!(errors_basic.iter().any(|e| 
            e.code == ValidationErrorCode::NonAsciiLabel && e.severity == Severity::Warning
        ));
    }
}
