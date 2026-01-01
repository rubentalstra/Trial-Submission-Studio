//! Optional Polars DataFrame integration.
//!
//! This module provides conversion between XPT datasets and Polars DataFrames.
//! Enable with the `polars` feature.
//!
//! # Reading XPT to DataFrame
//!
//! ```no_run
//! use std::path::Path;
//! use sdtm_xpt::read_xpt_to_dataframe;
//!
//! let df = read_xpt_to_dataframe(Path::new("dm.xpt")).unwrap();
//! println!("{}", df);
//! ```
//!
//! # Writing DataFrame to XPT
//!
//! ```no_run
//! use std::path::Path;
//! use polars::prelude::*;
//! use sdtm_xpt::write_dataframe_to_xpt;
//!
//! let df = df! {
//!     "USUBJID" => &["001", "002", "003"],
//!     "AGE" => &[25i64, 30, 35],
//! }.unwrap();
//!
//! write_dataframe_to_xpt(Path::new("out.xpt"), &df, "DM").unwrap();
//! ```

mod conversion;

use std::path::Path;

use polars::prelude::DataFrame;

use crate::error::Result;
use crate::reader::read_xpt;
use crate::types::{XptDataset, XptWriterOptions};
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
