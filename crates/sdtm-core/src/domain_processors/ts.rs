//! Trial Summary (TS) domain processor.
//!
//! Processes TS domain data per SDTMIG v3.4 Section 7.3.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{col, has_column, normalize_ct_value, set_string_column, string_column};

pub(super) fn process_ts(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if df.height() == 0 {
        return Ok(());
    }

    // Normalize CT columns that have direct codelist mappings
    for ct_col in ["TSPARMCD", "TSPARM", "TSVCDREF"] {
        normalize_ts_ct_column(domain, df, context, ct_col)?;
    }

    Ok(())
}

/// Normalize a TS column using controlled terminology.
fn normalize_ts_ct_column(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
    col_name: &str,
) -> Result<()> {
    let Some(col_ref) = col(domain, col_name) else {
        return Ok(());
    };
    if !has_column(df, col_ref) {
        return Ok(());
    }
    let Some(ct) = context.resolve_ct(domain, col_name) else {
        return Ok(());
    };

    let mut values = string_column(df, col_ref)?;
    for value in &mut values {
        *value = normalize_ct_value(ct, value, &context.options.normalization);
    }
    set_string_column(df, col_ref, values)
}
