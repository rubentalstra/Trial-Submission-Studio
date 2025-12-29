//! DataFrame construction utilities.
//!
//! Provides functions to build SDTM domain DataFrames from various sources.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use polars::prelude::*;

use sdtm_ingest::parse_f64;
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

/// Build a basic domain frame from a DataFrame without column mapping.
///
/// Creates a DataFrame with columns matching the source DataFrame.
/// Headers are deduplicated by Polars automatically, but we ensure domain code is set.
pub fn build_domain_frame(table: &DataFrame, domain_code: &str) -> Result<DomainFrame> {
    // Polars DataFrame is already built, just wrap it.
    // We might want to ensure column names are unique if they aren't already,
    // but Polars usually handles that on read.
    Ok(DomainFrame::new(domain_code.to_string(), table.clone()))
}

/// Build a domain frame using column mapping configuration.
///
/// Maps source columns to SDTM variables according to the mapping config,
/// applying type conversions and populating STUDYID/DOMAIN columns.
pub fn build_domain_frame_with_mapping(
    table: &DataFrame,
    domain: &Domain,
    mapping: Option<&MappingConfig>,
) -> Result<DomainFrame> {
    let mut expressions: Vec<Expr> = Vec::new();

    // Pre-calculate mapping lookup
    let mapping_lookup = mapping.map(|config| {
        let mut lookup = BTreeMap::new();
        for item in &config.mappings {
            lookup.insert(item.target_variable.to_uppercase(), item);
        }
        lookup
    });

    let source_columns: BTreeMap<String, String> = table
        .get_column_names()
        .iter()
        .map(|name| (name.to_uppercase(), name.to_string()))
        .collect();

    for variable in &domain.variables {
        let target_upper = variable.name.to_uppercase();

        // Handle STUDYID and DOMAIN specially to ensure they are in the correct order
        // and use the constant values if available.
        if target_upper == "STUDYID" {
            if let Some(config) = mapping {
                expressions.push(lit(config.study_id.clone()).alias(&variable.name));
                continue;
            }
        }
        if target_upper == "DOMAIN" {
            expressions.push(lit(domain.code.clone()).alias(&variable.name));
            continue;
        }

        // Determine source column name
        let source_col_name = mapping_lookup
            .as_ref()
            .and_then(|lookup| lookup.get(&target_upper))
            .and_then(|suggestion| {
                let source_name = suggestion
                    .transformation
                    .as_deref()
                    .filter(|name| source_columns.contains_key(&name.to_uppercase()))
                    .unwrap_or(suggestion.source_column.as_str());

                // Find exact case in source
                source_columns.get(&source_name.to_uppercase())
            })
            .or_else(|| source_columns.get(&target_upper));

        if let Some(source_name) = source_col_name {
            let mut expr = col(source_name);

            // Apply type conversion
            if matches!(variable.data_type, VariableType::Num) {
                // Try to cast to Float64, handling non-numeric strings if necessary
                expr = expr
                    .cast(DataType::String)
                    .str()
                    .strip_chars(lit(" "))
                    .cast(DataType::Float64);
            } else {
                expr = expr.cast(DataType::String);
            }

            expressions.push(expr.alias(&variable.name));
        } else {
            // Column missing in source, fill with nulls/empty
            let expr = match variable.data_type {
                VariableType::Num => lit(NULL).cast(DataType::Float64),
                _ => lit("").cast(DataType::String),
            };
            expressions.push(expr.alias(&variable.name));
        }
    }

    let new_df = table.clone().lazy().select(expressions).collect()?;

    Ok(DomainFrame::new(domain.code.clone(), new_df))
}
