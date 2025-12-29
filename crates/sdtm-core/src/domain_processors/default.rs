use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_default(
    domain: &Domain,
    df: &mut DataFrame,
    _context: &PipelineContext,
) -> Result<()> {
    // Clean NA-like values from EPOCH column
    if let Some(epoch_col) = col(domain, "EPOCH")
        && has_column(df, epoch_col)
    {
        clean_na_values(df, epoch_col)?;
    }
    Ok(())
}
