use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_default(domain: &Domain, df: &mut DataFrame, ctx: &ProcessingContext) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    if let Some(epoch_col) = col(domain, "EPOCH") {
        if has_column(df, &epoch_col) {
            let values = string_column(df, &epoch_col, Trim::Both)?;
            let normalized = values
                .into_iter()
                .map(|value| replace_unknown(&value, ""))
                .collect();
            set_string_column(df, &epoch_col, normalized)?;
        }
    }
    Ok(())
}
