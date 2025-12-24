use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_ts(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    if df.height() == 0 {
        return Ok(());
    }
    for col_name in [
        "STUDYID", "DOMAIN", "TSPARMCD", "TSPARM", "TSVAL", "TSVALCD", "TSVCDREF", "TSVCDVER",
        "TSGRPID", "TSVALNF",
    ] {
        if let Some(name) = col(domain, col_name) {
            if has_column(df, &name) {
                let values = string_column(df, &name, Trim::Both)?;
                set_string_column(df, &name, values)?;
            }
        }
    }
    if let Some(tsparmcd) = col(domain, "TSPARMCD") {
        if has_column(df, &tsparmcd) {
            if let Some(ct) = ctx.resolve_ct(domain, "TSPARMCD") {
                let mut values = string_column(df, &tsparmcd, Trim::Both)?;
                for idx in 0..values.len() {
                    values[idx] = normalize_ct_value_keep(ct, &values[idx]);
                }
                set_string_column(df, &tsparmcd, values)?;
            }
        }
    }
    if let Some(tsparm) = col(domain, "TSPARM") {
        if has_column(df, &tsparm) {
            if let Some(ct) = ctx.resolve_ct(domain, "TSPARM") {
                let mut values = string_column(df, &tsparm, Trim::Both)?;
                for idx in 0..values.len() {
                    values[idx] = normalize_ct_value_keep(ct, &values[idx]);
                }
                set_string_column(df, &tsparm, values)?;
            }
        }
    }
    if let Some(tsvcdref) = col(domain, "TSVCDREF") {
        if has_column(df, &tsvcdref) {
            if let Some(ct) = ctx.resolve_ct(domain, "TSVCDREF") {
                let mut values = string_column(df, &tsvcdref, Trim::Both)?;
                for idx in 0..values.len() {
                    values[idx] = normalize_ct_value_keep(ct, &values[idx]);
                }
                set_string_column(df, &tsvcdref, values)?;
            }
        }
    }
    Ok(())
}
