//! DS (Disposition) domain preprocessing.
//!
//! Handles inference of DSDECOD and DSTERM from CT matches and completion columns.

use anyhow::Result;
use polars::prelude::{AnyValue, NamedFrom, Series};

use sdtm_ingest::any_to_string;
use sdtm_model::CaseInsensitiveLookup;

use crate::ct_utils::{completion_column, ct_column_match};

use super::common::PreprocessContext;
use super::rule_table::{PreprocessRule, RuleCategory, RuleMetadata};

use polars::prelude::DataFrame;

/// Preprocess DS domain DataFrame.
///
/// # SDTMIG Reference
///
/// Disposition domain follows Events class structure (Chapter 6.2.2).
/// DSDECOD and DSTERM are populated from CT matches and completion columns.
pub fn preprocess_ds(ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
    let rule = DsDecodeTermRule::new();
    rule.apply(ctx, df)
}

/// Rule for inferring DS decode and term fields.
pub struct DsDecodeTermRule {
    metadata: RuleMetadata,
}

impl DsDecodeTermRule {
    pub fn new() -> Self {
        Self {
            metadata: RuleMetadata::new(
                "DS_DECODE_TERM",
                RuleCategory::DecodeTerm,
                "Infer DSDECOD, DSTERM from CT matches and completion columns",
            )
            .with_targets(&["DSDECOD", "DSTERM"]),
        }
    }
}

impl Default for DsDecodeTermRule {
    fn default() -> Self {
        Self::new()
    }
}

impl PreprocessRule for DsDecodeTermRule {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn apply(&self, ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
        let require_explicit = ctx.ctx.options.require_explicit_mapping;

        // Check for explicit mappings
        let has_explicit_decod = ctx.mapping_source("DSDECOD").is_some();
        let has_explicit_term = ctx.mapping_source("DSTERM").is_some();

        // If explicit mapping required and no mappings found, skip heuristic inference
        if require_explicit && !has_explicit_decod && !has_explicit_term {
            return Ok(());
        }

        let column_lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());
        let column_name = |name: &str| {
            column_lookup
                .get(name)
                .map(|value| value.to_string())
                .unwrap_or_else(|| name.to_string())
        };

        let decod_col = column_name("DSDECOD");
        let term_col = column_name("DSTERM");

        // Get existing values
        let mut decod_vals = if let Ok(series) = df.column(&decod_col) {
            (0..df.height())
                .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                .collect::<Vec<_>>()
        } else {
            vec![String::new(); df.height()]
        };

        let mut term_vals = if let Ok(series) = df.column(&term_col) {
            (0..df.height())
                .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                .collect::<Vec<_>>()
        } else {
            vec![String::new(); df.height()]
        };

        // Try to match from CT columns (heuristic, skip if require_explicit)
        if !require_explicit {
            if let Some(ct) = ctx.ctx.resolve_ct(ctx.domain, "DSDECOD")
                && let Some((_header, mapped, raw)) = ct_column_match(ctx.table, ctx.domain, ct)
            {
                for idx in 0..df.height().min(mapped.len()).min(raw.len()) {
                    if decod_vals[idx].trim().is_empty()
                        && let Some(ct_value) = &mapped[idx]
                    {
                        decod_vals[idx] = ct_value.clone();
                    }
                    if term_vals[idx].trim().is_empty() && !raw[idx].trim().is_empty() {
                        term_vals[idx] = raw[idx].trim().to_string();
                    }
                }
            }

            // Try completion column fallback
            if let Some((values, label)) = completion_column(ctx.table, ctx.domain) {
                for idx in 0..df.height().min(values.len()) {
                    if decod_vals[idx].trim().is_empty() && !values[idx].trim().is_empty() {
                        decod_vals[idx] = values[idx].trim().to_string();
                    }
                    if term_vals[idx].trim().is_empty() && !label.trim().is_empty() {
                        term_vals[idx] = label.clone();
                    }
                }
            }
        }

        // Update DataFrame columns
        df.with_column(Series::new(decod_col.as_str().into(), decod_vals))?;
        df.with_column(Series::new(term_col.as_str().into(), term_vals))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ds_decode_term_rule_metadata() {
        let rule = DsDecodeTermRule::new();
        let meta = rule.metadata();
        assert_eq!(meta.id, "DS_DECODE_TERM");
        assert_eq!(meta.category, RuleCategory::DecodeTerm);
        assert!(meta.target_variables.contains(&"DSDECOD".to_string()));
        assert!(meta.target_variables.contains(&"DSTERM".to_string()));
    }
}
