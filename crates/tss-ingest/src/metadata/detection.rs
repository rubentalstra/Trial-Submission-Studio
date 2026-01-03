//! Dynamic content-based schema detection for metadata files.
//!
//! Instead of matching column names (which can vary between EDC systems),
//! this module analyzes column content to determine their roles using
//! purely statistical patterns - NO hardcoded keywords.

use polars::prelude::*;

use crate::error::{IngestError, Result};

/// Represents a detected column role with its index and confidence.
#[derive(Debug, Clone)]
pub struct ColumnRole {
    /// Column index in the DataFrame.
    pub index: usize,
    /// Column name.
    pub name: String,
    /// Confidence score (0.0 - 1.0).
    #[allow(dead_code)]
    pub confidence: f64,
}

/// Detected schema for Items.csv metadata file.
#[derive(Debug, Clone)]
pub struct ItemsSchema {
    /// ID column: contains unique identifiers.
    pub id: ColumnRole,
    /// Label column: contains descriptive text.
    pub label: ColumnRole,
    /// Data type column: contains type values.
    pub data_type: Option<ColumnRole>,
    /// Mandatory column: contains boolean/yes-no values.
    pub mandatory: Option<ColumnRole>,
    /// Format name column: references to codelists.
    pub format_name: Option<ColumnRole>,
    /// Content length column: contains numeric lengths.
    pub content_length: Option<ColumnRole>,
}

/// Detected schema for CodeLists.csv metadata file.
#[derive(Debug, Clone)]
pub struct CodelistSchema {
    /// Format name column: groups the codelist entries.
    pub format_name: ColumnRole,
    /// Code value column: the coded values.
    pub code_value: ColumnRole,
    /// Code text column: the decoded text.
    pub code_text: ColumnRole,
}

/// Column analysis scores - purely statistical, no hardcoded keywords.
#[derive(Debug, Clone)]
struct ColumnScores {
    index: usize,
    name: String,
    /// Ratio of unique values to total rows (0.0-1.0)
    uniqueness: f64,
    /// Average string length of non-empty values
    avg_length: f64,
    /// Ratio of numeric-parseable values (0.0-1.0)
    numeric_ratio: f64,
    /// Number of unique values
    cardinality: usize,
    /// Ratio of empty/null values (0.0-1.0)
    empty_ratio: f64,
    /// Whether all unique values are very short (<=10 chars)
    all_short_values: bool,
}

/// Analyzes a column and returns statistical scores.
fn analyze_column(col: &Column, idx: usize) -> Option<ColumnScores> {
    let name = col.name().to_string();
    let series = col.as_materialized_series();

    // Try to cast to string
    let str_col = series.cast(&DataType::String).ok()?;
    let str_chunked = str_col.str().ok()?;

    let total = str_chunked.len();
    if total == 0 {
        return None;
    }

    // Calculate statistics
    let unique_count = str_chunked.n_unique().unwrap_or(0);
    let uniqueness = unique_count as f64 / total as f64;

    let mut total_len = 0usize;
    let mut non_null_count = 0usize;
    let mut numeric_count = 0usize;
    let mut empty_count = 0usize;
    let mut all_short = true;

    for opt_val in str_chunked.iter() {
        match opt_val {
            Some(val) => {
                let trimmed = val.trim();
                if trimmed.is_empty() {
                    empty_count += 1;
                } else {
                    total_len += trimmed.len();
                    non_null_count += 1;

                    if trimmed.len() > 10 {
                        all_short = false;
                    }

                    // Check if parseable as number
                    if trimmed.parse::<f64>().is_ok() {
                        numeric_count += 1;
                    }
                }
            }
            None => empty_count += 1,
        }
    }

    let avg_length = if non_null_count > 0 {
        total_len as f64 / non_null_count as f64
    } else {
        0.0
    };

    let numeric_ratio = if non_null_count > 0 {
        numeric_count as f64 / non_null_count as f64
    } else {
        0.0
    };

    let empty_ratio = empty_count as f64 / total as f64;

    Some(ColumnScores {
        index: idx,
        name,
        uniqueness,
        avg_length,
        numeric_ratio,
        cardinality: unique_count,
        empty_ratio,
        all_short_values: all_short && non_null_count > 0,
    })
}

/// Detects the schema for an Items.csv file using purely statistical patterns.
///
/// Detection strategy (no hardcoded keywords):
/// - **ID column**: Highest uniqueness ratio (each row has unique identifier)
/// - **Label column**: Longest average length (descriptive text)
/// - **DataType column**: Very low cardinality (3-8 unique values), short values
/// - **Mandatory column**: Binary (2-3 unique values), very short
/// - **FormatName column**: Medium cardinality, many empty values, short
/// - **ContentLength column**: All numeric, short values
pub fn detect_items_schema(df: &DataFrame, path: &std::path::Path) -> Result<ItemsSchema> {
    let columns = df.get_columns();
    if columns.len() < 2 {
        return Err(IngestError::SchemaDetection {
            file_type: "Items".to_string(),
            path: path.to_path_buf(),
            reason: "need at least 2 columns".to_string(),
        });
    }

    // Analyze all columns
    let scores: Vec<ColumnScores> = columns
        .iter()
        .enumerate()
        .filter_map(|(idx, col)| analyze_column(col, idx))
        .collect();

    if scores.len() < 2 {
        return Err(IngestError::SchemaDetection {
            file_type: "Items".to_string(),
            path: path.to_path_buf(),
            reason: "could not analyze columns".to_string(),
        });
    }

    // ID column: highest uniqueness with no empty values, shorter values
    // ID columns have: high uniqueness, no empty values, shorter avg length than labels
    // Priority: uniqueness high + empty_ratio low + shorter values preferred
    let id_scores = scores
        .iter()
        .filter(|s| s.empty_ratio < 0.1) // ID column shouldn't have empty values
        .max_by(|a, b| {
            // Score = uniqueness / (1 + avg_length/10) - prioritize short unique columns
            let score_a = a.uniqueness / (1.0 + a.avg_length / 10.0);
            let score_b = b.uniqueness / (1.0 + b.avg_length / 10.0);
            score_a.partial_cmp(&score_b).unwrap()
        })
        .ok_or_else(|| IngestError::SchemaDetection {
            file_type: "Items".to_string(),
            path: path.to_path_buf(),
            reason: "could not detect ID column".to_string(),
        })?;

    let id = ColumnRole {
        index: id_scores.index,
        name: id_scores.name.clone(),
        confidence: id_scores.uniqueness,
    };

    // Label column: longest average length (excluding ID)
    let label_scores = scores
        .iter()
        .filter(|s| s.index != id.index)
        .max_by(|a, b| a.avg_length.partial_cmp(&b.avg_length).unwrap())
        .ok_or_else(|| IngestError::SchemaDetection {
            file_type: "Items".to_string(),
            path: path.to_path_buf(),
            reason: "could not detect label column".to_string(),
        })?;

    let label = ColumnRole {
        index: label_scores.index,
        name: label_scores.name.clone(),
        confidence: if label_scores.avg_length > 10.0 {
            0.8
        } else {
            0.5
        },
    };

    // DataType column: very low cardinality (2-8 unique values), short values, not ID/Label
    let data_type = scores
        .iter()
        .filter(|s| {
            s.index != id.index
                && s.index != label.index
                && s.cardinality >= 2
                && s.cardinality <= 8
                && s.avg_length < 15.0
                && s.all_short_values
        })
        .min_by(|a, b| a.cardinality.cmp(&b.cardinality))
        .map(|s| ColumnRole {
            index: s.index,
            name: s.name.clone(),
            confidence: 0.7,
        });

    // Mandatory column: binary/ternary (2-3 unique values), very short values
    let mandatory = scores
        .iter()
        .find(|s| {
            s.index != id.index
                && s.index != label.index
                && data_type.as_ref().is_none_or(|dt| s.index != dt.index)
                && s.cardinality >= 2
                && s.cardinality <= 3
                && s.avg_length < 6.0
        })
        .map(|s| ColumnRole {
            index: s.index,
            name: s.name.clone(),
            confidence: 0.6,
        });

    // FormatName column: medium cardinality, many empty values
    let format_name = scores
        .iter()
        .filter(|s| {
            s.index != id.index
                && s.index != label.index
                && data_type.as_ref().is_none_or(|dt| s.index != dt.index)
                && mandatory.as_ref().is_none_or(|m| s.index != m.index)
                && s.empty_ratio > 0.2 // Many empty values (not all items have formats)
                && s.avg_length < 20.0
        })
        .max_by(|a, b| a.empty_ratio.partial_cmp(&b.empty_ratio).unwrap())
        .map(|s| ColumnRole {
            index: s.index,
            name: s.name.clone(),
            confidence: 0.5,
        });

    // ContentLength column: all numeric, short values
    let content_length = scores
        .iter()
        .find(|s| {
            s.index != id.index
                && s.index != label.index
                && data_type.as_ref().is_none_or(|dt| s.index != dt.index)
                && mandatory.as_ref().is_none_or(|m| s.index != m.index)
                && format_name.as_ref().is_none_or(|f| s.index != f.index)
                && s.numeric_ratio > 0.9
                && s.avg_length < 5.0
        })
        .map(|s| ColumnRole {
            index: s.index,
            name: s.name.clone(),
            confidence: 0.7,
        });

    Ok(ItemsSchema {
        id,
        label,
        data_type,
        mandatory,
        format_name,
        content_length,
    })
}

/// Detects the schema for a CodeLists.csv file using purely statistical patterns.
///
/// Detection strategy (no hardcoded keywords):
/// - **FormatName column**: Lowest uniqueness (repeated values per group)
/// - **CodeText column**: Longest average length (descriptive text)
/// - **CodeValue column**: Shortest average length (codes are brief)
pub fn detect_codelist_schema(df: &DataFrame, path: &std::path::Path) -> Result<CodelistSchema> {
    let columns = df.get_columns();
    if columns.len() < 3 {
        return Err(IngestError::SchemaDetection {
            file_type: "CodeLists".to_string(),
            path: path.to_path_buf(),
            reason: "need at least 3 columns".to_string(),
        });
    }

    // Analyze all columns
    let scores: Vec<ColumnScores> = columns
        .iter()
        .enumerate()
        .filter_map(|(idx, col)| analyze_column(col, idx))
        .collect();

    if scores.len() < 3 {
        return Err(IngestError::SchemaDetection {
            file_type: "CodeLists".to_string(),
            path: path.to_path_buf(),
            reason: "could not analyze columns".to_string(),
        });
    }

    // Format name: lowest uniqueness (repeated values per group)
    // When uniqueness is tied, prefer shorter values (format names are typically short codes)
    let format_scores = scores
        .iter()
        .min_by(|a, b| {
            // Primary: lower uniqueness (more repeated values)
            // Secondary: shorter avg length (format names are short)
            let cmp = a.uniqueness.partial_cmp(&b.uniqueness).unwrap();
            if cmp == std::cmp::Ordering::Equal {
                a.avg_length.partial_cmp(&b.avg_length).unwrap()
            } else {
                cmp
            }
        })
        .unwrap();

    let format_name = ColumnRole {
        index: format_scores.index,
        name: format_scores.name.clone(),
        confidence: 1.0 - format_scores.uniqueness,
    };

    // Code text: high uniqueness + longer values (excluding format name)
    // CodeText has unique decode values, DataType has repeated type names
    let text_scores = scores
        .iter()
        .filter(|s| s.index != format_name.index)
        .max_by(|a, b| {
            // Score = uniqueness * avg_length - prioritize unique, longer values
            let score_a = a.uniqueness * a.avg_length;
            let score_b = b.uniqueness * b.avg_length;
            score_a.partial_cmp(&score_b).unwrap()
        })
        .ok_or_else(|| IngestError::SchemaDetection {
            file_type: "CodeLists".to_string(),
            path: path.to_path_buf(),
            reason: "could not detect code text column".to_string(),
        })?;

    let code_text = ColumnRole {
        index: text_scores.index,
        name: text_scores.name.clone(),
        confidence: if text_scores.avg_length > 5.0 {
            0.8
        } else {
            0.5
        },
    };

    // Code value: shortest average length (excluding format name and code text)
    let value_scores = scores
        .iter()
        .filter(|s| s.index != format_name.index && s.index != code_text.index)
        .min_by(|a, b| a.avg_length.partial_cmp(&b.avg_length).unwrap())
        .ok_or_else(|| IngestError::SchemaDetection {
            file_type: "CodeLists".to_string(),
            path: path.to_path_buf(),
            reason: "could not detect code value column".to_string(),
        })?;

    let code_value = ColumnRole {
        index: value_scores.index,
        name: value_scores.name.clone(),
        confidence: 0.7,
    };

    Ok(CodelistSchema {
        format_name,
        code_value,
        code_text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_items_df() -> DataFrame {
        df! {
            "ID" => &["AGE", "SEX", "RACE", "ETHNIC"],
            "Label" => &["Age in Years", "Gender of Subject", "Race Category", "Ethnicity Classification"],
            "DataType" => &["integer", "text", "text", "text"],
            "Mandatory" => &["True", "True", "False", "False"],
            "FormatName" => &["", "SEX", "RACE", "ETHNIC"],
            "ContentLength" => &["3", "1", "1", "1"],
        }
        .unwrap()
    }

    fn create_codelist_df() -> DataFrame {
        df! {
            "FormatName" => &["SEX", "SEX", "RACE", "RACE", "RACE"],
            "CodeValue" => &["M", "F", "1", "2", "3"],
            "CodeText" => &["Male", "Female", "Asian", "Black", "White"],
        }
        .unwrap()
    }

    #[test]
    fn test_detect_items_schema() {
        let df = create_items_df();
        let path = std::path::Path::new("test_items.csv");
        let schema = detect_items_schema(&df, path).unwrap();

        assert_eq!(schema.id.name, "ID");
        assert_eq!(schema.label.name, "Label");
    }

    #[test]
    fn test_detect_codelist_schema() {
        let df = create_codelist_df();
        let path = std::path::Path::new("test_codelists.csv");
        let schema = detect_codelist_schema(&df, path).unwrap();

        assert_eq!(schema.format_name.name, "FormatName");
        assert_eq!(schema.code_value.name, "CodeValue");
        assert_eq!(schema.code_text.name, "CodeText");
    }

    #[test]
    fn test_column_analysis_uniqueness() {
        let series = Series::new("ID".into(), &["A", "B", "C", "D"]);
        let col = Column::from(series);
        let scores = analyze_column(&col, 0).unwrap();

        // All unique values
        assert_eq!(scores.uniqueness, 1.0);
        assert_eq!(scores.cardinality, 4);
    }

    #[test]
    fn test_column_analysis_repeated() {
        let series = Series::new("Format".into(), &["SEX", "SEX", "RACE", "RACE", "RACE"]);
        let col = Column::from(series);
        let scores = analyze_column(&col, 0).unwrap();

        // 2 unique values out of 5 rows
        assert!((scores.uniqueness - 0.4).abs() < 0.01);
        assert_eq!(scores.cardinality, 2);
    }

    #[test]
    fn test_column_analysis_numeric() {
        let series = Series::new("Length".into(), &["3", "1", "1", "1"]);
        let col = Column::from(series);
        let scores = analyze_column(&col, 0).unwrap();

        assert!(scores.numeric_ratio > 0.9);
    }
}
