//! Standalone SDTM transformation functions.
//!
//! This module provides pure, standalone functions for SDTM transformations
//! that can be used independently of the pipeline context. These are designed
//! for use in the GUI where transformations are applied incrementally.
//!
//! # SDTMIG v3.4 Reference
//!
//! - Section 4.1.2: USUBJID construction (STUDYID-SUBJID format)
//! - Section 4.1.5: --SEQ variable assignment per subject
//! - Chapter 10: Controlled Terminology conformance

use anyhow::Result;
use polars::prelude::*;
use sdtm_model::CaseInsensitiveSet;
use sdtm_model::ct::Codelist;
use sdtm_transform::data_utils::strip_all_quotes;

use crate::ct_utils::normalize_ct_value;
use crate::pipeline_context::CtMatchingMode;

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
///
/// # Example
///
/// ```ignore
/// let modified = apply_usubjid_prefix(&mut df, "CDISC01", "USUBJID", None)?;
/// println!("Modified {} USUBJID values", modified);
/// ```
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

/// Normalize a single column's values against Controlled Terminology.
///
/// This function normalizes values in a column to their preferred CT terms.
/// Values are matched against the codelist's terms and synonyms.
///
/// # Arguments
///
/// * `df` - DataFrame to modify (in place)
/// * `column_name` - Name of the column to normalize
/// * `codelist` - The controlled terminology codelist to use
/// * `matching_mode` - Strict or lenient matching
///
/// # Returns
///
/// Number of values that were normalized.
pub fn normalize_ct_column(
    df: &mut DataFrame,
    column_name: &str,
    codelist: &Codelist,
    matching_mode: CtMatchingMode,
) -> Result<usize> {
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());

    // Find the actual column name (case-insensitive)
    let Some(col_name) = column_lookup.get(column_name) else {
        return Ok(0);
    };

    // Check if column exists and is string type
    let Ok(column) = df.column(col_name) else {
        return Ok(0);
    };

    let Ok(str_ca) = column.str() else {
        return Ok(0); // Not a string column
    };

    let codelist_clone = codelist.clone();
    let mut normalized_count = 0;

    // Count how many values will change
    for opt_val in str_ca.into_iter() {
        if let Some(val) = opt_val {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                let normalized = normalize_ct_value(&codelist_clone, trimmed, matching_mode);
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
                            matching_mode,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_usubjid_prefix_adds_prefix() {
        let mut df = DataFrame::new(vec![
            Series::new("USUBJID".into(), vec!["001", "002", "003"]).into(),
        ])
        .unwrap();

        let modified = apply_usubjid_prefix(&mut df, "STUDY01", "USUBJID", None).unwrap();

        assert_eq!(modified, 3);
        let col = df.column("USUBJID").unwrap().str().unwrap();
        assert_eq!(col.get(0), Some("STUDY01-001"));
        assert_eq!(col.get(1), Some("STUDY01-002"));
        assert_eq!(col.get(2), Some("STUDY01-003"));
    }

    #[test]
    fn test_apply_usubjid_prefix_skips_existing() {
        let mut df = DataFrame::new(vec![
            Series::new("USUBJID".into(), vec!["STUDY01-001", "002"]).into(),
        ])
        .unwrap();

        let modified = apply_usubjid_prefix(&mut df, "STUDY01", "USUBJID", None).unwrap();

        assert_eq!(modified, 1); // Only second row modified
        let col = df.column("USUBJID").unwrap().str().unwrap();
        assert_eq!(col.get(0), Some("STUDY01-001")); // Unchanged
        assert_eq!(col.get(1), Some("STUDY01-002")); // Prefixed
    }

    #[test]
    fn test_assign_sequence_numbers() {
        let mut df = DataFrame::new(vec![
            Series::new("USUBJID".into(), vec!["A", "A", "B", "A", "B"]).into(),
        ])
        .unwrap();

        let count = assign_sequence_numbers(&mut df, "AESEQ", "USUBJID").unwrap();

        assert_eq!(count, 5);
        let seq = df.column("AESEQ").unwrap().f64().unwrap();
        // A gets 1, 2, 3; B gets 1, 2
        assert_eq!(seq.get(0), Some(1.0)); // A-1
        assert_eq!(seq.get(1), Some(2.0)); // A-2
        assert_eq!(seq.get(2), Some(1.0)); // B-1
        assert_eq!(seq.get(3), Some(3.0)); // A-3
        assert_eq!(seq.get(4), Some(2.0)); // B-2
    }
}
