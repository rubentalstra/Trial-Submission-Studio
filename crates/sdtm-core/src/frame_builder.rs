use std::collections::BTreeSet;

use anyhow::{Context, Result};
use polars::prelude::{Column, DataFrame, NamedFrom, Series};

use sdtm_ingest::CsvTable;
use sdtm_ingest::build_column_hints;
use sdtm_map::MappingEngine;
use sdtm_model::{Domain, MappingConfig, VariableType};

use crate::domain_utils::standard_columns;
use crate::frame::DomainFrame;
use crate::wide::{build_ie_wide_frame, build_lb_wide_frame, build_vs_wide_frame};

#[derive(Debug)]
pub struct MappedDomainFrame {
    pub mapping: MappingConfig,
    pub frame: DomainFrame,
    pub used_columns: BTreeSet<String>,
}

pub fn build_domain_frame(table: &CsvTable, domain_code: &str) -> Result<DomainFrame> {
    let headers = dedupe_headers(&table.headers);
    let mut columns: Vec<Column> = Vec::with_capacity(headers.len());
    for (col_idx, header) in headers.iter().enumerate() {
        let mut values = Vec::with_capacity(table.rows.len());
        for row in &table.rows {
            values.push(row.get(col_idx).cloned().unwrap_or_default());
        }
        columns.push(Series::new(header.as_str().into(), values).into());
    }
    let data = DataFrame::new(columns).context("build dataframe")?;
    Ok(DomainFrame {
        domain_code: domain_code.to_string(),
        data,
    })
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

pub fn build_domain_frame_with_mapping(
    table: &CsvTable,
    domain: &Domain,
    mapping: Option<&MappingConfig>,
) -> Result<DomainFrame> {
    let row_count = table.rows.len();
    let mut source_columns = std::collections::BTreeMap::new();
    let mut source_upper = std::collections::BTreeMap::new();
    for (col_idx, header) in table.headers.iter().enumerate() {
        let mut values = Vec::with_capacity(row_count);
        for row in &table.rows {
            values.push(row.get(col_idx).cloned().unwrap_or_default());
        }
        source_columns.insert(header.clone(), values);
        source_upper.insert(header.to_uppercase(), header.clone());
    }

    let mut mapping_lookup = std::collections::BTreeMap::new();
    if let Some(config) = mapping {
        for item in &config.mappings {
            mapping_lookup.insert(item.target_variable.to_uppercase(), item);
        }
    }

    let mut columns: Vec<Column> = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let mut values: Vec<String> = Vec::with_capacity(row_count);
        let target_upper = variable.name.to_uppercase();
        if mapping.is_some() {
            if let Some(suggestion) = mapping_lookup.get(&target_upper) {
                let mut source_name = suggestion.source_column.as_str();
                if let Some(transformation) = suggestion.transformation.as_deref()
                    && source_columns.contains_key(transformation)
                {
                    source_name = transformation;
                }
                if let Some(source) = source_columns.get(source_name) {
                    values = source.clone();
                }
            }
        } else if let Some(source_name) = source_upper.get(&target_upper)
            && let Some(source) = source_columns.get(source_name)
        {
            values = source.clone();
        }

        if values.is_empty() {
            values = vec![String::new(); row_count];
        }

        let column: Column = match variable.data_type {
            VariableType::Num => {
                let numeric: Vec<Option<f64>> = values
                    .iter()
                    .map(|value| value.trim().parse::<f64>().ok())
                    .collect();
                Series::new(variable.name.as_str().into(), numeric).into()
            }
            VariableType::Char => Series::new(variable.name.as_str().into(), values).into(),
        };
        columns.push(column);
    }

    let mut data = DataFrame::new(columns).context("build dataframe")?;
    let standard = standard_columns(domain);
    if let Some(config) = mapping
        && let Some(study_col) = standard.study_id.as_ref()
        && let Ok(series) = data.column(study_col)
    {
        let values = vec![config.study_id.clone(); row_count];
        let new_series = Series::new(series.name().as_str().into(), values);
        data.with_column(new_series)?;
    }
    if let Some(domain_col) = standard.domain.as_ref()
        && data.column(domain_col).is_ok()
    {
        let values = vec![domain.code.clone(); row_count];
        let new_series = Series::new(domain_col.as_str().into(), values);
        data.with_column(new_series)?;
    }

    Ok(DomainFrame {
        domain_code: domain.code.clone(),
        data,
    })
}

pub fn build_mapped_domain_frame(
    table: &CsvTable,
    domain: &Domain,
    study_id: &str,
) -> Result<MappedDomainFrame> {
    let domain_code = domain.code.to_uppercase();
    let wide_result = match domain_code.as_str() {
        "LB" => build_lb_wide_frame(table, domain, study_id)?,
        "VS" => build_vs_wide_frame(table, domain, study_id)?,
        "IE" => build_ie_wide_frame(table, domain, study_id)?,
        _ => None,
    };
    if let Some((mapping, frame, used_columns)) = wide_result {
        return Ok(MappedDomainFrame {
            mapping,
            frame,
            used_columns,
        });
    }

    let hints = build_column_hints(table);
    let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
    let mapping_result = engine.suggest(&table.headers);
    let mapping = engine.to_config(study_id, mapping_result);
    let frame = build_domain_frame_with_mapping(table, domain, Some(&mapping))?;
    let used_columns = mapping
        .mappings
        .iter()
        .map(|item| item.source_column.clone())
        .collect::<BTreeSet<String>>();
    Ok(MappedDomainFrame {
        mapping,
        frame,
        used_columns,
    })
}
