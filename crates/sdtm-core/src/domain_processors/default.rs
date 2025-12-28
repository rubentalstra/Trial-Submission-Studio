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
    if let Some(epoch_col) = col(domain, "EPOCH")
        && has_column(df, &epoch_col)
    {
        let values = string_column(df, &epoch_col)?;
        let normalized = values
            .into_iter()
            .map(|value| {
                let trimmed = value.trim();
                let upper = trimmed.to_uppercase();
                match upper.as_str() {
                    "" | "UNK" | "UNKNOWN" | "NA" | "N/A" | "NONE" | "NAN" | "<NA>" => {
                        String::new()
                    }
                    _ => trimmed.to_string(),
                }
            })
            .collect();
        set_string_column(df, &epoch_col, normalized)?;
    }
    Ok(())
}
