//! DataFrame construction utilities.
//!
//! Provides functions to build SDTM domain DataFrames from various sources.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use polars::prelude::{Column, DataFrame, NamedFrom, Series};

use sdtm_ingest::{parse_f64, CsvTable};
use sdtm_model::{Domain, MappingConfig, VariableType};

use crate::frame::DomainFrame;

/// Build a DataFrame from a vector of record maps.
///
/// Used to construct frames for SUPPQUAL and relationship datasets.
///
/// # Arguments
///
/// * `domain` - The domain definition with variable metadata
/// * `records` - Vector of record maps (variable name -> value)
///
/// # Returns
///
/// A DataFrame with columns matching the domain variables.
pub fn build_domain_frame_from_records(
    domain: &Domain,
    records: &[BTreeMap<String, String>],
) -> Result<DataFrame> {
    let mut columns: Vec<Column> = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        match variable.data_type {
            VariableType::Num => {
                let mut values: Vec<Option<f64>> = Vec::with_capacity(records.len());
                for record in records {
                    let raw = record.get(&variable.name).map(|v| v.trim()).unwrap_or("");
                    values.push(parse_f64(raw));
                }
                columns.push(Series::new(variable.name.as_str().into(), values).into());
            }
            VariableType::Char => {
                let mut values: Vec<String> = Vec::with_capacity(records.len());
                for record in records {
                    values.push(record.get(&variable.name).cloned().unwrap_or_default());
                }
                columns.push(Series::new(variable.name.as_str().into(), values).into());
            }
            // Handle future VariableType variants as strings
            _ => {
                let mut values: Vec<String> = Vec::with_capacity(records.len());
                for record in records {
                    values.push(record.get(&variable.name).cloned().unwrap_or_default());
                }
                columns.push(Series::new(variable.name.as_str().into(), values).into());
            }
        }
    }
    let data = DataFrame::new(columns).context("build dataframe from records")?;
    Ok(data)
}

/// Build a basic domain frame from a CSV table without column mapping.
///
/// Creates a DataFrame with columns matching the CSV headers exactly.
/// Headers are deduplicated by appending suffixes for duplicates.
pub fn build_domain_frame(table: &CsvTable, domain_code: &str) -> Result<DomainFrame> {
    let headers = dedupe_headers(&table.headers);
    let column_values = collect_table_columns(table);
    let mut columns: Vec<Column> = Vec::with_capacity(headers.len());
    for (header, values) in headers.iter().zip(column_values) {
        columns.push(Series::new(header.as_str().into(), values).into());
    }
    let data = DataFrame::new(columns).context("build dataframe")?;
    Ok(DomainFrame::new(domain_code.to_string(), data))
}

fn dedupe_headers(headers: &[String]) -> Vec<String> {
    let mut seen: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut deduped = Vec::with_capacity(headers.len());
    for header in headers {
        let key = header.to_uppercase();
        let count = seen.entry(key).or_insert(0);
        *count += 1;
        if *count == 1 {
            deduped.push(header.clone());
        } else {
            deduped.push(format!("{header}__{count}"));
        }
    }
    deduped
}

/// Collect column values from a CSV table as vectors.
pub fn collect_table_columns(table: &CsvTable) -> Vec<Vec<String>> {
    let row_count = table.rows.len();
    let mut columns: Vec<Vec<String>> = (0..table.headers.len())
        .map(|_| Vec::with_capacity(row_count))
        .collect();
    for row in &table.rows {
        for (col_idx, column) in columns.iter_mut().enumerate() {
            column.push(row.get(col_idx).cloned().unwrap_or_default());
        }
    }
    columns
}

/// Build a domain frame using column mapping configuration.
///
/// Maps source columns to SDTM variables according to the mapping config,
/// applying type conversions and populating STUDYID/DOMAIN columns.
pub fn build_domain_frame_with_mapping(
    table: &CsvTable,
    domain: &Domain,
    mapping: Option<&MappingConfig>,
) -> Result<DomainFrame> {
    let row_count = table.rows.len();
    let column_values = collect_table_columns(table);
    let mut source_indices = BTreeMap::new();
    let mut source_upper = BTreeMap::new();
    for (col_idx, header) in table.headers.iter().enumerate() {
        source_indices.insert(header.clone(), col_idx);
        source_upper.insert(header.to_uppercase(), col_idx);
    }
    let mapping_lookup = mapping.map(|config| {
        let mut lookup = BTreeMap::new();
        for item in &config.mappings {
            lookup.insert(item.target_variable.to_uppercase(), item);
        }
        lookup
    });

    let mut columns: Vec<Column> = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let target_upper = variable.name.to_uppercase();
        let source_index = mapping_lookup
            .as_ref()
            .and_then(|lookup| lookup.get(&target_upper))
            .and_then(|suggestion| {
                let source_name = suggestion
                    .transformation
                    .as_deref()
                    .filter(|name| source_indices.contains_key(*name))
                    .unwrap_or(suggestion.source_column.as_str());
                source_indices.get(source_name).copied()
            })
            .or_else(|| source_upper.get(&target_upper).copied());

        let values = source_index
            .map(|idx| column_values[idx].clone())
            .unwrap_or_else(|| vec![String::new(); row_count]);

        let column: Column = match variable.data_type {
            VariableType::Num => {
                let numeric: Vec<Option<f64>> = values
                    .iter()
                    .map(|value| value.trim().parse::<f64>().ok())
                    .collect();
                Series::new(variable.name.as_str().into(), numeric).into()
            }
            VariableType::Char => Series::new(variable.name.as_str().into(), values).into(),
            // Handle future VariableType variants as strings
            _ => Series::new(variable.name.as_str().into(), values).into(),
        };
        columns.push(column);
    }

    let mut data = DataFrame::new(columns).context("build dataframe")?;
    if let Some(config) = mapping
        && let Some(study_col) = domain.column_name("STUDYID")
        && let Ok(series) = data.column(study_col)
    {
        let values = vec![config.study_id.clone(); row_count];
        let new_series = Series::new(series.name().as_str().into(), values);
        data.with_column(new_series)?;
    }
    if let Some(domain_col) = domain.column_name("DOMAIN")
        && data.column(domain_col).is_ok()
    {
        let values = vec![domain.code.clone(); row_count];
        let new_series = Series::new(domain_col.into(), values);
        data.with_column(new_series)?;
    }

    Ok(DomainFrame::new(domain.code.clone(), data))
}
