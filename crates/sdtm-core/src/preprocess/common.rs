//! Common utilities for preprocessing operations.
//!
//! This module provides shared utilities used across all domain-specific
//! preprocessing modules, including column access helpers, value transformation
//! utilities, and the preprocessing context structure.

use std::collections::HashMap;

use anyhow::Result;
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};

use sdtm_ingest::{CsvTable, any_to_string};
use sdtm_model::{CaseInsensitiveLookup, Domain, MappingConfig};

use crate::ProcessingContext;
use crate::data_utils::{
    column_hint_for_domain, mapping_source_for_target, table_column_values, table_label,
};

/// Configuration for preprocessing operations.
#[derive(Debug, Clone)]
pub struct PreprocessConfig {
    /// Whether to allow heuristic inference from source data.
    pub allow_heuristic_inference: bool,
    /// Whether to require explicit mapping metadata for value population.
    pub require_explicit_mapping: bool,
}

impl Default for PreprocessConfig {
    fn default() -> Self {
        Self {
            allow_heuristic_inference: true,
            require_explicit_mapping: false,
        }
    }
}

/// Context for preprocessing operations.
///
/// This struct provides convenient access to all resources needed during
/// preprocessing: domain metadata, mapping configuration, source table,
/// and processing context.
pub struct PreprocessContext<'a> {
    /// Domain metadata from SDTMIG standards.
    pub domain: &'a Domain,
    /// Mapping configuration for this domain.
    pub mapping: &'a MappingConfig,
    /// Source CSV table.
    pub table: &'a CsvTable,
    /// Processing context with study metadata and options.
    pub ctx: &'a ProcessingContext<'a>,
}

impl<'a> PreprocessContext<'a> {
    /// Create a new preprocessing context.
    pub fn new(
        domain: &'a Domain,
        mapping: &'a MappingConfig,
        table: &'a CsvTable,
        ctx: &'a ProcessingContext<'a>,
    ) -> Self {
        Self {
            domain,
            mapping,
            table,
            ctx,
        }
    }

    /// Get the domain code in uppercase.
    pub fn domain_code(&self) -> String {
        self.domain.code.to_uppercase()
    }

    /// Build a column lookup from the DataFrame.
    pub fn build_column_lookup(&self, df: &DataFrame) -> CaseInsensitiveLookup {
        CaseInsensitiveLookup::new(df.get_column_names_owned())
    }

    /// Get the actual column name (case-insensitive lookup).
    pub fn column_name(&self, df: &DataFrame, name: &str) -> String {
        let lookup = self.build_column_lookup(df);
        lookup
            .get(name)
            .map(|value| value.to_string())
            .unwrap_or_else(|| name.to_string())
    }

    /// Get mapping source for a target variable.
    pub fn mapping_source(&self, target: &str) -> Option<String> {
        mapping_source_for_target(self.mapping, target)
    }

    /// Get column hint (label + allow_raw flag) for a domain column.
    pub fn column_hint(&self, column: &str) -> Option<(String, bool)> {
        column_hint_for_domain(self.table, self.domain, column)
    }

    /// Get the label for a source column.
    pub fn source_label(&self, column: &str) -> Option<String> {
        table_label(self.table, column)
    }

    /// Get values from a source column.
    pub fn source_values(&self, column: &str) -> Option<Vec<String>> {
        table_column_values(self.table, column)
    }

    /// Get standard variable names set for the domain.
    pub fn standard_variable_set(&self) -> std::collections::BTreeSet<String> {
        let mut vars = std::collections::BTreeSet::new();
        for variable in &self.domain.variables {
            vars.insert(variable.name.to_uppercase());
        }
        vars
    }
}

/// Get column hint for a domain table.
pub fn column_hint_for_domain_table(
    table: &CsvTable,
    domain: &Domain,
    column: &str,
) -> Option<(String, bool)> {
    column_hint_for_domain(table, domain, column)
}

/// Check if a DataFrame has a column (case-insensitive).
pub fn has_column(df: &DataFrame, name: &str) -> bool {
    df.column(name).is_ok()
}

/// Get string values from a DataFrame column.
///
/// Returns a vector of trimmed string values, or empty strings for null values.
pub fn get_column_values(df: &DataFrame, name: &str) -> Result<Vec<String>> {
    let series = df.column(name)?;
    let mut values = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        values.push(value.trim().to_string());
    }
    Ok(values)
}

/// Get a single column value at a specific index.
pub fn get_column_value(df: &DataFrame, name: &str, idx: usize) -> Result<String> {
    let series = df.column(name)?;
    let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
    Ok(value.trim().to_string())
}

/// Set string values for a DataFrame column.
#[allow(dead_code)]
pub fn set_column_values(df: &mut DataFrame, name: &str, values: Vec<String>) -> Result<()> {
    let series = Series::new(name.into(), values);
    df.with_column(series)?;
    Ok(())
}

/// Initialize a column with empty strings if it doesn't exist.
#[allow(dead_code)]
pub fn ensure_column(df: &mut DataFrame, name: &str) -> Result<()> {
    if !has_column(df, name) {
        let values = vec![String::new(); df.height()];
        set_column_values(df, name, values)?;
    }
    Ok(())
}

/// Get or create column values.
///
/// Returns existing values if column exists, otherwise returns empty strings.
#[allow(dead_code)]
pub fn get_or_create_column_values(df: &DataFrame, name: &str) -> Vec<String> {
    if has_column(df, name) {
        get_column_values(df, name).unwrap_or_else(|_| vec![String::new(); df.height()])
    } else {
        vec![String::new(); df.height()]
    }
}

/// Fill empty values in a column with a constant.
#[allow(dead_code)]
pub fn fill_empty_values(values: &mut [String], fill: &str) {
    for value in values.iter_mut() {
        if value.trim().is_empty() {
            *value = fill.to_string();
        }
    }
}

/// Apply a transformation to all values in a column.
#[allow(dead_code)]
pub fn transform_column_values<F>(values: &mut [String], transform: F)
where
    F: Fn(&str) -> String,
{
    for value in values.iter_mut() {
        *value = transform(value);
    }
}

/// Create a value map from pairs for lookup transformations.
#[allow(dead_code)]
pub fn create_value_map<const N: usize>(pairs: [(&str, &str); N]) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(N);
    for (key, value) in pairs {
        map.insert(key.to_uppercase(), value.to_string());
    }
    map
}
