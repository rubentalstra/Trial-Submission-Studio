//! Drug Accountability (DA) domain processor.
//!
//! Processes DA domain data per SDTMIG v3.4 Section 6.3.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    apply_map_upper, backward_fill_var, clear_unit_when_empty_var, col, compute_study_days_batch,
    has_column, map_values, normalize_ct_batch, numeric_column_f64, parse_f64, set_f64_column,
    set_string_column, string_column,
};

pub(super) fn process_da(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Apply DASTAT mapping
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

    // Set DASTAT to "NOT DONE" when DAREASND is populated
    set_dastat_from_reason(domain, df)?;

    // Clear unit when result is empty
    clear_unit_when_empty_var(domain, df, "DAORRES", "DAORRESU")?;

    // Backward fill: DAORRES → DASTRESC
    backward_fill_var(domain, df, "DAORRES", "DASTRESC")?;

    // Derive numeric result from DASTRESC
    derive_dastresn(domain, df)?;

    // Compute study day
    compute_study_days_batch(domain, df, context, &[("DADTC", "DADY")])?;

    // Batch CT normalization
    normalize_ct_batch(
        domain,
        df,
        context,
        &[
            "DASTAT", "DATESTCD", "DATEST", "DACAT", "DASCAT", "DAORRESU", "DASTRESU", "EPOCH",
        ],
    )?;

    Ok(())
}

/// Set DASTAT to "NOT DONE" when DAREASND is populated.
fn set_dastat_from_reason(domain: &Domain, df: &mut DataFrame) -> Result<()> {
    let (Some(dastat), Some(dareasnd)) = (col(domain, "DASTAT"), col(domain, "DAREASND")) else {
        return Ok(());
    };
    if !has_column(df, dastat) || !has_column(df, dareasnd) {
        return Ok(());
    }

    let reason_vals = string_column(df, dareasnd)?;
    let mut stat_vals = string_column(df, dastat)?;
    for idx in 0..df.height() {
        if stat_vals[idx].is_empty() && !reason_vals[idx].is_empty() {
            stat_vals[idx] = "NOT DONE".to_string();
        }
    }
    set_string_column(df, dastat, stat_vals)
}

/// Derive numeric result from DASTRESC.
fn derive_dastresn(domain: &Domain, df: &mut DataFrame) -> Result<()> {
    let (Some(dastresc), Some(dastresn)) = (col(domain, "DASTRESC"), col(domain, "DASTRESN"))
    else {
        return Ok(());
    };
    if !has_column(df, dastresc) {
        return Ok(());
    }

    let stresc = string_column(df, dastresc)?;
    let mut stresn_vals: Vec<Option<f64>> = if has_column(df, dastresn) {
        numeric_column_f64(df, dastresn)?
    } else {
        vec![None; df.height()]
    };

    for (idx, value) in stresc.iter().enumerate() {
        if stresn_vals[idx].is_none() {
            if let Some(parsed) = parse_f64(value) {
                stresn_vals[idx] = Some(parsed);
            }
        }
        if !value.is_empty() && parse_f64(value).is_none() {
            stresn_vals[idx] = None;
        }
    }
    set_f64_column(df, dastresn, stresn_vals)
}
