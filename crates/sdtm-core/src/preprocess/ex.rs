//! EX (Exposure) domain preprocessing.
//!
//! Handles inference of EXTRT from treatment-related column hints.

use anyhow::Result;
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};

use sdtm_ingest::any_to_string;
use sdtm_model::CaseInsensitiveLookup;

use crate::data_utils::{table_column_values, table_label};

use super::common::PreprocessContext;
use super::rule_table::{PreprocessRule, RuleCategory, RuleMetadata};

/// Preprocess EX domain DataFrame.
///
/// # SDTMIG Reference
///
/// Exposure domain follows Interventions class structure (Chapter 6.1.2).
/// EXTRT is populated from treatment-related column hints.
pub fn preprocess_ex(ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
    let rule = ExTreatmentRule::new();
    rule.apply(ctx, df)
}

/// Rule for inferring EX treatment fields.
pub struct ExTreatmentRule {
    metadata: RuleMetadata,
}

impl ExTreatmentRule {
    pub fn new() -> Self {
        Self {
            metadata: RuleMetadata::new(
                "EX_TREATMENT",
                RuleCategory::Treatment,
                "Infer EXTRT from treatment-related column hints",
            )
            .with_targets(&["EXTRT"]),
        }
    }
}

impl Default for ExTreatmentRule {
    fn default() -> Self {
        Self::new()
    }
}

impl PreprocessRule for ExTreatmentRule {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn apply(&self, ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
        let column_lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());
        let column_name = |name: &str| {
            column_lookup
                .get(name)
                .map(|value| value.to_string())
                .unwrap_or_else(|| name.to_string())
        };

        let extrt_col = column_name("EXTRT");
        let require_explicit = ctx.ctx.options.require_explicit_mapping;

        // Get existing EXTRT values
        let mut extrt_vals = if let Ok(series) = df.column(&extrt_col) {
            (0..df.height())
                .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                .collect::<Vec<_>>()
        } else {
            vec![String::new(); df.height()]
        };

        // Build set of standard variables to exclude from candidate search
        let standard_vars = ctx.standard_variable_set();

        // Build candidate headers list
        let mut candidate_headers: Vec<String> = Vec::new();

        // Check for explicit mapping source
        let has_explicit_mapping = ctx.mapping_source("EXTRT").is_some();

        // Prefer explicit mapping source
        if let Some(preferred) = ctx.mapping_source("EXTRT") {
            candidate_headers.push(preferred);
        }

        // Only search for heuristic candidates if explicit mapping not required
        if !require_explicit || has_explicit_mapping {
            // Search for treatment-related columns
            let keywords = ["TREAT", "DRUG", "THERAP", "INTERVENT"];
            for header in &ctx.table.headers {
                if standard_vars.contains(&header.to_uppercase()) {
                    continue;
                }
                let label = table_label(ctx.table, header).unwrap_or_default();
                let mut hay = header.to_uppercase();
                if !label.is_empty() {
                    hay.push(' ');
                    hay.push_str(&label.to_uppercase());
                }
                if keywords.iter().any(|kw| hay.contains(kw)) {
                    candidate_headers.push(header.clone());
                }
            }

            // Fallback to common event/activity columns
            for fallback in ["EventName", "ActivityName"] {
                if ctx
                    .table
                    .headers
                    .iter()
                    .any(|header| header.eq_ignore_ascii_case(fallback))
                {
                    candidate_headers.push(fallback.to_string());
                }
            }
        }

        candidate_headers.sort();
        candidate_headers.dedup();

        // Get values from candidate columns
        let mut candidates: Vec<Vec<String>> = Vec::new();
        for header in candidate_headers {
            if let Some(values) = table_column_values(ctx.table, &header)
                && values.iter().any(|value| !value.trim().is_empty())
            {
                candidates.push(values);
            }
        }

        // Fill missing EXTRT values from candidates
        if !candidates.is_empty() {
            for (idx, extrt_value) in extrt_vals.iter_mut().enumerate().take(df.height()) {
                if !extrt_value.trim().is_empty() {
                    continue;
                }
                for values in &candidates {
                    let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
                    if !value.is_empty() {
                        *extrt_value = value.to_string();
                        break;
                    }
                }
            }
            df.with_column(Series::new(extrt_col.as_str().into(), extrt_vals))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ex_treatment_rule_metadata() {
        let rule = ExTreatmentRule::new();
        let meta = rule.metadata();
        assert_eq!(meta.id, "EX_TREATMENT");
        assert_eq!(meta.category, RuleCategory::Treatment);
        assert!(meta.target_variables.contains(&"EXTRT".to_string()));
    }
}
