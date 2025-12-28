use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_qs(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    for col_name in [
        "QSTESTCD", "QSTEST", "QSCAT", "QSSCAT", "QSORRES", "QSSTRESC", "QSLOBXFL", "VISIT",
        "EPOCH",
    ] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, &name)
        {
            let values = string_column(df, &name)?;
            set_string_column(df, &name, values)?;
        }
    }

    if let (Some(qsstresc), Some(qsorres)) = (col(domain, "QSSTRESC"), col(domain, "QSORRES"))
        && has_column(df, &qsstresc)
        && has_column(df, &qsorres)
    {
        let orres = string_column(df, &qsorres)?;
        let mut stresc = string_column(df, &qsstresc)?;
        for idx in 0..df.height() {
            if stresc[idx].is_empty() {
                stresc[idx] = orres[idx].clone();
            }
        }
        set_string_column(df, &qsstresc, stresc)?;
    }
    if let Some(qslobxfl) = col(domain, "QSLOBXFL")
        && has_column(df, &qslobxfl)
    {
        let values = string_column(df, &qslobxfl)?
            .into_iter()
            .map(|value| if value == "N" { "".to_string() } else { value })
            .collect();
        set_string_column(df, &qslobxfl, values)?;
    }
    if let Some(qsdtc) = col(domain, "QSDTC")
        && has_column(df, &qsdtc)
    {
        let values = string_column(df, &qsdtc)?
            .into_iter()
            .map(|value| normalize_iso8601(&value))
            .collect();
        set_string_column(df, &qsdtc, values)?;
        if let Some(qsdy) = col(domain, "QSDY") {
            compute_study_day(domain, df, &qsdtc, &qsdy, context, "RFSTDTC")?;
        }
    }
    if let Some(qstptref) = col(domain, "QSTPTREF")
        && has_column(df, &qstptref)
    {
        let has_timing = ["QSELTM", "QSTPTNUM", "QSTPT"]
            .into_iter()
            .filter_map(|name| col(domain, name))
            .any(|name| has_column(df, &name));
        if !has_timing {
            let values = vec![String::new(); df.height()];
            set_string_column(df, &qstptref, values)?;
        }
    }
    Ok(())
}
