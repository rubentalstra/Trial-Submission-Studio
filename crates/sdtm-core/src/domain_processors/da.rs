use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_da(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if let Some(dastat) = col(domain, "DASTAT") {
        let stat_map = map_values([
            ("NOT DONE", "NOT DONE"),
            ("ND", "NOT DONE"),
            ("DONE", ""),
            ("COMPLETED", ""),
            ("", ""),
            ("NAN", ""),
        ]);
        apply_map_upper(df, Some(dastat), &stat_map)?;
    }
    if let (Some(dastat), Some(dareasnd)) = (col(domain, "DASTAT"), col(domain, "DAREASND"))
        && has_column(df, dastat)
        && has_column(df, dareasnd)
    {
        let reason_vals = string_column(df, dareasnd)?;
        let mut stat_vals = string_column(df, dastat)?;
        for idx in 0..df.height() {
            if stat_vals[idx].is_empty() && !reason_vals[idx].is_empty() {
                stat_vals[idx] = "NOT DONE".to_string();
            }
        }
        set_string_column(df, dastat, stat_vals)?;
    }
    if let (Some(daorresu), Some(daorres)) = (col(domain, "DAORRESU"), col(domain, "DAORRES"))
        && has_column(df, daorresu)
        && has_column(df, daorres)
    {
        let orres = string_column(df, daorres)?;
        let mut orresu = string_column(df, daorresu)?;
        for (idx, value) in orres.iter().enumerate() {
            if value.is_empty() {
                orresu[idx].clear();
            }
        }
        set_string_column(df, daorresu, orresu)?;
    }
    if let (Some(daorres), Some(dastresc)) = (col(domain, "DAORRES"), col(domain, "DASTRESC"))
        && has_column(df, daorres)
        && has_column(df, dastresc)
    {
        let orres = string_column(df, daorres)?;
        let mut stresc = string_column(df, dastresc)?;
        for (idx, value) in orres.iter().enumerate() {
            if !value.is_empty() && stresc[idx].is_empty() {
                stresc[idx] = value.clone();
            }
        }
        set_string_column(df, dastresc, stresc)?;
    }
    if let (Some(dastresc), Some(dastresn)) = (col(domain, "DASTRESC"), col(domain, "DASTRESN"))
        && has_column(df, dastresc)
    {
        let stresc = string_column(df, dastresc)?;
        let mut stresn_vals: Vec<Option<f64>> = vec![None; df.height()];
        if has_column(df, dastresn) {
            stresn_vals = numeric_column_f64(df, dastresn)?;
        }
        for (idx, value) in stresc.iter().enumerate() {
            if stresn_vals[idx].is_none()
                && let Some(parsed) = parse_f64(value)
            {
                stresn_vals[idx] = Some(parsed);
            }
            if !value.is_empty() && parse_f64(value).is_none() {
                stresn_vals[idx] = None;
            }
        }
        set_f64_column(df, dastresn, stresn_vals)?;
    }
    if let Some(dadtc) = col(domain, "DADTC")
        && has_column(df, dadtc)
        && let Some(dady) = col(domain, "DADY")
    {
        compute_study_day(domain, df, dadtc, dady, context, "RFSTDTC")?;
    }
    Ok(())
}
