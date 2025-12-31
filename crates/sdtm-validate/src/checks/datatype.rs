//! Data type validation (SDTMIG 2.4).
//!
//! Checks that Num variables contain only numeric values.

use polars::prelude::{AnyValue, DataFrame, DataType};
use sdtm_common::any_to_string;
use sdtm_model::{Domain, VariableType};

use crate::issue::Issue;
use crate::util::CaseInsensitiveSet;

/// Check that Num variables contain only numeric values.
pub fn check(domain: &Domain, df: &DataFrame, columns: &CaseInsensitiveSet) -> Vec<Issue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        if variable.data_type != VariableType::Num {
            continue;
        }

        let Some(column) = columns.get(&variable.name) else {
            continue;
        };

        let Ok(series) = df.column(column) else {
            continue;
        };

        // Check if the Polars column is numeric
        let dtype = series.dtype();
        let is_numeric = matches!(
            dtype,
            DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
                | DataType::Float32
                | DataType::Float64
        );

        if is_numeric {
            continue;
        }

        // String column - check if values can be parsed as numbers
        let (non_numeric_count, samples) = collect_non_numeric_values(df, column);
        if non_numeric_count > 0 {
            issues.push(Issue::DataTypeMismatch {
                variable: variable.name.clone(),
                non_numeric_count,
                samples,
            });
        }
    }

    issues
}

/// Collect non-numeric values from a column that should be numeric.
fn collect_non_numeric_values(df: &DataFrame, column: &str) -> (u64, Vec<String>) {
    let Ok(series) = df.column(column) else {
        return (0, vec![]);
    };

    let mut count = 0u64;
    let mut samples = Vec::new();
    const MAX_SAMPLES: usize = 5;

    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        let str_value = any_to_string(value);
        let trimmed = str_value.trim();

        if trimmed.is_empty() {
            continue; // Nulls are not type errors
        }

        // Try to parse as a number
        if trimmed.parse::<f64>().is_err() {
            count += 1;
            if samples.len() < MAX_SAMPLES {
                samples.push(trimmed.to_string());
            }
        }
    }

    (count, samples)
}
