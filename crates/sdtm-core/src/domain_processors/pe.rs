use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_pe(domain: &Domain, df: &mut DataFrame, ctx: &ProcessingContext) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    if let (Some(peseq), Some(usubjid)) = (col(domain, "PESEQ"), col(domain, "USUBJID")) {
        assign_sequence(df, &peseq, &usubjid)?;
    }
    if let Some(pestat) = col(domain, "PESTAT") {
        let stat_map = map_values([
            ("NOT DONE", "NOT DONE"),
            ("ND", "NOT DONE"),
            ("DONE", ""),
            ("COMPLETED", ""),
            ("", ""),
            ("NAN", ""),
        ]);
        apply_map_upper(df, Some(&pestat), &stat_map)?;
    }
    if let (Some(peorres), Some(pestresc)) = (col(domain, "PEORRES"), col(domain, "PESTRESC")) {
        if has_column(df, &peorres) && has_column(df, &pestresc) {
            let orres = string_column(df, &peorres, Trim::Both)?;
            let mut stresc = string_column(df, &pestresc, Trim::Both)?;
            for idx in 0..df.height() {
                if stresc[idx].is_empty() && !orres[idx].is_empty() {
                    stresc[idx] = orres[idx].clone();
                }
            }
            set_string_column(df, &pestresc, stresc)?;
        }
    }
    if let Some(pedtc) = col(domain, "PEDTC") {
        if let Some(pedy) = col(domain, "PEDY") {
            compute_study_day(domain, df, &pedtc, &pedy, ctx, "RFSTDTC")?;
        }
    }
    if let Some(epoch) = col(domain, "EPOCH") {
        if has_column(df, &epoch) {
            let values = string_column(df, &epoch, Trim::Both)?;
            set_string_column(df, &epoch, values)?;
        }
    }
    Ok(())
}
