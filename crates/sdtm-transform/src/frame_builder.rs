//! DataFrame construction utilities.
//!
//! Provides functions to build SDTM domain DataFrames from various sources.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use polars::prelude::{Column, DataFrame, NamedFrom, Series};

use sdtm_ingest::parse_f64;
use sdtm_model::{Domain, VariableType};

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
