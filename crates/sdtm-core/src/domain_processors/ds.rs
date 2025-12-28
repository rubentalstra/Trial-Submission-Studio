use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_ds(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    for col_name in ["DSDECOD", "DSTERM", "DSCAT", "EPOCH"] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, name)
        {
            let values = string_column(df, name)?;
            set_string_column(df, name, values)?;
        }
    }
    if let Some(dsdecod) = col(domain, "DSDECOD")
        && has_column(df, dsdecod)
        && let Some(ct) = context.resolve_ct(domain, "DSDECOD")
    {
        let values = string_column(df, dsdecod)?
            .into_iter()
            .map(|value| normalize_ct_value(ct, &value, context.options.ct_matching))
            .collect();
        set_string_column(df, dsdecod, values)?;
    }
    if let Some(dsstdtc) = col(domain, "DSSTDTC")
        && has_column(df, dsstdtc)
    {
        let values = string_column(df, dsstdtc)?
            .into_iter()
            .map(|value| normalize_iso8601(&value))
            .collect();
        set_string_column(df, dsstdtc, values)?;
        if let Some(dsstudy) = col(domain, "DSSTDY") {
            compute_study_day(domain, df, dsstdtc, dsstudy, context, "RFSTDTC")?;
        }
    }
    if let Some(dsdtc) = col(domain, "DSDTC")
        && has_column(df, dsdtc)
    {
        let values = string_column(df, dsdtc)?
            .into_iter()
            .map(|value| normalize_iso8601(&value))
            .collect();
        set_string_column(df, dsdtc, values)?;
        if let Some(dsdy) = col(domain, "DSDY") {
            compute_study_day(domain, df, dsdtc, dsdy, context, "RFSTDTC")?;
        }
    }
    Ok(())
}
