//! Exposure (EX) domain processor.
//!
//! Processes EX domain data per SDTMIG v3.4 Section 6.1.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    col, compute_study_days_batch, has_column, normalize_ct_batch, normalize_iso8601,
    normalize_numeric_f64, set_string_column, string_column, trim_columns,
};

pub(super) fn process_ex(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Trim EXTRT
    trim_columns(domain, df, &["EXTRT"])?;

    // Move treatment name from EXELTM to EXTRT if EXTRT is empty
    relocate_extrt_from_exeltm(domain, df)?;

    // Normalize date columns
    for date_col in ["EXSTDTC", "EXENDTC"] {
        if let Some(name) = col(domain, date_col)
            && has_column(df, name)
        {
            let values = string_column(df, name)?
                .into_iter()
                .map(|v| normalize_iso8601(&v))
                .collect();
            set_string_column(df, name, values)?;
        }
    }

    // Compute study days
    compute_study_days_batch(
        domain,
        df,
        context,
        &[("EXSTDTC", "EXSTDY"), ("EXENDTC", "EXENDY")],
    )?;

    // Normalize EXDOSE as numeric
    normalize_numeric_f64(domain, df, &["EXDOSE"])?;

    // Trim other string columns
    trim_columns(
        domain,
        df,
        &[
            "EXDOSFRM", "EXDOSU", "EXDOSFRQ", "EXDUR", "EXSCAT", "EXCAT", "EPOCH", "EXELTM",
            "EXTPTREF", "EXRFTDTC",
        ],
    )?;

    // Batch CT normalization
    normalize_ct_batch(
        domain,
        df,
        context,
        &[
            "EXDOSFRM", "EXDOSU", "EXDOSFRQ", "EXROUTE", "EXCAT", "EXSCAT", "EPOCH",
        ],
    )?;

    // Normalize study day columns as numeric
    normalize_numeric_f64(domain, df, &["EXSTDY", "EXENDY"])?;

    Ok(())
}

/// Move treatment name from EXELTM to EXTRT if EXTRT is empty and EXELTM contains text.
fn relocate_extrt_from_exeltm(domain: &Domain, df: &mut DataFrame) -> Result<()> {
    let extrt = match col(domain, "EXTRT") {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };
    let exeltm = match col(domain, "EXELTM") {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };

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

    Ok(())
}
