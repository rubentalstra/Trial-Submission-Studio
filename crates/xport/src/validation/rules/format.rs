//! Format and informat validation rules.

use crate::error::{ErrorLocation, Severity, ValidationError, ValidationErrorCode};
use crate::types::XptColumn;
use crate::validation::{ValidationContext, ValidationMode, ValidationRule};

/// Validates format and informat names.
pub struct FormatNameRule;

impl ValidationRule for FormatNameRule {
    fn name(&self) -> &'static str {
        "FormatName"
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
        let limit = ctx.format_name_limit();
        let location = ErrorLocation::Column {
            dataset: dataset_name.to_string(),
            column: column.name.clone(),
            index,
        };

        // Validate format name
        if let Some(format) = &column.format
            && format.len() > limit
        {
            errors.push(ValidationError::new(
                ValidationErrorCode::FormatNameTooLong,
                format!(
                    "Format name '{}' for variable '{}' exceeds {} character limit",
                    format, column.name, limit
                ),
                location.clone(),
                Severity::Error,
            ));
        }

        // Validate informat name
        if let Some(informat) = &column.informat
            && informat.len() > limit
        {
            errors.push(ValidationError::new(
                ValidationErrorCode::InformatNameTooLong,
                format!(
                    "Informat name '{}' for variable '{}' exceeds {} character limit",
                    informat, column.name, limit
                ),
                location,
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

    fn make_context(version: XptVersion) -> ValidationContext {
        ValidationContext::new(version, ValidationMode::Basic)
    }

    #[test]
    fn test_format_valid() {
        let rule = FormatNameRule;
        let mut column = XptColumn::numeric("AGE");
        column.format = Some("BEST12.".to_string());
        let ctx = make_context(XptVersion::V5);

        let errors = rule.validate_column(&column, 0, "DM", &ctx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_format_too_long_v5() {
        let rule = FormatNameRule;
        let mut column = XptColumn::numeric("AGE");
        column.format = Some("LONGFORMAT".to_string()); // > 8 chars
        let ctx = make_context(XptVersion::V5);

        let errors = rule.validate_column(&column, 0, "DM", &ctx);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::FormatNameTooLong)
        );
    }

    #[test]
    fn test_format_ok_v8() {
        let rule = FormatNameRule;
        let mut column = XptColumn::numeric("AGE");
        column.format = Some("LONGFORMAT".to_string()); // > 8 but < 32
        let ctx = make_context(XptVersion::V8);

        let errors = rule.validate_column(&column, 0, "DM", &ctx);
        assert!(
            errors
                .iter()
                .all(|e| e.code != ValidationErrorCode::FormatNameTooLong)
        );
    }

    #[test]
    fn test_informat_too_long() {
        let rule = FormatNameRule;
        let mut column = XptColumn::numeric("AGE");
        column.informat = Some("VERYLONGINFORMAT".to_string());
        let ctx = make_context(XptVersion::V5);

        let errors = rule.validate_column(&column, 0, "DM", &ctx);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::InformatNameTooLong)
        );
    }
}
