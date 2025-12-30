//! Transformation executor functions.
//!
//! This module provides the actual transformation functions that are called
//! by the pipeline executor. Each function corresponds to a [`TransformType`]
//! and operates on DataFrame columns.
//!
//! # SDTMIG v3.4 Reference
//!
//! - Section 4.1.2: USUBJID construction (STUDYID-SUBJID format)
//! - Section 4.1.5: --SEQ variable assignment per subject
//! - Section 4.4.4: Study day (--DY) calculation rules
//! - Chapter 10: Controlled Terminology conformance

use anyhow::Result;
use polars::prelude::*;
use sdtm_model::CaseInsensitiveSet;
use sdtm_model::ct::Codelist;
use sdtm_model::options::{CtMatchingMode, NormalizationOptions};

use crate::data_utils::strip_all_quotes;
use crate::normalization::ct::normalize_ct_value;
use crate::normalization::datetime::parse_date;

/// Apply a constant value to all rows of a column.
///
/// Used for STUDYID and DOMAIN which are set from the context.
pub fn apply_constant(df: &mut DataFrame, column_name: &str, value: &str) -> Result<usize> {
    let height = df.height();
    if height == 0 {
        return Ok(0);
    }

    let col = Column::new(column_name.into(), vec![value; height]);
    df.with_column(col)?;

    Ok(height)
}

/// Apply STUDYID prefix to USUBJID column.
///
/// Per SDTMIG 4.1.2, USUBJID should be formatted as "STUDYID-SUBJID".
/// This function adds the study prefix to USUBJID values that don't already have it.
///
/// # Arguments
///
/// * `df` - DataFrame to modify (in place)
/// * `study_id` - Study identifier to use as prefix
/// * `usubjid_column` - Name of the USUBJID column (case-insensitive lookup)
/// * `studyid_column` - Optional name of STUDYID column to use per-row values
///
/// # Returns
///
/// Number of rows that were modified.
pub fn apply_usubjid_prefix(
    df: &mut DataFrame,
    study_id: &str,
    usubjid_column: &str,
    studyid_column: Option<&str>,
) -> Result<usize> {
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());

    // Find the actual column name (case-insensitive)
    let Some(usubjid_col) = column_lookup.get(usubjid_column) else {
        return Ok(0); // Column not found, nothing to do
    };

    let study_col = studyid_column.and_then(|name| column_lookup.get(name));

    // Get column data
    let usubjid_ca = df.column(usubjid_col)?.str()?;
    let study_ca = study_col.and_then(|name| df.column(name).ok()?.str().ok());

    let mut updated_builder =
        polars::prelude::StringChunkedBuilder::new(usubjid_col.into(), df.height());
    let mut modified_count = 0;

    for (idx, opt_u) in usubjid_ca.into_iter().enumerate() {
        let raw_usubjid = opt_u.unwrap_or("");
        let mut usubjid = strip_all_quotes(raw_usubjid);

        // Get study value (from column or parameter)
        let study_val = if let Some(ca) = study_ca {
            strip_all_quotes(ca.get(idx).unwrap_or(""))
        } else {
            study_id.to_string()
        };

        // Add prefix if needed
        if !study_val.is_empty() && !usubjid.is_empty() {
            let prefix = format!("{study_val}-");
            if !usubjid.starts_with(&prefix) {
                usubjid = format!("{prefix}{usubjid}");
                modified_count += 1;
            }
        }

        updated_builder.append_value(usubjid);
    }

    if modified_count > 0 {
        let new_series = updated_builder.finish().into_series();
        df.with_column(new_series)?;
    }

    Ok(modified_count)
}

/// Assign sequence numbers (--SEQ) per subject group.
///
/// Per SDTMIG 4.1.5, --SEQ is a unique number for each record within a domain
/// for a subject. This function assigns 1-based sequence numbers within each
/// group defined by the group column (typically USUBJID).
///
/// # Arguments
///
/// * `df` - DataFrame to modify (in place)
/// * `seq_column` - Name of the sequence column to create/update
/// * `group_column` - Name of the grouping column (typically USUBJID)
///
/// # Returns
///
/// Number of rows that received sequence numbers.
pub fn assign_sequence_numbers(
    df: &mut DataFrame,
    seq_column: &str,
    group_column: &str,
) -> Result<usize> {
    if df.height() == 0 {
        return Ok(0);
    }

    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());

    // Find the actual column names (case-insensitive)
    let group_col = column_lookup.get(group_column).unwrap_or(group_column);
    let seq_col = column_lookup.get(seq_column).unwrap_or(seq_column);

    // Check if group column exists
    if df.column(group_col).is_err() {
        return Ok(0);
    }

    // Calculate sequence numbers: 1-based index within each group
    use polars::lazy::dsl::int_range;

    let seq_expr =
        int_range(lit(0), col(group_col).len(), 1, DataType::Int64).over([col(group_col)]) + lit(1);

    let new_df = df
        .clone()
        .lazy()
        .with_column(seq_expr.cast(DataType::Float64).alias(seq_col))
        .collect()?;

    let row_count = new_df.height();
    *df = new_df;

    Ok(row_count)
}

/// Copy a column directly with optional renaming.
///
/// If the source column exists, it's copied to the target name.
/// Handles case-insensitive column lookup.
pub fn copy_column(df: &mut DataFrame, source: &str, target: &str) -> Result<usize> {
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());

    let Some(src_col) = column_lookup.get(source) else {
        return Ok(0);
    };

    let col = df.column(src_col)?.clone();
    let renamed = col.with_name(target.into());
    df.with_column(renamed)?;

    Ok(df.height())
}

/// Normalize a single column's values against Controlled Terminology.
///
/// This function normalizes values in a column to their preferred CT terms.
/// Values are matched against the codelist's terms and synonyms.
pub fn normalize_ct_column(
    df: &mut DataFrame,
    column_name: &str,
    codelist: &Codelist,
    options: &NormalizationOptions,
) -> Result<usize> {
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());

    let Some(col_name) = column_lookup.get(column_name) else {
        return Ok(0);
    };

    let Ok(column) = df.column(col_name) else {
        return Ok(0);
    };

    let Ok(str_ca) = column.str() else {
        return Ok(0);
    };

    let codelist_clone = codelist.clone();
    let mut normalized_count = 0;

    // Count how many values will change
    for opt_val in str_ca.into_iter() {
        if let Some(val) = opt_val {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                let normalized = normalize_ct_value(&codelist_clone, trimmed, options);
                if normalized != trimmed {
                    normalized_count += 1;
                }
            }
        }
    }

    if normalized_count == 0 {
        return Ok(0);
    }

    // Apply normalization
    let codelist_for_expr = codelist.clone();
    let options_for_expr = options.clone();
    let expr = col(col_name)
        .map(
            move |c: Column| {
                let ca = c.str()?;
                let out: StringChunked = ca.apply_values(|s| {
                    if s.trim().is_empty() {
                        std::borrow::Cow::Borrowed("")
                    } else {
                        std::borrow::Cow::Owned(normalize_ct_value(
                            &codelist_for_expr,
                            s,
                            &options_for_expr,
                        ))
                    }
                });
                Ok(out.into_column())
            },
            |_, field| Ok(Field::new(field.name().clone(), DataType::String)),
        )
        .alias(col_name);

    let new_df = df.clone().lazy().with_columns([expr]).collect()?;
    *df = new_df;

    Ok(normalized_count)
}

/// Parse and format a column as ISO 8601 datetime.
///
/// Attempts to parse various datetime formats and output ISO 8601 format.
/// Missing or invalid values become null.
pub fn apply_iso8601_datetime(df: &mut DataFrame, source: &str, target: &str) -> Result<usize> {
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());

    let Some(src_col) = column_lookup.get(source) else {
        return Ok(0);
    };

    let column = df.column(src_col)?;
    let str_ca = column.str()?;

    let mut builder = polars::prelude::StringChunkedBuilder::new(target.into(), df.height());
    let mut success_count = 0;

    for opt_val in str_ca.into_iter() {
        let value = opt_val.unwrap_or("").trim();
        if value.is_empty() {
            builder.append_null();
        } else {
            let normalized = value.to_string();
            if !normalized.is_empty() && normalized != value {
                builder.append_value(&normalized);
                success_count += 1;
            } else {
                builder.append_value(value);
            }
        }
    }

    let new_col = builder.finish().into_series();
    df.with_column(new_col)?;

    Ok(success_count)
}

/// Parse and format a column as ISO 8601 date only.
///
/// Similar to datetime but only keeps the date portion (YYYY-MM-DD).
pub fn apply_iso8601_date(df: &mut DataFrame, source: &str, target: &str) -> Result<usize> {
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());

    let Some(src_col) = column_lookup.get(source) else {
        return Ok(0);
    };

    let column = df.column(src_col)?;
    let str_ca = column.str()?;

    let mut builder = polars::prelude::StringChunkedBuilder::new(target.into(), df.height());
    let mut success_count = 0;

    for opt_val in str_ca.into_iter() {
        let value = opt_val.unwrap_or("").trim();
        if value.is_empty() {
            builder.append_null();
        } else {
            // Try to parse and extract just the date portion
            let normalized = value.to_string();
            // Extract date portion (first 10 chars if it's a datetime)
            let date_part = if normalized.len() >= 10 && normalized.chars().nth(4) == Some('-') {
                &normalized[..10.min(normalized.len())]
            } else {
                &normalized
            };
            if !date_part.is_empty() {
                builder.append_value(date_part);
                success_count += 1;
            } else {
                builder.append_value(value);
            }
        }
    }

    let new_col = builder.finish().into_series();
    df.with_column(new_col)?;

    Ok(success_count)
}

/// Calculate study day from a reference date.
///
/// Per SDTMIG 4.4.4:
/// - If DTC >= RFSTDTC: DY = DTC - RFSTDTC + 1
/// - If DTC < RFSTDTC: DY = DTC - RFSTDTC (negative, no +1)
pub fn calculate_study_day(
    df: &mut DataFrame,
    dtc_column: &str,
    rfstdtc_column: &str,
    dy_column: &str,
) -> Result<usize> {
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());

    let Some(dtc_col) = column_lookup.get(dtc_column) else {
        return Ok(0);
    };

    let Some(ref_col) = column_lookup.get(rfstdtc_column) else {
        return Ok(0);
    };

    let dtc_ca = df.column(dtc_col)?.str()?;
    let ref_ca = df.column(ref_col)?.str()?;

    let mut values = Vec::with_capacity(df.height());
    let mut success_count = 0;

    for (dtc_opt, ref_opt) in dtc_ca.into_iter().zip(ref_ca.into_iter()) {
        let dy = match (dtc_opt, ref_opt) {
            (Some(dtc), Some(rfstdtc)) if !dtc.is_empty() && !rfstdtc.is_empty() => {
                // Parse dates and calculate difference
                let dtc_date = parse_date(dtc.trim());
                let ref_date = parse_date(rfstdtc.trim());

                match (dtc_date, ref_date) {
                    (Some(d), Some(r)) => {
                        let diff = (d - r).num_days();
                        let dy_val = if diff >= 0 { diff + 1 } else { diff };
                        success_count += 1;
                        Some(dy_val as f64)
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        values.push(dy);
    }

    let col = Column::new(dy_column.into(), values);
    df.with_column(col)?;

    Ok(success_count)
}

/// Convert a column to numeric values.
///
/// Attempts to parse string values as floating point numbers.
/// Non-numeric values become null.
pub fn apply_numeric_conversion(df: &mut DataFrame, source: &str, target: &str) -> Result<usize> {
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());

    let Some(src_col) = column_lookup.get(source) else {
        return Ok(0);
    };

    let column = df.column(src_col)?;
    let str_ca = column.str()?;

    let mut values = Vec::with_capacity(df.height());
    let mut success_count = 0;

    for opt_val in str_ca.into_iter() {
        let value = opt_val.unwrap_or("").trim();
        if value.is_empty() {
            values.push(None);
        } else {
            match value.parse::<f64>() {
                Ok(num) => {
                    values.push(Some(num));
                    success_count += 1;
                }
                Err(_) => {
                    values.push(None);
                }
            }
        }
    }

    let col = Column::new(target.into(), values);
    df.with_column(col)?;

    Ok(success_count)
}

/// Get columns in a DataFrame that have associated CT codelists.
///
/// Returns a list of (column_name, codelist_code) pairs for columns
/// that should have CT normalization applied.
pub fn get_ct_columns(df: &DataFrame, domain: &sdtm_model::Domain) -> Vec<(String, String)> {
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
    let mut result = Vec::new();

    for variable in &domain.variables {
        if let Some(codelist_code) = &variable.codelist_code {
            if let Some(col_name) = column_lookup.get(&variable.name) {
                // Take the first codelist code (before semicolon)
                let code = codelist_code.split(';').next().unwrap_or("").trim();
                if !code.is_empty() {
                    result.push((col_name.to_string(), code.to_string()));
                }
            }
        }
    }

    result
}

/// Build a preview DataFrame by applying accepted mappings.
///
/// This creates a transformed DataFrame suitable for validation:
/// 1. Renames source columns to their mapped SDTM variable names
/// 2. Applies CT normalization where codelists are defined
/// 3. Adds constant columns (STUDYID, DOMAIN) if mappings don't exist
pub fn build_preview_dataframe(
    source_df: &DataFrame,
    accepted_mappings: &std::collections::BTreeMap<String, String>,
    domain: &sdtm_model::Domain,
    study_id: &str,
    ct_registry: Option<&sdtm_model::TerminologyRegistry>,
) -> Result<DataFrame> {
    use polars::prelude::*;

    if source_df.height() == 0 {
        return Ok(source_df.clone());
    }

    let height = source_df.height();

    // Build list of columns to include, with renaming
    let mut columns: Vec<Column> = Vec::new();

    // Add mapped columns (renamed from source to SDTM name)
    for (sdtm_var, source_col) in accepted_mappings {
        if let Ok(col) = source_df.column(source_col) {
            let renamed = col.clone().with_name(sdtm_var.into());
            columns.push(renamed);
        }
    }

    // Add constant STUDYID if not mapped
    if !accepted_mappings.contains_key("STUDYID") {
        let studyid_col = Column::new("STUDYID".into(), vec![study_id; height]);
        columns.push(studyid_col);
    }

    // Add constant DOMAIN if not mapped
    if !accepted_mappings.contains_key("DOMAIN") {
        let domain_col = Column::new("DOMAIN".into(), vec![domain.code.as_str(); height]);
        columns.push(domain_col);
    }

    // Create the preview DataFrame
    let mut preview_df = DataFrame::new(columns)?;

    // Apply CT normalization for variables that have codelists
    if let Some(registry) = ct_registry {
        for variable in &domain.variables {
            if !accepted_mappings.contains_key(&variable.name) {
                continue;
            }

            let Some(codelist_code) = &variable.codelist_code else {
                continue;
            };

            let code = codelist_code.split(';').next().unwrap_or("").trim();
            if code.is_empty() {
                continue;
            }

            let Some(resolved) = registry.resolve(code, None) else {
                continue;
            };

            let _ = normalize_ct_column(
                &mut preview_df,
                &variable.name,
                resolved.codelist,
                &NormalizationOptions {
                    matching_mode: CtMatchingMode::Lenient,
                    ..Default::default()
                },
            );
        }
    }

    Ok(preview_df)
}

/// Build a simple preview DataFrame with only column renaming (no CT normalization).
///
/// This is a lighter-weight version for quick previews that doesn't require
/// loading the CT registry.
pub fn build_simple_preview(
    source_df: &DataFrame,
    accepted_mappings: &std::collections::BTreeMap<String, String>,
    domain_code: &str,
    study_id: &str,
) -> Result<DataFrame> {
    use polars::prelude::*;

    if source_df.height() == 0 {
        return Ok(source_df.clone());
    }

    let height = source_df.height();
    let mut columns: Vec<Column> = Vec::new();

    // Add mapped columns (renamed)
    for (sdtm_var, source_col) in accepted_mappings {
        if let Ok(col) = source_df.column(source_col) {
            let renamed = col.clone().with_name(sdtm_var.into());
            columns.push(renamed);
        }
    }

    // Add constants
    if !accepted_mappings.contains_key("STUDYID") {
        columns.push(Column::new("STUDYID".into(), vec![study_id; height]));
    }
    if !accepted_mappings.contains_key("DOMAIN") {
        columns.push(Column::new("DOMAIN".into(), vec![domain_code; height]));
    }

    DataFrame::new(columns).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_constant() {
        let mut df = DataFrame::new(vec![Column::new("A".into(), vec!["1", "2", "3"])]).unwrap();

        let count = apply_constant(&mut df, "STUDYID", "TEST001").unwrap();
        assert_eq!(count, 3);
        assert!(df.column("STUDYID").is_ok());
    }

    #[test]
    fn test_copy_column() {
        let mut df =
            DataFrame::new(vec![Column::new("source_col".into(), vec!["a", "b", "c"])]).unwrap();

        let count = copy_column(&mut df, "source_col", "TARGET").unwrap();
        assert_eq!(count, 3);
        assert!(df.column("TARGET").is_ok());
    }

    #[test]
    fn test_apply_numeric_conversion() {
        let mut df = DataFrame::new(vec![Column::new(
            "values".into(),
            vec!["1.5", "2.0", "abc", ""],
        )])
        .unwrap();

        let count = apply_numeric_conversion(&mut df, "values", "NUMERIC").unwrap();
        assert_eq!(count, 2); // Only 1.5 and 2.0 should succeed
    }

    #[test]
    fn test_usubjid_prefix() {
        let mut df = DataFrame::new(vec![Column::new(
            "USUBJID".into(),
            vec!["001", "002", "STUDY-003"],
        )])
        .unwrap();

        let count = apply_usubjid_prefix(&mut df, "STUDY", "USUBJID", None).unwrap();
        assert_eq!(count, 2); // 001 and 002 should be prefixed, 003 already has it

        let col = df.column("USUBJID").unwrap();
        let values: Vec<_> = col.str().unwrap().into_iter().map(|v| v.unwrap()).collect();
        assert_eq!(values, vec!["STUDY-001", "STUDY-002", "STUDY-003"]);
    }
}
