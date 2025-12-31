//! Optional Polars DataFrame integration.
//!
//! This module provides conversion between XPT datasets and Polars DataFrames.
//! Enable with the `polars` feature.

use std::path::Path;

use polars::prelude::*;

use crate::error::{Result, XptError};
use crate::reader::read_xpt;
use crate::types::{NumericValue, XptColumn, XptDataset, XptType, XptValue, XptWriterOptions};
use crate::writer::write_xpt_with_options;

/// Read an XPT file directly to a Polars DataFrame.
///
/// # Arguments
/// * `path` - Path to the XPT file
///
/// # Returns
/// A Polars DataFrame containing the XPT data.
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use sdtm_xpt::read_xpt_to_dataframe;
///
/// let df = read_xpt_to_dataframe(Path::new("dm.xpt")).unwrap();
/// println!("{}", df);
/// ```
pub fn read_xpt_to_dataframe(path: &Path) -> Result<DataFrame> {
    let dataset = read_xpt(path)?;
    dataset.to_dataframe()
}

/// Write a Polars DataFrame to an XPT file.
///
/// # Arguments
/// * `path` - Output path for the XPT file
/// * `df` - The DataFrame to write
/// * `name` - Dataset name (1-8 characters)
///
/// # Returns
/// Ok(()) on success.
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use polars::prelude::*;
/// use sdtm_xpt::write_dataframe_to_xpt;
///
/// let df = df! {
///     "USUBJID" => &["001", "002", "003"],
///     "AGE" => &[25i64, 30, 35],
/// }.unwrap();
///
/// write_dataframe_to_xpt(Path::new("out.xpt"), &df, "DM").unwrap();
/// ```
pub fn write_dataframe_to_xpt(path: &Path, df: &DataFrame, name: &str) -> Result<()> {
    let dataset = XptDataset::from_dataframe(df, name)?;
    write_xpt_with_options(path, &dataset, &XptWriterOptions::default())
}

impl XptDataset {
    /// Convert this XPT dataset to a Polars DataFrame.
    ///
    /// - Numeric columns become Float64 (with None for missing values)
    /// - Character columns become String
    pub fn to_dataframe(&self) -> Result<DataFrame> {
        let mut series_vec = Vec::with_capacity(self.columns.len());

        for (col_idx, column) in self.columns.iter().enumerate() {
            let series = match column.data_type {
                XptType::Num => {
                    let values: Vec<Option<f64>> = self
                        .rows
                        .iter()
                        .map(|row| row.get(col_idx).and_then(|v| v.as_f64()))
                        .collect();
                    Series::new(column.name.clone().into(), values)
                }
                XptType::Char => {
                    let values: Vec<Option<String>> = self
                        .rows
                        .iter()
                        .map(|row| {
                            row.get(col_idx).and_then(|v| {
                                if let XptValue::Char(s) = v {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                        })
                        .collect();
                    Series::new(column.name.clone().into(), values)
                }
            };
            series_vec.push(series.into());
        }

        DataFrame::new(series_vec).map_err(|e| XptError::invalid_format(e.to_string()))
    }

    /// Create an XPT dataset from a Polars DataFrame.
    ///
    /// # Arguments
    /// * `df` - The DataFrame to convert
    /// * `name` - Dataset name (1-8 characters)
    ///
    /// # Column Type Mapping
    /// - Float32/Float64 → Numeric (length 8)
    /// - Int8/16/32/64/UInt8/16/32/64 → Numeric (length 8)
    /// - String/Categorical → Character (length = max string length, min 1)
    /// - Boolean → Character (length 1, "Y"/"N")
    /// - Other → Character (converted to string)
    pub fn from_dataframe(df: &DataFrame, name: &str) -> Result<Self> {
        let mut columns = Vec::with_capacity(df.width());
        let mut rows: Vec<Vec<XptValue>> = vec![Vec::with_capacity(df.width()); df.height()];

        for column in df.get_columns() {
            let col_name = column.name().to_string();
            let series = column.as_materialized_series();

            let (xpt_col, col_values) = series_to_xpt_column(series, &col_name)?;
            columns.push(xpt_col);

            for (row_idx, value) in col_values.into_iter().enumerate() {
                if row_idx < rows.len() {
                    rows[row_idx].push(value);
                }
            }
        }

        let mut dataset = XptDataset::new(name);
        dataset.columns = columns;
        dataset.rows = rows;

        Ok(dataset)
    }
}

/// Convert a Polars Series to an XPT column and values.
fn series_to_xpt_column(series: &Series, name: &str) -> Result<(XptColumn, Vec<XptValue>)> {
    let dtype = series.dtype();

    match dtype {
        DataType::Float32 | DataType::Float64 => {
            let col = XptColumn::numeric(name);
            let values = float_series_to_values(series);
            Ok((col, values))
        }
        DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64 => {
            let col = XptColumn::numeric(name);
            let values = int_series_to_values(series);
            Ok((col, values))
        }
        DataType::Boolean => {
            let col = XptColumn::character(name, 1);
            let values = bool_series_to_values(series);
            Ok((col, values))
        }
        DataType::String | DataType::Categorical(_, _) => {
            let (col, values) = string_series_to_column(series, name);
            Ok((col, values))
        }
        _ => {
            // Fallback: convert to string
            let str_series = series
                .cast(&DataType::String)
                .map_err(|e| XptError::invalid_format(e.to_string()))?;
            let (col, values) = string_series_to_column(&str_series, name);
            Ok((col, values))
        }
    }
}

/// Convert float series to XPT values.
fn float_series_to_values(series: &Series) -> Vec<XptValue> {
    // Cast to Float64 if needed, then iterate
    let float_series = if series.dtype() == &DataType::Float64 {
        series.clone()
    } else {
        series.cast(&DataType::Float64).unwrap()
    };

    let ca = float_series.f64().unwrap();
    ca.into_iter()
        .map(|opt| match opt {
            Some(v) if v.is_finite() => XptValue::Num(NumericValue::Value(v)),
            _ => XptValue::numeric_missing(),
        })
        .collect()
}

/// Convert integer series to XPT values.
fn int_series_to_values(series: &Series) -> Vec<XptValue> {
    let ca = series.cast(&DataType::Float64).unwrap();
    float_series_to_values(&ca)
}

/// Convert boolean series to XPT character values.
fn bool_series_to_values(series: &Series) -> Vec<XptValue> {
    let ca = series.bool().unwrap();
    ca.into_iter()
        .map(|opt| match opt {
            Some(true) => XptValue::character("Y"),
            Some(false) => XptValue::character("N"),
            None => XptValue::character(""),
        })
        .collect()
}

/// Convert string series to XPT column and values.
fn string_series_to_column(series: &Series, name: &str) -> (XptColumn, Vec<XptValue>) {
    // Cast to String if needed
    let str_series = if series.dtype() == &DataType::String {
        series.clone()
    } else {
        series.cast(&DataType::String).unwrap()
    };

    let ca = str_series.str().unwrap();

    // Find max string length
    let max_len = ca
        .into_iter()
        .filter_map(|opt| opt.map(|s| s.len()))
        .max()
        .unwrap_or(1)
        .max(1) as u16;

    let col = XptColumn::character(name, max_len.min(200)); // XPT max char length is 200

    let values: Vec<XptValue> = ca
        .into_iter()
        .map(|opt| XptValue::character(opt.unwrap_or("")))
        .collect();

    (col, values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_to_dataframe() {
        let mut ds = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::character("NAME", 10)],
        );
        ds.add_row(vec![XptValue::numeric(25.0), XptValue::character("Alice")]);
        ds.add_row(vec![XptValue::numeric(30.0), XptValue::character("Bob")]);

        let df = ds.to_dataframe().unwrap();

        assert_eq!(df.width(), 2);
        assert_eq!(df.height(), 2);

        let age = df.column("AGE").unwrap().f64().unwrap();
        assert_eq!(age.get(0), Some(25.0));
        assert_eq!(age.get(1), Some(30.0));

        let name = df.column("NAME").unwrap().str().unwrap();
        assert_eq!(name.get(0), Some("Alice"));
        assert_eq!(name.get(1), Some("Bob"));
    }

    #[test]
    fn test_dataframe_to_dataset() {
        let df = df! {
            "USUBJID" => &["001", "002"],
            "AGE" => &[25.0f64, 30.0],
        }
        .unwrap();

        let ds = XptDataset::from_dataframe(&df, "DM").unwrap();

        assert_eq!(ds.name, "DM");
        assert_eq!(ds.num_columns(), 2);
        assert_eq!(ds.num_rows(), 2);

        assert_eq!(ds.columns[0].name, "USUBJID");
        assert!(ds.columns[0].is_character());

        assert_eq!(ds.columns[1].name, "AGE");
        assert!(ds.columns[1].is_numeric());
    }

    #[test]
    fn test_roundtrip_numeric() {
        let mut ds1 = XptDataset::with_columns("TEST", vec![XptColumn::numeric("X")]);
        ds1.add_row(vec![XptValue::numeric(1.5)]);
        ds1.add_row(vec![XptValue::numeric_missing()]);
        ds1.add_row(vec![XptValue::numeric(-999.0)]);

        let df = ds1.to_dataframe().unwrap();
        let ds2 = XptDataset::from_dataframe(&df, "TEST").unwrap();

        assert_eq!(ds2.num_rows(), 3);
        assert_eq!(ds2.value(0, 0).unwrap().as_f64(), Some(1.5));
        assert!(ds2.value(1, 0).unwrap().is_missing());
        assert_eq!(ds2.value(2, 0).unwrap().as_f64(), Some(-999.0));
    }

    #[test]
    fn test_integer_series() {
        let df = df! {
            "COUNT" => &[1i64, 2, 3],
        }
        .unwrap();

        let ds = XptDataset::from_dataframe(&df, "TEST").unwrap();

        assert!(ds.columns[0].is_numeric());
        assert_eq!(ds.value(0, 0).unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_boolean_series() {
        let df = df! {
            "FLAG" => &[true, false, true],
        }
        .unwrap();

        let ds = XptDataset::from_dataframe(&df, "TEST").unwrap();

        assert!(ds.columns[0].is_character());
        assert_eq!(ds.value(0, 0).unwrap().as_str(), Some("Y"));
        assert_eq!(ds.value(1, 0).unwrap().as_str(), Some("N"));
    }
}
