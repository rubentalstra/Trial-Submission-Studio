//! QS (Questionnaires) domain preprocessing.
//!
//! Handles inference of QS test fields from source column hints.

use anyhow::Result;
use polars::prelude::DataFrame;

use crate::ct_utils::resolve_ct_for_variable;
use crate::data_utils::{fill_string_column, sanitize_test_code};

use super::common::PreprocessContext;
use super::rule_table::{PreprocessRule, RuleCategory, RuleMetadata};

/// Preprocess QS domain DataFrame.
///
/// # SDTMIG Reference
///
/// Questionnaire domains follow Findings class structure (Chapter 6.3).
/// QSTEST, QSTESTCD, and QSCAT are populated from source column labels.
pub fn preprocess_qs(ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
    let rule = QsTestFieldRule::new();
    rule.apply(ctx, df)
}

/// Rule for inferring QS test fields from source column hints.
pub struct QsTestFieldRule {
    metadata: RuleMetadata,
}

impl QsTestFieldRule {
    pub fn new() -> Self {
        Self {
            metadata: RuleMetadata::new(
                "QS_TEST_FIELD",
                RuleCategory::TestField,
                "Infer QSTEST, QSTESTCD, QSCAT from source column hints",
            )
            .with_targets(&["QSTEST", "QSTESTCD", "QSCAT"]),
        }
    }
}

impl Default for QsTestFieldRule {
    fn default() -> Self {
        Self::new()
    }
}

impl PreprocessRule for QsTestFieldRule {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn apply(&self, ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
        let require_explicit = ctx.ctx.options.require_explicit_mapping;

        // Try to find source column for QSORRES or QSSTRESC
        let orres_source = ctx
            .mapping_source("QSORRES")
            .or_else(|| ctx.mapping_source("QSSTRESC"));

        // If explicit mapping required but no mapping found, skip
        if require_explicit && orres_source.is_none() {
            return Ok(());
        }

        // Try to get label hint from various sources
        let label_hint = orres_source
            .as_deref()
            .and_then(|col| ctx.column_hint(col))
            .or_else(|| {
                // Only use fallback hints if explicit mapping not required
                if require_explicit {
                    None
                } else {
                    ctx.column_hint("QSPGARS")
                        .or_else(|| ctx.column_hint("QSPGARSCD"))
                }
            });

        if let Some((label, allow_raw)) = label_hint {
            let test_code = sanitize_test_code(&label);
            fill_string_column(df, "QSTEST", &label)?;
            fill_string_column(df, "QSTESTCD", &test_code)?;

            if let Some(qscat) =
                resolve_ct_for_variable(ctx.ctx, ctx.domain, "QSCAT", &label, allow_raw)
            {
                fill_string_column(df, "QSCAT", &qscat)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qs_test_field_rule_metadata() {
        let rule = QsTestFieldRule::new();
        let meta = rule.metadata();
        assert_eq!(meta.id, "QS_TEST_FIELD");
        assert_eq!(meta.category, RuleCategory::TestField);
        assert!(meta.target_variables.contains(&"QSTEST".to_string()));
        assert!(meta.target_variables.contains(&"QSTESTCD".to_string()));
        assert!(meta.target_variables.contains(&"QSCAT".to_string()));
    }
}
