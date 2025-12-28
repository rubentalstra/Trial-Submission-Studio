use anyhow::Result;
use polars::prelude::{DataFrame, UInt32Chunked};
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_ta(
    domain: &Domain,
    df: &mut DataFrame,
    _context: &PipelineContext,
) -> Result<()> {
    for col_name in ["EPOCH", "ARMCD", "ARM", "ETCD", "STUDYID", "DOMAIN"] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, &name)
        {
            let values = string_column(df, &name)?;
            set_string_column(df, &name, values)?;
        }
    }
    if let Some(taetord) = col(domain, "TAETORD")
        && has_column(df, &taetord)
    {
        let values = numeric_column_f64(df, &taetord)?;
        set_f64_column(df, &taetord, values)?;
        sort_by_numeric(df, &taetord)?;
    }
    Ok(())
}

fn sort_by_numeric(df: &mut DataFrame, column: &str) -> Result<()> {
    let values = numeric_column_f64(df, column)?;
    let mut indices: Vec<u32> = (0..df.height()).map(|idx| idx as u32).collect();
    indices.sort_by(|a, b| {
        let left = values[*a as usize];
        let right = values[*b as usize];
        left.partial_cmp(&right)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let idx = UInt32Chunked::from_vec("idx".into(), indices);
    *df = df.take(&idx)?;
    Ok(())
}
