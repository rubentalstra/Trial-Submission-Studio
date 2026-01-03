//! Application of study metadata to DataFrames.

use std::collections::BTreeSet;

use polars::prelude::*;

use crate::error::Result;

use super::types::StudyMetadata;

/// Result of applying study metadata to a DataFrame.
#[derive(Debug, Clone, Default)]
pub struct AppliedStudyMetadata {
    /// Columns that had codelist values decoded.
    pub decoded_columns: BTreeSet<String>,
    /// New columns that were derived from decoded values.
    pub derived_columns: BTreeSet<String>,
    /// Codelists that were applied.
    pub applied_codelists: BTreeSet<String>,
}

impl AppliedStudyMetadata {
    /// Returns true if any changes were applied.
    pub fn has_changes(&self) -> bool {
        !self.decoded_columns.is_empty()
            || !self.derived_columns.is_empty()
            || !self.applied_codelists.is_empty()
    }
}

/// Applies study metadata (codelists) to a DataFrame.
///
/// For each column in the DataFrame:
/// - If the column has an associated codelist in the metadata, decode values
/// - If a decoded column doesn't exist, create a derived column
///
/// Returns the modified DataFrame and information about what was applied.
pub fn apply_study_metadata(
    df: DataFrame,
    metadata: &StudyMetadata,
) -> Result<(DataFrame, AppliedStudyMetadata)> {
    let mut result_df = df;
    let mut applied = AppliedStudyMetadata::default();

    // Get column names to process
    let column_names: Vec<String> = result_df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    for col_name in &column_names {
        // Look up the item definition
        let Some(item) = metadata.get_item(col_name) else {
            continue;
        };

        // Check if it has a format/codelist
        let Some(format_name) = &item.format_name else {
            continue;
        };

        // Look up the codelist
        let Some(codelist) = metadata.get_codelist(format_name) else {
            continue;
        };

        // Apply the codelist to decode values
        let column = result_df.column(col_name)?;

        let decoded_series = decode_column(column, codelist, col_name)?;

        // Determine where to put the decoded values
        // Common pattern: if column ends with CD, decoded goes to base name
        let decoded_col_name = if col_name.to_uppercase().ends_with("CD") {
            // SEXCD -> SEX
            col_name[..col_name.len() - 2].to_string()
        } else {
            // Otherwise, add _DECODED suffix
            format!("{col_name}_DECODED")
        };

        // Check if the decoded column already exists
        if result_df.column(&decoded_col_name).is_ok() {
            // Update existing column with decoded values (fill empty cells)
            result_df = fill_column_with_decoded(result_df, &decoded_col_name, decoded_series)?;
            applied.decoded_columns.insert(decoded_col_name);
        } else {
            // Create new derived column
            let new_series = decoded_series.with_name(decoded_col_name.clone().into());
            result_df.with_column(new_series)?;
            applied.derived_columns.insert(decoded_col_name);
        }

        applied.applied_codelists.insert(format_name.clone());
    }

    Ok((result_df, applied))
}

/// Decodes values in a column using a codelist.
fn decode_column(
    column: &Column,
    codelist: &super::types::StudyCodelist,
    name: &str,
) -> Result<Series> {
    let str_col = column.cast(&DataType::String)?;
    let str_chunked = str_col.str()?;

    let decoded: Vec<Option<String>> = str_chunked
        .iter()
        .map(|opt_val| {
            opt_val.and_then(|val| {
                let trimmed = val.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    codelist.lookup(trimmed).map(|s| s.to_string())
                }
            })
        })
        .collect();

    Ok(Series::new(name.into(), decoded))
}

/// Fills empty cells in an existing column with decoded values.
fn fill_column_with_decoded(
    mut df: DataFrame,
    col_name: &str,
    decoded_series: Series,
) -> Result<DataFrame> {
    let existing = df.column(col_name)?;
    let existing_str = existing.cast(&DataType::String)?;
    let existing_chunked = existing_str.str()?;
    let decoded_chunked = decoded_series.str()?;

    // Fill empty cells with decoded values
    let filled: Vec<Option<String>> = existing_chunked
        .iter()
        .zip(decoded_chunked.iter())
        .map(|(existing_val, decoded_val)| match existing_val {
            Some(v) if !v.trim().is_empty() => Some(v.to_string()),
            _ => decoded_val.map(|s| s.to_string()),
        })
        .collect();

    let filled_series = Series::new(col_name.into(), filled);
    df.with_column(filled_series)?;
    Ok(df)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::types::{SourceColumn, StudyCodelist};

    fn create_test_metadata() -> StudyMetadata {
        let mut meta = StudyMetadata::new();

        // Add items
        meta.add_item(SourceColumn::new("SEXCD", "Sex Code").with_format("SEX"));
        meta.add_item(SourceColumn::new("RACECD", "Race Code").with_format("RACE"));

        // Add codelists
        let mut sex_cl = StudyCodelist::new("SEX");
        sex_cl.insert("M", "Male");
        sex_cl.insert("F", "Female");
        meta.add_codelist(sex_cl);

        let mut race_cl = StudyCodelist::new("RACE");
        race_cl.insert("1", "Asian");
        race_cl.insert("2", "Black");
        race_cl.insert("3", "White");
        meta.add_codelist(race_cl);

        meta
    }

    #[test]
    fn test_apply_study_metadata_creates_derived() {
        let df = df! {
            "SEXCD" => &["M", "F", "M"],
            "RACECD" => &["1", "2", "3"],
        }
        .unwrap();

        let metadata = create_test_metadata();
        let (result_df, applied) = apply_study_metadata(df, &metadata).unwrap();

        // Should create SEX and RACE columns (derived from SEXCD and RACECD)
        assert!(result_df.column("SEX").is_ok());
        assert!(result_df.column("RACE").is_ok());

        // Check values
        let sex_col = result_df.column("SEX").unwrap();
        let sex_str = sex_col.str().unwrap();
        assert_eq!(sex_str.get(0), Some("Male"));
        assert_eq!(sex_str.get(1), Some("Female"));

        // Check applied info
        assert!(applied.derived_columns.contains("SEX"));
        assert!(applied.derived_columns.contains("RACE"));
        assert!(applied.applied_codelists.contains("SEX"));
        assert!(applied.applied_codelists.contains("RACE"));
    }

    #[test]
    fn test_apply_study_metadata_fills_existing() {
        let df = df! {
            "SEXCD" => &["M", "F", "M"],
            "SEX" => &["", "Female", ""],  // Existing column with some empty values
        }
        .unwrap();

        let metadata = create_test_metadata();
        let (result_df, applied) = apply_study_metadata(df, &metadata).unwrap();

        // Should fill empty SEX values
        let sex_col = result_df.column("SEX").unwrap();
        let sex_str = sex_col.str().unwrap();
        assert_eq!(sex_str.get(0), Some("Male")); // Filled
        assert_eq!(sex_str.get(1), Some("Female")); // Kept
        assert_eq!(sex_str.get(2), Some("Male")); // Filled

        assert!(applied.decoded_columns.contains("SEX"));
    }

    #[test]
    fn test_apply_study_metadata_no_match() {
        let df = df! {
            "UNKNOWN" => &["A", "B", "C"],
        }
        .unwrap();

        let metadata = create_test_metadata();
        let (result_df, applied) = apply_study_metadata(df, &metadata).unwrap();

        // No changes should be made
        assert!(!applied.has_changes());
        assert_eq!(result_df.width(), 1);
    }
}
