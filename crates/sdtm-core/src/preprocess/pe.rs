//! PE (Physical Examination) domain preprocessing.
//!
//! Handles inference of PE test fields from source column hints.

use anyhow::Result;
use polars::prelude::DataFrame;

use crate::data_utils::{fill_string_column, sanitize_test_code};

use super::common::PreprocessContext;
use super::rule_table::{PreprocessRule, RuleCategory, RuleMetadata};

/// Preprocess PE domain DataFrame.
///
/// # SDTMIG Reference
///
/// Physical Examination domain follows Findings class structure (Chapter 6.3.6).
/// PETEST and PETESTCD are populated from source column labels.
pub fn preprocess_pe(ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
    let rule = PeTestFieldRule::new();
    rule.apply(ctx, df)
}

/// Rule for inferring PE test fields from source column hints.
pub struct PeTestFieldRule {
    metadata: RuleMetadata,
}

impl PeTestFieldRule {
    pub fn new() -> Self {
        Self {
            metadata: RuleMetadata::new(
                "PE_TEST_FIELD",
                RuleCategory::TestField,
                "Infer PETEST, PETESTCD from source column hints",
            )
            .with_targets(&["PETEST", "PETESTCD"]),
        }
    }
}

impl Default for PeTestFieldRule {
    fn default() -> Self {
        Self::new()
    }
}

impl PreprocessRule for PeTestFieldRule {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn apply(&self, ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
        let require_explicit = ctx.ctx.options.require_explicit_mapping;

        // Try to find source column for PEORRES
        let orres_source = ctx
            .mapping_source("PEORRES")
            .or_else(|| ctx.mapping_source("PEORRESSP"));

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
                    ctx.column_hint("PEORRES")
                        .or_else(|| ctx.column_hint("PEORRESSP"))
                }
            });

        if let Some((label, _allow_raw)) = label_hint {
            let test_code = sanitize_test_code(&label);
            fill_string_column(df, "PETEST", &label)?;
            fill_string_column(df, "PETESTCD", &test_code)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pe_test_field_rule_metadata() {
        let rule = PeTestFieldRule::new();
        let meta = rule.metadata();
        assert_eq!(meta.id, "PE_TEST_FIELD");
        assert_eq!(meta.category, RuleCategory::TestField);
        assert!(meta.target_variables.contains(&"PETEST".to_string()));
        assert!(meta.target_variables.contains(&"PETESTCD".to_string()));
    }
}
