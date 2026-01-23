//! XPT (SAS Transport) output generation.

use std::path::{Path, PathBuf};

use polars::prelude::{AnyValue, DataFrame};

use super::types::{DomainFrame, domain_map_by_code};
use crate::error::{Result, SubmitError};
use tss_standards::{SdtmDomain, VariableType};
use tss_standards::{any_to_f64, any_to_string};
use xportrs::{Column, ColumnData, Dataset, Xpt};

use super::common::{ensure_output_dir, variable_length};

/// Write XPT outputs for all domains.
pub fn write_xpt_outputs(
    output_dir: &Path,
    domains: &[SdtmDomain],
    frames: &[DomainFrame],
) -> Result<Vec<PathBuf>> {
    let domain_lookup = domain_map_by_code(domains);
    let mut frames_sorted: Vec<&DomainFrame> = frames.iter().collect();
    frames_sorted.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));

    let xpt_dir = ensure_output_dir(output_dir, "xpt")?;

    let mut outputs = Vec::new();
    for frame in frames_sorted {
        let code = frame.domain_code.to_uppercase();
        let domain = domain_lookup
            .get(&code)
            .ok_or_else(|| SubmitError::MissingDomain {
                domain: code.clone(),
            })?;
        // Use frame's dataset name (from metadata) for split domains, falling back to domain.name
        let output_dataset_name = frame.dataset_name();
        let dataset = build_xpt_dataset_with_name(domain, frame, &output_dataset_name)?;
        let disk_name = output_dataset_name.to_lowercase();
        let filename = format!("{disk_name}.xpt");
        let path = xpt_dir.join(&filename);

        // Write using xportrs builder pattern
        Xpt::writer(dataset)
            .finalize()
            .map_err(|e| SubmitError::XptValidation {
                dataset: filename.clone(),
                message: e.to_string(),
            })?
            .write_path(&path)
            .map_err(|e| SubmitError::write_error("XPT", path.display().to_string(), e))?;

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
    domain: &SdtmDomain,
    frame: &DomainFrame,
    dataset_name: &str,
) -> Result<Dataset> {
    let df = &frame.data;
    let columns = build_xpt_columns(domain, df)?;

    // Use domain label if available, otherwise use domain name
    let dataset_label = domain.label.as_deref().unwrap_or(&domain.name);

    Dataset::with_label(dataset_name, dataset_label, columns).map_err(|e| {
        SubmitError::XptValidation {
            dataset: dataset_name.to_string(),
            message: e.to_string(),
        }
    })
}

/// Build XPT columns from domain variables.
fn build_xpt_columns(domain: &SdtmDomain, df: &DataFrame) -> Result<Vec<Column>> {
    // Filter to only variables that exist in the DataFrame
    let existing_vars: Vec<_> = domain
        .variables
        .iter()
        .filter(|v| df.column(&v.name).is_ok())
        .collect();

    let row_count = df.height();
    let mut columns = Vec::with_capacity(existing_vars.len());

    for variable in &existing_vars {
        let col = df
            .column(variable.name.as_str())
            .map_err(|_| SubmitError::ColumnNotFound {
                column: variable.name.clone(),
            })?;

        let column_data = match variable.data_type {
            VariableType::Num => {
                let mut values = Vec::with_capacity(row_count);
                for row_idx in 0..row_count {
                    let value = col.get(row_idx).unwrap_or(AnyValue::Null);
                    let num = any_to_f64(value);
                    values.push(num);
                }
                ColumnData::F64(values)
            }
            VariableType::Char => {
                let mut values = Vec::with_capacity(row_count);
                for row_idx in 0..row_count {
                    let value = col.get(row_idx).unwrap_or(AnyValue::Null);
                    let s = any_to_string(value);
                    values.push(if s.is_empty() { None } else { Some(s) });
                }
                ColumnData::String(values)
            }
        };

        // Create column with name and data
        let mut column = Column::new(&variable.name, column_data);

        // Set label if available (required for FDA submissions)
        if let Some(label) = &variable.label {
            column = column.with_label(label.as_str());
        }

        // Set explicit length for all columns
        match variable.data_type {
            VariableType::Char => {
                let length = variable_length(variable, df)?;
                column = column.with_length(length as usize);
            }
            VariableType::Num => {
                // Numeric columns should always be 8 bytes in SAS XPT format
                column = column.with_length(8);
            }
        }

        columns.push(column);
    }

    Ok(columns)
}
