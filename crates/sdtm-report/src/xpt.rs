//! XPT (SAS Transport) output generation.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use polars::prelude::{AnyValue, DataFrame};

use sdtm_transform::frame::DomainFrame;
use sdtm_ingest::{any_to_f64, any_to_string};
use sdtm_model::{Domain, VariableType};
use sdtm_xpt::{XptColumn, XptDataset, XptType, XptValue, XptWriterOptions, write_xpt};

use crate::common::variable_length;

/// Write XPT outputs for all domains.
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
        // Use frame's dataset name (from metadata) for split domains, falling back to domain.code
        let output_dataset_name = frame.dataset_name();
        let dataset = build_xpt_dataset_with_name(domain, frame, &output_dataset_name)?;
        let disk_name = output_dataset_name.to_lowercase();
        let filename = format!("{disk_name}.xpt");
        let path = xpt_dir.join(filename);
        write_xpt(&path, &dataset, options)?;
        outputs.push(path);
    }
    Ok(outputs)
}

/// Build XPT dataset with an explicit dataset name.
///
/// This variant allows specifying the dataset name directly, useful for:
/// - Split domains (e.g., LBCH, FAAE) where the name comes from frame metadata
/// - Custom output naming requirements
pub fn build_xpt_dataset_with_name(
    domain: &Domain,
    frame: &DomainFrame,
    dataset_name: &str,
) -> Result<XptDataset> {
    let df = &frame.data;
    let columns = build_xpt_columns(domain, df)?;
    let rows = build_xpt_rows(domain, df)?;
    Ok(XptDataset {
        name: dataset_name.to_uppercase(),
        label: domain.label.clone(),
        columns,
        rows,
    })
}

/// Build XPT columns from domain variables.
fn build_xpt_columns(domain: &Domain, df: &DataFrame) -> Result<Vec<XptColumn>> {
    let mut columns = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let length = variable_length(variable, df)?;
        columns.push(XptColumn {
            name: variable.name.clone(),
            label: variable.label.clone(),
            data_type: match variable.data_type {
                VariableType::Num => XptType::Num,
                // Treat Char and future types as Char
                VariableType::Char | _ => XptType::Char,
            },
            length,
        });
    }
    Ok(columns)
}

/// Build XPT rows from DataFrame.
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
                // Treat Char and future types as Char
                VariableType::Char | _ => XptValue::Char(any_to_string(value)),
            };
            row.push(cell);
        }
        rows.push(row);
    }
    Ok(rows)
}
