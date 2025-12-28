//! DA (Drug Accountability) domain preprocessing.
//!
//! Handles inference of DA test fields from column patterns.

use anyhow::Result;
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};

use sdtm_ingest::any_to_string;
use sdtm_model::CaseInsensitiveLookup;

use crate::ct_utils::resolve_ct_value_from_hint;
use crate::data_utils::{sanitize_test_code, table_column_values, table_label};

use super::common::PreprocessContext;
use super::rule_table::{PreprocessRule, RuleCategory, RuleMetadata};

/// Type alias for DA ORRES candidate tuple:
/// (test_name, test_code, unit, values)
type DaOrresCandidate = (Option<String>, Option<String>, Option<String>, Vec<String>);

/// Preprocess DA domain DataFrame.
///
/// # SDTMIG Reference
///
/// Drug Accountability domain follows Findings class structure (Chapter 6.3.3).
/// DATEST, DATESTCD, DAORRES, DAORRESU are populated from column patterns.
pub fn preprocess_da(ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
    let rule = DaTestFieldRule::new();
    rule.apply(ctx, df)
}

/// Rule for inferring DA test and result fields.
pub struct DaTestFieldRule {
    metadata: RuleMetadata,
}

impl DaTestFieldRule {
    pub fn new() -> Self {
        Self {
            metadata: RuleMetadata::new(
                "DA_TEST_FIELD",
                RuleCategory::TestField,
                "Infer DATEST, DATESTCD, DAORRES, DAORRESU from column patterns",
            )
            .with_targets(&["DATEST", "DATESTCD", "DAORRES", "DAORRESU", "DASTRESU"]),
        }
    }
}

impl Default for DaTestFieldRule {
    fn default() -> Self {
        Self::new()
    }
}

impl PreprocessRule for DaTestFieldRule {
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

        // Resolve CT codelists
        let ctdatest = ctx.ctx.resolve_ct(ctx.domain, "DATEST");
        let ctdatestcd = ctx.ctx.resolve_ct(ctx.domain, "DATESTCD");
        let ct_units = ctx.ctx.resolve_ct(ctx.domain, "DAORRESU");
        let datest_extensible = ctdatest.map(|ct| ct.extensible).unwrap_or(false);
        let datestcd_extensible = ctdatestcd.map(|ct| ct.extensible).unwrap_or(false);

        // Build candidate headers
        let mut candidates: Vec<DaOrresCandidate> = Vec::new();
        let mut candidate_headers: Vec<String> = Vec::new();
        let require_explicit = ctx.ctx.options.require_explicit_mapping;

        // First check for explicit mapping source
        let has_explicit_mapping = ctx.mapping_source("DAORRES").is_some();

        if let Some(preferred) = ctx.mapping_source("DAORRES") {
            candidate_headers.push(preferred);
        } else if !require_explicit {
            // Only search for implicit candidates if explicit mapping not required
            for header in &ctx.table.headers {
                if header.to_uppercase().ends_with("_DAORRES") {
                    candidate_headers.push(header.clone());
                }
            }
        }

        // Build set of standard variables
        let standard_vars = ctx.standard_variable_set();

        // Find DA-prefixed columns that aren't standard variables
        // Skip this if require_explicit_mapping and no explicit mapping exists
        if !require_explicit || has_explicit_mapping {
            for header in &ctx.table.headers {
                let upper = header.to_uppercase();
                if !upper.starts_with("DA") {
                    continue;
                }
                if upper.ends_with("CD") {
                    continue;
                }
                if standard_vars.contains(&upper) {
                    continue;
                }
                candidate_headers.push(header.clone());
            }
        }

        candidate_headers.sort();
        candidate_headers.dedup();

        // Process each candidate header
        for header in candidate_headers {
            let upper = header.to_uppercase();
            let prefix = upper.strip_suffix("_DAORRES").unwrap_or(&upper);

            if let Some(values) = table_column_values(ctx.table, &header) {
                let label = table_label(ctx.table, &header);
                let hint = label.clone().unwrap_or_else(|| prefix.to_string());

                // Try to resolve test code from CT
                let mut test_code = ctdatestcd
                    .and_then(|ct| resolve_ct_value_from_hint(ct, prefix))
                    .or_else(|| {
                        label.as_deref().and_then(|text| {
                            ctdatestcd.and_then(|ct| resolve_ct_value_from_hint(ct, text))
                        })
                    })
                    .or_else(|| ctdatestcd.and_then(|ct| resolve_ct_value_from_hint(ct, &hint)));

                // Try to resolve test name from CT
                let mut test_name = ctdatest
                    .and_then(|ct| resolve_ct_value_from_hint(ct, prefix))
                    .or_else(|| {
                        label.as_deref().and_then(|text| {
                            ctdatest.and_then(|ct| resolve_ct_value_from_hint(ct, text))
                        })
                    })
                    .or_else(|| ctdatest.and_then(|ct| resolve_ct_value_from_hint(ct, &hint)));

                // Try to get test name from preferred term if we have test code
                if test_name.is_none()
                    && let (Some(ct), Some(code)) = (ctdatestcd, test_code.as_ref())
                {
                    test_name = crate::ct_utils::preferred_term_for(ct, code);
                }

                // Use extensible fallbacks
                if test_name.is_none() && datest_extensible {
                    test_name = label.clone().or_else(|| Some(prefix.to_string()));
                }
                if test_code.is_none() && datestcd_extensible {
                    let raw = label.clone().unwrap_or_else(|| prefix.to_string());
                    test_code = Some(sanitize_test_code(&raw));
                }

                // Try to resolve unit from CT
                let unit = ct_units
                    .and_then(|ct| resolve_ct_value_from_hint(ct, &hint))
                    .or_else(|| {
                        label.as_deref().and_then(|text| {
                            ct_units.and_then(|ct| resolve_ct_value_from_hint(ct, text))
                        })
                    })
                    .or_else(|| ct_units.and_then(|ct| resolve_ct_value_from_hint(ct, prefix)));

                candidates.push((test_name, test_code, unit, values));
            }
        }

        // Fill missing values from candidates
        if !candidates.is_empty() {
            let daorres_col = column_name("DAORRES");
            let datest_col = column_name("DATEST");
            let datestcd_col = column_name("DATESTCD");
            let daorresu_col = column_name("DAORRESU");
            let dastresu_col = column_name("DASTRESU");

            // Get existing values
            let mut daorres_vals = get_column_values_or_empty(df, &daorres_col);
            let mut datest_vals = get_column_values_or_empty(df, &datest_col);
            let mut datestcd_vals = get_column_values_or_empty(df, &datestcd_col);
            let mut daorresu_vals = get_column_values_or_empty(df, &daorresu_col);
            let mut dastresu_vals = get_column_values_or_empty(df, &dastresu_col);

            // Fill from candidates
            for idx in 0..df.height() {
                let needs_orres = daorres_vals[idx].trim().is_empty();
                let needs_test = datest_vals[idx].trim().is_empty();
                let needs_testcd = datestcd_vals[idx].trim().is_empty();
                let needs_orresu = daorresu_vals[idx].trim().is_empty();
                let needs_stresu = dastresu_vals[idx].trim().is_empty();

                if !needs_orres && !needs_test && !needs_testcd && !needs_orresu && !needs_stresu {
                    continue;
                }

                for (test_name, test_code, unit, values) in &candidates {
                    let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
                    if value.is_empty() {
                        continue;
                    }

                    // Skip if we need test fields but don't have them
                    if needs_test && test_name.is_none() {
                        continue;
                    }
                    if needs_testcd && test_code.is_none() {
                        continue;
                    }

                    if needs_orres {
                        daorres_vals[idx] = value.to_string();
                    }
                    if needs_test && let Some(name) = test_name {
                        datest_vals[idx] = name.clone();
                    }
                    if needs_testcd && let Some(code) = test_code {
                        datestcd_vals[idx] = code.clone();
                    }
                    if needs_orresu && let Some(unit) = unit {
                        daorresu_vals[idx] = unit.clone();
                    }
                    if needs_stresu && let Some(unit) = unit {
                        dastresu_vals[idx] = unit.clone();
                    }
                    break;
                }
            }

            // Update DataFrame
            df.with_column(Series::new(daorres_col.as_str().into(), daorres_vals))?;
            df.with_column(Series::new(datest_col.as_str().into(), datest_vals))?;
            df.with_column(Series::new(datestcd_col.as_str().into(), datestcd_vals))?;
            df.with_column(Series::new(daorresu_col.as_str().into(), daorresu_vals))?;
            df.with_column(Series::new(dastresu_col.as_str().into(), dastresu_vals))?;
        }

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
    fn test_da_test_field_rule_metadata() {
        let rule = DaTestFieldRule::new();
        let meta = rule.metadata();
        assert_eq!(meta.id, "DA_TEST_FIELD");
        assert_eq!(meta.category, RuleCategory::TestField);
        assert!(meta.target_variables.contains(&"DATEST".to_string()));
        assert!(meta.target_variables.contains(&"DATESTCD".to_string()));
        assert!(meta.target_variables.contains(&"DAORRES".to_string()));
    }
}
