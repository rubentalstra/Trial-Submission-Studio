//! Trial Arms (TA) domain processor.
//!
//! Processes TA domain data per SDTMIG v3.4 Section 7.2.

use anyhow::Result;
use polars::prelude::{DataFrame, UInt32Chunked};
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{col, has_column, normalize_numeric_f64, numeric_column_f64};

pub(super) fn process_ta(
    domain: &Domain,
    df: &mut DataFrame,
    _context: &PipelineContext,
) -> Result<()> {
    // Normalize and sort by TAETORD
    normalize_numeric_f64(domain, df, &["TAETORD"])?;
    if let Some(taetord) = col(domain, "TAETORD")
        && has_column(df, taetord)
    {
        sort_by_numeric(df, taetord)?;
    }
    Ok(())
}

/// Sort DataFrame by a numeric column.
fn sort_by_numeric(df: &mut DataFrame, column: &str) -> Result<()> {
    let values = numeric_column_f64(df, column)?;
    let mut indices: Vec<u32> = (0..df.height()).map(|idx| idx as u32).collect();
    indices.sort_by(|a, b| {
        let left = values[*a as usize];
        let right = values[*b as usize];
        left.partial_cmp(&right).unwrap_or(std::cmp::Ordering::Equal)
    });
    let idx = UInt32Chunked::from_vec("idx".into(), indices);
    *df = df.take(&idx)?;
    Ok(())
}
