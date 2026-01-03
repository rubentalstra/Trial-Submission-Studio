//! Column hints and sample value extraction.

use std::collections::{BTreeMap, BTreeSet};

use polars::prelude::*;
use tss_common::any_to_string;
use tss_map::ColumnHint;

/// Builds column hints from a DataFrame.
///
/// Analyzes each column to determine:
/// - Whether values are numeric
/// - Ratio of unique values (cardinality)
/// - Ratio of null/missing values
pub fn build_column_hints(df: &DataFrame) -> BTreeMap<String, ColumnHint> {
    let mut hints = BTreeMap::new();

    for col in df.get_columns() {
        let name = col.name().to_string();
        let hint = analyze_column_for_hint(col);
        hints.insert(name, hint);
    }

    hints
}

/// Analyzes a column to create a ColumnHint.
fn analyze_column_for_hint(col: &Column) -> ColumnHint {
    let total = col.len();
    if total == 0 {
        return ColumnHint {
            is_numeric: false,
            unique_ratio: 0.0,
            null_ratio: 1.0,
            label: None,
        };
    }

    // Count nulls/empty and unique values
    let mut null_count = 0usize;
    let mut unique_values: BTreeSet<String> = BTreeSet::new();
    let mut numeric_count = 0usize;

    // Cast to string for analysis
    let str_col = col
        .cast(&DataType::String)
        .map(Column::take_materialized_series)
        .unwrap_or_else(|_| col.as_materialized_series().clone());
    let str_chunked = str_col.str().ok();

    if let Some(chunked) = str_chunked {
        for opt_val in chunked.iter() {
            match opt_val {
                Some(val) => {
                    let trimmed = val.trim();
                    if trimmed.is_empty() {
                        null_count += 1;
                    } else {
                        unique_values.insert(trimmed.to_string());

                        // Check if numeric
                        if trimmed.parse::<f64>().is_ok() {
                            numeric_count += 1;
                        }
                    }
                }
                None => null_count += 1,
            }
        }
    } else {
        // Fallback for non-string columns
        for i in 0..total {
            if let Ok(val) = col.get(i) {
                let s = any_to_string(val);
                if s.is_empty() {
                    null_count += 1;
                } else {
                    unique_values.insert(s);
                }
            }
        }
    }

    let non_null = total - null_count;
    let null_ratio = null_count as f64 / total as f64;
    let unique_ratio = if non_null > 0 {
        unique_values.len() as f64 / non_null as f64
    } else {
        0.0
    };

    // Determine if numeric (>90% of non-null values are numeric)
    let is_numeric = non_null > 0 && (numeric_count as f64 / non_null as f64) > 0.9;

    ColumnHint {
        is_numeric,
        unique_ratio,
        null_ratio,
        label: None,
    }
}

/// Gets sample unique values from a column.
///
/// Returns up to `limit` unique non-empty values.
pub fn get_sample_values(df: &DataFrame, column: &str, limit: usize) -> Vec<String> {
    let col = match df.column(column) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let str_col = match col.cast(&DataType::String) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let str_chunked = match str_col.str() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut unique: BTreeSet<String> = BTreeSet::new();

    for val in str_chunked.iter().flatten() {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            unique.insert(trimmed.to_string());
            if unique.len() >= limit {
                break;
            }
        }
    }

    unique.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_column_hints() {
        let df = df! {
            "id" => &["001", "002", "003"],
            "age" => &["25", "30", "35"],
            "name" => &["Alice", "Bob", "Charlie"],
        }
        .unwrap();

        let hints = build_column_hints(&df);

        assert_eq!(hints.len(), 3);

        // Age should be numeric
        let age_hint = hints.get("age").unwrap();
        assert!(age_hint.is_numeric);
        assert!((age_hint.unique_ratio - 1.0).abs() < 0.01); // 3 unique out of 3

        // Name should not be numeric
        let name_hint = hints.get("name").unwrap();
        assert!(!name_hint.is_numeric);
    }

    #[test]
    fn test_get_sample_values() {
        let df = df! {
            "col" => &["A", "B", "C", "D", "E", "F", "G"],
        }
        .unwrap();

        let samples = get_sample_values(&df, "col", 3);
        assert_eq!(samples.len(), 3);
    }

    #[test]
    fn test_column_with_nulls() {
        let df = df! {
            "col" => &[Some("A"), None, Some("B"), Some(""), None],
        }
        .unwrap();

        let hints = build_column_hints(&df);
        let hint = hints.get("col").unwrap();

        // 3 nulls/empty out of 5
        assert!((hint.null_ratio - 0.6).abs() < 0.01);
        // 2 unique out of 2 non-null
        assert!((hint.unique_ratio - 1.0).abs() < 0.01);
    }
}
