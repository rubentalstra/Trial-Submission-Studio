//! Shared utilities for wide format processing.

use std::collections::BTreeSet;

use anyhow::Result;
use polars::prelude::{DataFrame, NamedFrom, Series};

use sdtm_ingest::build_column_hints;
use sdtm_map::MappingEngine;
use sdtm_model::{Domain, MappingConfig, VariableType};

use crate::data_utils::column_value_string;
use crate::frame::DomainFrame;
use crate::frame_builder::build_domain_frame_with_mapping;

/// Build base mapping for non-wide columns.
pub fn build_wide_base_mapping(
    table: &DataFrame,
    domain: &Domain,
    study_id: &str,
    wide_columns: &BTreeSet<String>,
) -> Result<(MappingConfig, DomainFrame)> {
    let base_table = filter_table_columns(table, wide_columns, false)?;
    let hints = build_column_hints(&base_table);
    let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
    let headers: Vec<String> = base_table
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let result = engine.suggest(&headers);
    let mapping_config = engine.to_config(study_id, result);
    let base_frame = build_domain_frame_with_mapping(&base_table, domain, Some(&mapping_config))?;
    Ok((mapping_config, base_frame))
}

/// Extract source columns used in a mapping configuration.
pub fn mapping_used_sources(mapping: &MappingConfig) -> BTreeSet<String> {
    mapping
        .mappings
        .iter()
        .map(|item| item.source_column.clone())
        .collect()
}

/// Filter table columns by inclusion/exclusion set.
pub fn filter_table_columns(
    table: &DataFrame,
    columns: &BTreeSet<String>,
    include: bool,
) -> Result<DataFrame> {
    let col_names = table.get_column_names();
    let selection: Vec<&str> = col_names
        .iter()
        .filter(|name| {
            let has = columns.contains(&name.to_uppercase());
            has == include
        })
        .map(|s| s.as_str())
        .collect();

    Ok(table.select(selection)?)
}

/// Extract values from base DataFrame for a single row.
pub fn base_row_values(
    base_df: &DataFrame,
    variable_names: &[String],
    row_idx: usize,
) -> Vec<String> {
    variable_names
        .iter()
        .map(|name| column_value_string(base_df, name, row_idx))
        .collect()
}

/// Push a row of values into the output value vectors.
pub fn push_row(values: &mut [Vec<String>], row: Vec<String>) {
    for (idx, value) in row.into_iter().enumerate() {
        values[idx].push(value);
    }
}

/// Build DataFrame from collected wide format values.
pub fn build_wide_data(domain: &Domain, mut values: Vec<Vec<String>>) -> Result<DataFrame> {
    let mut columns = Vec::with_capacity(domain.variables.len());
    for (idx, variable) in domain.variables.iter().enumerate() {
        let vals = values.get_mut(idx).map(std::mem::take).unwrap_or_default();
        let column = match variable.data_type {
            VariableType::Num => {
                let numeric: Vec<Option<f64>> = vals
                    .iter()
                    .map(|value| value.trim().parse::<f64>().ok())
                    .collect();
                Series::new(variable.name.as_str().into(), numeric).into()
            }
            VariableType::Char => Series::new(variable.name.as_str().into(), vals).into(),
            // Handle future VariableType variants as strings
            _ => Series::new(variable.name.as_str().into(), vals).into(),
        };
        columns.push(column);
    }
    DataFrame::new(columns).map_err(Into::into)
}

/// Normalize a numeric string value, returning empty for non-numeric.
pub fn normalize_numeric(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.parse::<f64>().is_ok() {
        trimmed.to_string()
    } else {
        String::new()
    }
}
