use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_ex(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if let Some(extrt) = col(domain, "EXTRT")
        && has_column(df, extrt)
    {
        let values = string_column(df, extrt)?;
        set_string_column(df, extrt, values)?;
    }
    if let (Some(extrt), Some(exeltm)) = (col(domain, "EXTRT"), col(domain, "EXELTM"))
        && has_column(df, extrt)
        && has_column(df, exeltm)
    {
        let mut extrt_vals = string_column(df, extrt)?;
        let mut exeltm_vals = string_column(df, exeltm)?;
        for idx in 0..df.height() {
            let has_letters = exeltm_vals[idx].chars().any(|c| c.is_ascii_alphabetic());
            if extrt_vals[idx].is_empty() && !exeltm_vals[idx].is_empty() && has_letters {
                extrt_vals[idx] = exeltm_vals[idx].clone();
                exeltm_vals[idx].clear();
            }
        }
        set_string_column(df, extrt, extrt_vals)?;
        set_string_column(df, exeltm, exeltm_vals)?;
    }
    for date_col in ["EXSTDTC", "EXENDTC"] {
        if let Some(name) = col(domain, date_col)
            && has_column(df, name)
        {
            let values = string_column(df, name)?
                .into_iter()
                .map(|value| normalize_iso8601(&value))
                .collect();
            set_string_column(df, name, values)?;
        }
    }
    if let (Some(exstdtc), Some(exstdy)) = (col(domain, "EXSTDTC"), col(domain, "EXSTDY")) {
        compute_study_day(domain, df, exstdtc, exstdy, context, "RFSTDTC")?;
    }
    if let (Some(exendtc), Some(exendy)) = (col(domain, "EXENDTC"), col(domain, "EXENDY")) {
        compute_study_day(domain, df, exendtc, exendy, context, "RFSTDTC")?;
    }
    if let Some(exdose) = col(domain, "EXDOSE")
        && has_column(df, exdose)
    {
        let values = numeric_column_f64(df, exdose)?;
        set_f64_column(df, exdose, values)?;
    }
    for col_name in [
        "EXDOSFRM", "EXDOSU", "EXDOSFRQ", "EXDUR", "EXSCAT", "EXCAT", "EPOCH", "EXELTM",
        "EXTPTREF", "EXRFTDTC",
    ] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, name)
        {
            let values = string_column(df, name)?;
            set_string_column(df, name, values)?;
        }
    }
    for col_name in ["EXSTDY", "EXENDY"] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, name)
        {
            let values = numeric_column_f64(df, name)?;
            set_f64_column(df, name, values)?;
        }
    }
    Ok(())
}
