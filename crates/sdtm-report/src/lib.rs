use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use polars::prelude::{AnyValue, DataFrame};

use sdtm_core::DomainFrame;
use sdtm_model::{Domain, Variable, VariableType};
use sdtm_xpt::{XptColumn, XptDataset, XptType, XptValue, XptWriterOptions, write_xpt};

const SAS_NUMERIC_LEN: u16 = 8;

pub fn write_xpt_outputs(
    output_dir: &Path,
    domains: &[Domain],
    frames: &[DomainFrame],
    options: &XptWriterOptions,
) -> Result<Vec<PathBuf>> {
    let mut domain_map = BTreeMap::new();
    for domain in domains {
        domain_map.insert(domain.code.to_uppercase(), domain);
    }

    let mut frames_sorted: Vec<&DomainFrame> = frames.iter().collect();
    frames_sorted.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));

    let xpt_dir = output_dir.join("xpt");
    std::fs::create_dir_all(&xpt_dir).with_context(|| format!("create {}", xpt_dir.display()))?;

    let mut outputs = Vec::new();
    for frame in frames_sorted {
        let code = frame.domain_code.to_uppercase();
        let domain = domain_map
            .get(&code)
            .ok_or_else(|| anyhow!("missing domain definition for {code}"))?;
        let dataset = build_xpt_dataset(domain, frame)?;
        let filename = format!("{code}.xpt");
        let path = xpt_dir.join(filename);
        write_xpt(&path, &dataset, options)?;
        outputs.push(path);
    }
    Ok(outputs)
}

pub fn build_xpt_dataset(domain: &Domain, frame: &DomainFrame) -> Result<XptDataset> {
    let df = &frame.data;
    let columns = build_xpt_columns(domain, df)?;
    let rows = build_xpt_rows(domain, df)?;
    Ok(XptDataset {
        name: domain
            .dataset_name
            .clone()
            .unwrap_or_else(|| domain.code.clone())
            .to_uppercase(),
        label: domain.label.clone(),
        columns,
        rows,
    })
}

fn build_xpt_columns(domain: &Domain, df: &DataFrame) -> Result<Vec<XptColumn>> {
    let mut columns = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let length = variable_length(variable, df)?;
        columns.push(XptColumn {
            name: variable.name.clone(),
            label: variable.label.clone(),
            data_type: match variable.data_type {
                VariableType::Num => XptType::Num,
                VariableType::Char => XptType::Char,
            },
            length,
        });
    }
    Ok(columns)
}

fn build_xpt_rows(domain: &Domain, df: &DataFrame) -> Result<Vec<Vec<XptValue>>> {
    let mut series = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let col = df
            .column(variable.name.as_str())
            .with_context(|| format!("missing column {}", variable.name))?;
        series.push(col);
    }

    let row_count = df.height();
    let mut rows = Vec::with_capacity(row_count);
    for row_idx in 0..row_count {
        let mut row = Vec::with_capacity(series.len());
        for (variable, column) in domain.variables.iter().zip(series.iter()) {
            let value = column.get(row_idx).unwrap_or(AnyValue::Null);
            let cell = match variable.data_type {
                VariableType::Num => XptValue::Num(any_to_f64(value)),
                VariableType::Char => XptValue::Char(any_to_string(value)),
            };
            row.push(cell);
        }
        rows.push(row);
    }
    Ok(rows)
}

fn variable_length(variable: &Variable, df: &DataFrame) -> Result<u16> {
    if let Some(length) = variable.length {
        if length == 0 {
            return Err(anyhow!("variable {} has zero length", variable.name));
        }
        return Ok(length.min(u16::MAX as u32) as u16);
    }
    match variable.data_type {
        VariableType::Num => Ok(SAS_NUMERIC_LEN),
        VariableType::Char => {
            let series = df
                .column(variable.name.as_str())
                .with_context(|| format!("missing column {}", variable.name))?;
            let mut max_len = 0usize;
            for idx in 0..df.height() {
                let value = series.get(idx).unwrap_or(AnyValue::Null);
                let text = any_to_string(value);
                let len = text.trim_end().len();
                if len > max_len {
                    max_len = len;
                }
            }
            let len = max_len.max(1);
            if len > u16::MAX as usize {
                return Err(anyhow!("variable {} length too large", variable.name));
            }
            Ok(len as u16)
        }
    }
}

fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::Null => String::new(),
        AnyValue::String(v) => v.to_string(),
        AnyValue::StringOwned(v) => v.to_string(),
        AnyValue::Float64(v) => format_numeric(v),
        AnyValue::Float32(v) => format_numeric(v as f64),
        AnyValue::Int64(v) => v.to_string(),
        AnyValue::Int32(v) => v.to_string(),
        AnyValue::Int16(v) => v.to_string(),
        AnyValue::Int8(v) => v.to_string(),
        AnyValue::UInt64(v) => v.to_string(),
        AnyValue::UInt32(v) => v.to_string(),
        AnyValue::UInt16(v) => v.to_string(),
        AnyValue::UInt8(v) => v.to_string(),
        AnyValue::Boolean(v) => {
            if v {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
        value => value.to_string(),
    }
}

fn any_to_f64(value: AnyValue) -> Option<f64> {
    match value {
        AnyValue::Null => None,
        AnyValue::Float64(v) => Some(v),
        AnyValue::Float32(v) => Some(v as f64),
        AnyValue::Int64(v) => Some(v as f64),
        AnyValue::Int32(v) => Some(v as f64),
        AnyValue::Int16(v) => Some(v as f64),
        AnyValue::Int8(v) => Some(v as f64),
        AnyValue::UInt64(v) => Some(v as f64),
        AnyValue::UInt32(v) => Some(v as f64),
        AnyValue::UInt16(v) => Some(v as f64),
        AnyValue::UInt8(v) => Some(v as f64),
        AnyValue::String(v) => v.trim().parse::<f64>().ok(),
        AnyValue::StringOwned(v) => v.as_str().trim().parse::<f64>().ok(),
        AnyValue::Boolean(v) => Some(if v { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn format_numeric(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}
