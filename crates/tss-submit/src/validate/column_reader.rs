//! Column reader utilities for validation.
//!
//! This module provides a `ColumnReader` abstraction that reduces boilerplate
//! when iterating over DataFrame columns during validation. It handles:
//!
//! - Column access by name (with proper error handling)
//! - Row iteration with automatic value conversion
//! - Common counting and filtering operations
//!
//! # Example
//!
//! ```ignore
//! let reader = ColumnReader::new(df);
//!
//! // Count null/empty values
//! let null_count = reader.count_nulls("STUDYID");
//!
//! // Count values matching a predicate
//! let invalid = reader.count_matching("SEX", |v| !["M", "F"].contains(&v.as_str()));
//!
//! // Collect matching values with a limit
//! let (count, values) = reader.collect_matching("AETERM", |v| v.len() > 200, 5);
//! ```

use polars::prelude::{AnyValue, Column, DataFrame};
use tss_standards::any_to_string;

/// A reader for accessing DataFrame columns with common validation operations.
///
/// This abstraction eliminates repeated boilerplate for:
/// - Column access
/// - Row iteration
/// - Value conversion to strings
/// - Null/empty checking
#[derive(Debug)]
pub struct ColumnReader<'a> {
    df: &'a DataFrame,
}

impl<'a> ColumnReader<'a> {
    /// Create a new column reader for the given DataFrame.
    #[inline]
    pub fn new(df: &'a DataFrame) -> Self {
        Self { df }
    }

    /// Get the number of rows in the DataFrame.
    #[inline]
    pub fn height(&self) -> usize {
        self.df.height()
    }

    /// Get a column by name, returning None if not found.
    #[inline]
    pub fn column(&self, name: &str) -> Option<&Column> {
        self.df.column(name).ok()
    }

    /// Check if a column exists.
    #[inline]
    pub fn has_column(&self, name: &str) -> bool {
        self.df.column(name).is_ok()
    }

    /// Get a string value from a column at a specific row index.
    ///
    /// Returns the trimmed string representation of the value.
    /// Returns empty string if the column doesn't exist or row is out of bounds.
    pub fn get_string(&self, column: &str, row_idx: usize) -> String {
        let Some(series) = self.column(column) else {
            return String::new();
        };
        let value = series.get(row_idx).unwrap_or(AnyValue::Null);
        any_to_string(value)
    }

    /// Iterate over string values in a column.
    ///
    /// Returns an iterator that yields (row_index, string_value) pairs.
    /// The string value is the result of `any_to_string()` on the cell value.
    pub fn values(&self, column_name: &str) -> Option<ColumnValueIter<'_>> {
        let column = self.column(column_name)?;
        Some(ColumnValueIter {
            column,
            current: 0,
            len: self.df.height(),
        })
    }

    /// Count null or empty string values in a column.
    ///
    /// A value is considered null/empty if `any_to_string(value).trim().is_empty()`.
    pub fn count_nulls(&self, column: &str) -> u64 {
        let Some(series) = self.column(column) else {
            return 0;
        };

        let mut count = 0u64;
        for idx in 0..self.df.height() {
            let value = series.get(idx).unwrap_or(AnyValue::Null);
            let str_value = any_to_string(value);
            if str_value.trim().is_empty() {
                count += 1;
            }
        }
        count
    }

    /// Count non-null/non-empty values in a column.
    ///
    /// Returns 0 if the column doesn't exist.
    pub fn count_non_nulls(&self, column: &str) -> u64 {
        // Return 0 for nonexistent columns (safer than total - 0 = total)
        if !self.has_column(column) {
            return 0;
        }
        let total = self.df.height() as u64;
        total.saturating_sub(self.count_nulls(column))
    }

    /// Count values that match a predicate.
    ///
    /// The predicate receives the trimmed string representation of each value.
    /// Null/empty values are not passed to the predicate.
    pub fn count_matching<F>(&self, column: &str, predicate: F) -> u64
    where
        F: Fn(&str) -> bool,
    {
        let Some(series) = self.column(column) else {
            return 0;
        };

        let mut count = 0u64;
        for idx in 0..self.df.height() {
            let value = series.get(idx).unwrap_or(AnyValue::Null);
            let str_value = any_to_string(value);
            let trimmed = str_value.trim();

            if !trimmed.is_empty() && predicate(trimmed) {
                count += 1;
            }
        }
        count
    }

    /// Collect values that match a predicate, with a limit.
    ///
    /// Returns (total_count, limited_values) where:
    /// - `total_count` is the total number of matching values
    /// - `limited_values` contains up to `limit` unique matching values
    ///
    /// Null/empty values are not passed to the predicate.
    pub fn collect_matching<F>(
        &self,
        column: &str,
        predicate: F,
        limit: usize,
    ) -> (u64, Vec<String>)
    where
        F: Fn(&str) -> bool,
    {
        let Some(series) = self.column(column) else {
            return (0, Vec::new());
        };

        let mut count = 0u64;
        let mut collected = Vec::with_capacity(limit);

        for idx in 0..self.df.height() {
            let value = series.get(idx).unwrap_or(AnyValue::Null);
            let str_value = any_to_string(value);
            let trimmed = str_value.trim();

            if !trimmed.is_empty() && predicate(trimmed) {
                count += 1;
                if collected.len() < limit && !collected.contains(&trimmed.to_string()) {
                    collected.push(trimmed.to_string());
                }
            }
        }

        (count, collected)
    }

    /// Collect unique values that match a predicate.
    ///
    /// Returns a set of unique trimmed string values.
    /// Null/empty values are not included.
    pub fn collect_unique_matching<F>(
        &self,
        column: &str,
        predicate: F,
    ) -> std::collections::BTreeSet<String>
    where
        F: Fn(&str) -> bool,
    {
        let mut result = std::collections::BTreeSet::new();

        let Some(series) = self.column(column) else {
            return result;
        };

        for idx in 0..self.df.height() {
            let value = series.get(idx).unwrap_or(AnyValue::Null);
            let str_value = any_to_string(value);
            let trimmed = str_value.trim();

            if !trimmed.is_empty() && predicate(trimmed) {
                result.insert(trimmed.to_string());
            }
        }

        result
    }

    /// Check length violations for a character column.
    ///
    /// Returns (violation_count, max_length_found) where:
    /// - `violation_count` is the number of values exceeding `max_length`
    /// - `max_length_found` is the maximum length encountered among violations
    pub fn length_violations(&self, column: &str, max_length: u32) -> (u64, usize) {
        let Some(series) = self.column(column) else {
            return (0, 0);
        };

        let mut count = 0u64;
        let mut max_found = 0usize;

        for idx in 0..self.df.height() {
            let value = series.get(idx).unwrap_or(AnyValue::Null);
            let str_value = any_to_string(value);
            let len = str_value.len();

            if len > max_length as usize {
                count += 1;
                max_found = max_found.max(len);
            }
        }

        (count, max_found)
    }

    /// Check if all values in a column are null/empty.
    pub fn all_null(&self, column: &str) -> bool {
        let height = self.df.height() as u64;
        height > 0 && self.count_nulls(column) == height
    }

    /// Get unique non-null values per subject (for sequence checks).
    ///
    /// Groups values by subject and returns a map of subject -> values.
    /// Useful for checking sequence uniqueness per subject.
    pub fn values_by_subject(
        &self,
        subject_column: &str,
        value_column: &str,
    ) -> std::collections::HashMap<String, Vec<String>> {
        let mut result: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        let (Some(subject_series), Some(value_series)) =
            (self.column(subject_column), self.column(value_column))
        else {
            return result;
        };

        for idx in 0..self.df.height() {
            let subject = any_to_string(subject_series.get(idx).unwrap_or(AnyValue::Null));
            let value = any_to_string(value_series.get(idx).unwrap_or(AnyValue::Null));

            if subject.trim().is_empty() {
                continue;
            }

            result
                .entry(subject.trim().to_string())
                .or_default()
                .push(value.trim().to_string());
        }

        result
    }
}

/// Iterator over string values in a column.
pub struct ColumnValueIter<'a> {
    column: &'a Column,
    current: usize,
    len: usize,
}

impl Iterator for ColumnValueIter<'_> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.len {
            return None;
        }

        let idx = self.current;
        self.current += 1;

        let value = self.column.get(idx).unwrap_or(AnyValue::Null);
        Some((idx, any_to_string(value)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len.saturating_sub(self.current);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ColumnValueIter<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn test_df() -> DataFrame {
        df! {
            "STUDYID" => &["STUDY01", "STUDY01", "STUDY01"],
            "USUBJID" => &["SUBJ01", "SUBJ02", "SUBJ03"],
            "AETERM" => &["HEADACHE", "", "NAUSEA"],
            "AESEQ" => &[1i64, 2, 3],
        }
        .unwrap()
    }

    #[test]
    fn test_count_nulls() {
        let df = test_df();
        let reader = ColumnReader::new(&df);

        assert_eq!(reader.count_nulls("STUDYID"), 0);
        assert_eq!(reader.count_nulls("AETERM"), 1); // Empty string counts as null
        assert_eq!(reader.count_nulls("NONEXISTENT"), 0);
    }

    #[test]
    fn test_count_matching() {
        let df = test_df();
        let reader = ColumnReader::new(&df);

        // Count values starting with "H"
        let count = reader.count_matching("AETERM", |v| v.starts_with('H'));
        assert_eq!(count, 1);
    }

    #[test]
    fn test_values_iterator() {
        let df = test_df();
        let reader = ColumnReader::new(&df);

        let values: Vec<_> = reader.values("USUBJID").unwrap().collect();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], (0, "SUBJ01".to_string()));
    }

    // ==========================================================================
    // Edge case tests - verifying safety guarantees (#112)
    // ==========================================================================

    /// Test that all methods handle empty DataFrames gracefully without panicking.
    #[test]
    fn test_empty_dataframe_safety() {
        let df = DataFrame::empty();
        let reader = ColumnReader::new(&df);

        // Basic properties
        assert_eq!(reader.height(), 0);
        assert!(!reader.has_column("ANY"));

        // Counting operations return 0
        assert_eq!(reader.count_nulls("STUDYID"), 0);
        assert_eq!(reader.count_non_nulls("STUDYID"), 0);
        assert_eq!(reader.count_matching("STUDYID", |_| true), 0);

        // Collection operations return empty results
        let (count, values) = reader.collect_matching("STUDYID", |_| true, 10);
        assert_eq!(count, 0);
        assert!(values.is_empty());

        let unique = reader.collect_unique_matching("STUDYID", |_| true);
        assert!(unique.is_empty());

        let by_subject = reader.values_by_subject("USUBJID", "AESEQ");
        assert!(by_subject.is_empty());

        // Length violations return zeros
        let (violations, max_found) = reader.length_violations("STUDYID", 10);
        assert_eq!(violations, 0);
        assert_eq!(max_found, 0);

        // all_null returns false for empty DataFrame
        assert!(!reader.all_null("STUDYID"));

        // get_string returns empty string
        assert_eq!(reader.get_string("STUDYID", 0), "");
    }

    /// Test that methods return None/empty when column doesn't exist.
    #[test]
    fn test_missing_column_returns_none_or_empty() {
        let df = test_df();
        let reader = ColumnReader::new(&df);

        // column() returns None
        assert!(reader.column("NONEXISTENT").is_none());

        // has_column returns false
        assert!(!reader.has_column("NONEXISTENT"));

        // values() returns None
        assert!(reader.values("NONEXISTENT").is_none());

        // get_string returns empty string
        assert_eq!(reader.get_string("NONEXISTENT", 0), "");

        // count operations return 0
        assert_eq!(reader.count_nulls("NONEXISTENT"), 0);
        assert_eq!(reader.count_non_nulls("NONEXISTENT"), 0);
        assert_eq!(reader.count_matching("NONEXISTENT", |_| true), 0);

        // collect operations return empty
        let (count, values) = reader.collect_matching("NONEXISTENT", |_| true, 10);
        assert_eq!(count, 0);
        assert!(values.is_empty());

        let unique = reader.collect_unique_matching("NONEXISTENT", |_| true);
        assert!(unique.is_empty());

        // length_violations returns zeros
        let (violations, max_found) = reader.length_violations("NONEXISTENT", 10);
        assert_eq!(violations, 0);
        assert_eq!(max_found, 0);

        // all_null returns false (column doesn't exist = not all null)
        assert!(!reader.all_null("NONEXISTENT"));

        // values_by_subject returns empty when either column is missing
        let by_subject = reader.values_by_subject("NONEXISTENT", "AESEQ");
        assert!(by_subject.is_empty());
        let by_subject = reader.values_by_subject("USUBJID", "NONEXISTENT");
        assert!(by_subject.is_empty());
    }

    /// Test that out-of-bounds row access doesn't panic.
    #[test]
    fn test_out_of_bounds_row_access() {
        let df = test_df();
        let reader = ColumnReader::new(&df);

        // get_string with out-of-bounds index returns empty string
        assert_eq!(reader.get_string("STUDYID", 999), "");
        assert_eq!(reader.get_string("STUDYID", usize::MAX), "");
    }

    /// Test handling of DataFrame with columns but zero rows.
    #[test]
    fn test_empty_columns_no_rows() {
        let df = df! {
            "STUDYID" => Vec::<String>::new(),
            "USUBJID" => Vec::<String>::new(),
        }
        .unwrap();
        let reader = ColumnReader::new(&df);

        assert_eq!(reader.height(), 0);
        assert!(reader.has_column("STUDYID"));
        assert_eq!(reader.count_nulls("STUDYID"), 0);
        assert!(!reader.all_null("STUDYID")); // Empty = not all null

        let values: Vec<_> = reader.values("STUDYID").unwrap().collect();
        assert!(values.is_empty());
    }

    /// Test that null values in polars are handled correctly.
    #[test]
    fn test_null_values_handling() {
        let df = df! {
            "COL" => &[Some("A"), None, Some(""), None, Some("B")],
        }
        .unwrap();
        let reader = ColumnReader::new(&df);

        // Nulls and empty strings both count as "null" in our abstraction
        assert_eq!(reader.count_nulls("COL"), 3); // 2 None + 1 empty string
        assert_eq!(reader.count_non_nulls("COL"), 2); // "A" and "B"
    }

    /// Test exact size iterator implementation.
    #[test]
    fn test_values_iterator_exact_size() {
        let df = test_df();
        let reader = ColumnReader::new(&df);

        let iter = reader.values("STUDYID").unwrap();
        assert_eq!(iter.len(), 3);

        // After consuming one, len should decrease
        let mut iter = reader.values("STUDYID").unwrap();
        assert_eq!(iter.len(), 3);
        let _ = iter.next();
        assert_eq!(iter.len(), 2);
    }
}
