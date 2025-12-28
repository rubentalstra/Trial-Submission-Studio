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
        if let Some(name) = col(domain, col_name)
            && has_column(df, &name)
        {
            let values = string_column(df, &name)?;
            set_string_column(df, &name, values)?;
        }
    }
    if let Some(tsparmcd) = col(domain, "TSPARMCD")
        && has_column(df, &tsparmcd)
        && let Some(ct) = ctx.resolve_ct(domain, "TSPARMCD")
    {
        let mut values = string_column(df, &tsparmcd)?;
        for value in &mut values {
            *value = normalize_ct_value_safe(ct, value);
        }
        set_string_column(df, &tsparmcd, values)?;
    }
    if let Some(tsparm) = col(domain, "TSPARM")
        && has_column(df, &tsparm)
        && let Some(ct) = ctx.resolve_ct(domain, "TSPARM")
    {
        let mut values = string_column(df, &tsparm)?;
        for value in &mut values {
            *value = normalize_ct_value_safe(ct, value);
        }
        set_string_column(df, &tsparm, values)?;
    }
    if let Some(tsvcdref) = col(domain, "TSVCDREF")
        && has_column(df, &tsvcdref)
        && let Some(ct) = ctx.resolve_ct(domain, "TSVCDREF")
    {
        let mut values = string_column(df, &tsvcdref)?;
        for value in &mut values {
            *value = normalize_ct_value_safe(ct, value);
        }
        set_string_column(df, &tsvcdref, values)?;
    }
    Ok(())
}
