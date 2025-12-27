//! IE (Inclusion/Exclusion) domain preprocessing.
//!
//! Handles inference of IE test fields from column patterns.

use anyhow::Result;
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};

use sdtm_ingest::any_to_string;
use sdtm_model::CaseInsensitiveLookup;

use crate::ct_utils::{is_yes_no_token, resolve_ct_value_from_hint};
use crate::data_utils::{sanitize_test_code, table_column_values, table_label};

use super::common::PreprocessContext;
use super::rule_table::{PreprocessRule, RuleCategory, RuleMetadata};

/// Preprocess IE domain DataFrame.
///
/// # SDTMIG Reference
///
/// Inclusion/Exclusion domain follows Findings class structure (Chapter 6.3.5).
/// IETEST, IETESTCD, IECAT are populated from column patterns.
pub fn preprocess_ie(ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
    let rule = IeTestFieldRule::new();
    rule.apply(ctx, df)
}

/// Rule for inferring IE test and category fields.
pub struct IeTestFieldRule {
    metadata: RuleMetadata,
}

impl IeTestFieldRule {
    pub fn new() -> Self {
        Self {
            metadata: RuleMetadata::new(
                "IE_TEST_FIELD",
                RuleCategory::TestField,
                "Infer IETEST, IETESTCD, IECAT from column patterns",
            )
            .with_targets(&["IETEST", "IETESTCD", "IECAT"]),
        }
    }
}

impl Default for IeTestFieldRule {
    fn default() -> Self {
        Self::new()
    }
}

impl PreprocessRule for IeTestFieldRule {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn apply(&self, ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
        let require_explicit = ctx.ctx.options.require_explicit_mapping;

        // Check for explicit mappings
        let has_explicit_test = ctx.mapping_source("IETEST").is_some();
        let has_explicit_testcd = ctx.mapping_source("IETESTCD").is_some();
        let has_explicit_cat = ctx.mapping_source("IECAT").is_some();

        // If explicit mapping required and no mappings found, skip heuristic inference
        if require_explicit && !has_explicit_test && !has_explicit_testcd && !has_explicit_cat {
            return Ok(());
        }

        // Build candidate list: (label, values, category)
        let mut candidates: Vec<(String, Vec<String>, String)> = Vec::new();

        // Only search heuristically if explicit mapping not required
        if !require_explicit {
            let ct_cat = ctx.ctx.resolve_ct(ctx.domain, "IECAT");

            for header in &ctx.table.headers {
                let upper = header.to_uppercase();
                if !upper.starts_with("IE") {
                    continue;
                }

                let label = table_label(ctx.table, header).unwrap_or_else(|| header.clone());
                let category = ct_cat.and_then(|ct| resolve_ct_value_from_hint(ct, &label));

                if let Some(category) = category
                    && let Some(values) = table_column_values(ctx.table, header)
                {
                    candidates.push((label, values, category));
                }
            }
        }

        if candidates.is_empty() {
            return Ok(());
        }

        let column_lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());
        let column_name = |name: &str| {
            column_lookup
                .get(name)
                .map(|value| value.to_string())
                .unwrap_or_else(|| name.to_string())
        };

        let ietest_col = column_name("IETEST");
        let ietestcd_col = column_name("IETESTCD");
        let iecat_col = column_name("IECAT");
        let ieorres_col = column_name("IEORRES");

        // Get existing values
        let mut ietest_vals = get_column_values_or_empty(df, &ietest_col);
        let mut ietestcd_vals = get_column_values_or_empty(df, &ietestcd_col);
        let mut iecat_vals = get_column_values_or_empty(df, &iecat_col);
        let orres_vals = get_column_values_or_empty(df, &ieorres_col);

        // Fill missing values from candidates
        for idx in 0..df.height() {
            let testcd_raw = ietestcd_vals[idx].trim();
            let orres_raw = orres_vals.get(idx).map(|val| val.trim()).unwrap_or("");

            let needs_test = ietest_vals[idx].trim().is_empty();
            let needs_testcd = testcd_raw.is_empty()
                || is_yes_no_token(testcd_raw)
                || (!orres_raw.is_empty() && testcd_raw.eq_ignore_ascii_case(orres_raw));
            let needs_cat = iecat_vals[idx].trim().is_empty();

            if !needs_test && !needs_cat && !needs_testcd {
                continue;
            }

            for (label, values, category) in &candidates {
                let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
                if value.is_empty() {
                    continue;
                }

                if needs_test {
                    ietest_vals[idx] = label.clone();
                }
                if needs_testcd {
                    ietestcd_vals[idx] = sanitize_test_code(label);
                }
                if needs_cat {
                    iecat_vals[idx] = category.clone();
                }
                break;
            }
        }

        // Update DataFrame
        df.with_column(Series::new(ietest_col.as_str().into(), ietest_vals))?;
        df.with_column(Series::new(ietestcd_col.as_str().into(), ietestcd_vals))?;
        df.with_column(Series::new(iecat_col.as_str().into(), iecat_vals))?;

        Ok(())
    }
}

/// Helper to get column values or return empty strings.
fn get_column_values_or_empty(df: &DataFrame, col_name: &str) -> Vec<String> {
    if let Ok(series) = df.column(col_name) {
        (0..df.height())
            .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
            .collect()
    } else {
        vec![String::new(); df.height()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ie_test_field_rule_metadata() {
        let rule = IeTestFieldRule::new();
        let meta = rule.metadata();
        assert_eq!(meta.id, "IE_TEST_FIELD");
        assert_eq!(meta.category, RuleCategory::TestField);
        assert!(meta.target_variables.contains(&"IETEST".to_string()));
        assert!(meta.target_variables.contains(&"IETESTCD".to_string()));
        assert!(meta.target_variables.contains(&"IECAT".to_string()));
    }
}
