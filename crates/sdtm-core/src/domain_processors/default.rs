use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_default(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, context)?;
    if let Some(epoch_col) = col(domain, "EPOCH")
        && has_column(df, &epoch_col)
    {
        let values = string_column(df, &epoch_col)?;
        let normalized = values
            .into_iter()
            .map(|value| replace_unknown(&value, ""))
            .collect();
        set_string_column(df, &epoch_col, normalized)?;
    }
    Ok(())
}
