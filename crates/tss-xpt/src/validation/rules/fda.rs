//! FDA-specific validation rules.
//!
//! These rules implement FDA requirements for XPT files submitted
//! as part of regulatory submissions.

use crate::error::{ErrorLocation, Severity, ValidationError, ValidationErrorCode};
use crate::types::{XptDataset, XptValue};
use crate::validation::{ValidationContext, ValidationMode, ValidationRule};

use super::is_ascii_printable;

/// Validates that FDA submissions use V5 format.
///
/// FDA requires SAS Transport V5 format for regulatory submissions.
pub struct FdaVersionRule;

impl ValidationRule for FdaVersionRule {
    fn name(&self) -> &'static str {
        "FdaVersion"
    }

    fn applies_to(&self, mode: ValidationMode) -> bool {
        mode == ValidationMode::FdaCompliant
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // FDA requires V5 format
        if ctx.version() != crate::version::XptVersion::V5 {
            errors.push(ValidationError::new(
                ValidationErrorCode::WrongVersion,
                "FDA submissions require SAS Transport V5 format".to_string(),
                ErrorLocation::Dataset {
                    name: dataset.name.clone(),
                },
                Severity::Error,
            ));
        }

        errors
    }
}

/// Validates that character data contains only ASCII.
///
/// FDA requires all character data to be ASCII-only.
pub struct FdaAsciiRule;

impl ValidationRule for FdaAsciiRule {
    fn name(&self) -> &'static str {
        "FdaAscii"
    }

    fn applies_to(&self, mode: ValidationMode) -> bool {
        mode == ValidationMode::FdaCompliant
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check character data in rows
        for (row_idx, row) in dataset.rows.iter().enumerate() {
            for (col_idx, value) in row.iter().enumerate() {
                if let XptValue::Char(s) = value {
                    if !is_ascii_printable(s) {
                        let col_name = dataset
                            .columns
                            .get(col_idx)
                            .map(|c| c.name.as_str())
                            .unwrap_or("unknown");

                        errors.push(ValidationError::new(
                            ValidationErrorCode::NonAsciiValue,
                            format!(
                                "Non-ASCII character in column '{}' at row {}",
                                col_name, row_idx
                            ),
                            ErrorLocation::Value {
                                dataset: dataset.name.clone(),
                                column: col_name.to_string(),
                                row: row_idx,
                            },
                            Severity::Error,
                        ));
                    }
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::XptColumn;
    use crate::version::XptVersion;

    fn make_fda_context(version: XptVersion) -> ValidationContext {
        ValidationContext::new(version, ValidationMode::FdaCompliant)
    }

    #[allow(dead_code)]
    fn make_basic_context() -> ValidationContext {
        ValidationContext::new(XptVersion::V5, ValidationMode::Basic)
    }

    #[test]
    fn test_fda_version_v5_ok() {
        let rule = FdaVersionRule;
        let dataset = XptDataset::new("DM");
        let ctx = make_fda_context(XptVersion::V5);

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_fda_version_v8_error() {
        let rule = FdaVersionRule;
        let dataset = XptDataset::new("DM");
        let ctx = make_fda_context(XptVersion::V8);

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::WrongVersion);
    }

    #[test]
    fn test_fda_version_not_applied_basic() {
        let rule = FdaVersionRule;
        assert!(!rule.applies_to(ValidationMode::Basic));
    }

    #[test]
    fn test_fda_ascii_ok() {
        let rule = FdaAsciiRule;
        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.rows.push(vec![XptValue::character("SUBJ001")]);
        let ctx = make_fda_context(XptVersion::V5);

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_fda_ascii_error() {
        let rule = FdaAsciiRule;
        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.rows.push(vec![XptValue::character("SÜBJ001")]); // non-ASCII
        let ctx = make_fda_context(XptVersion::V5);

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::NonAsciiValue);
    }

    #[test]
    fn test_fda_ascii_not_applied_basic() {
        let rule = FdaAsciiRule;
        assert!(!rule.applies_to(ValidationMode::Basic));
    }
}
