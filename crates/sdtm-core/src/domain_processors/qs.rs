//! Questionnaires (QS) domain processor.
//!
//! Processes QS domain data per SDTMIG v3.4 Section 6.3.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    backward_fill_var, col, compute_study_days_batch, has_column, normalize_ct_batch,
    normalize_iso8601, set_string_column, string_column,
};

pub(super) fn process_qs(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Backward fill: QSORRES → QSSTRESC
    backward_fill_var(domain, df, "QSORRES", "QSSTRESC")?;

    // Clear QSLOBXFL="N" values
    if let Some(qslobxfl) = col(domain, "QSLOBXFL")
        && has_column(df, qslobxfl)
    {
        let values = string_column(df, qslobxfl)?
            .into_iter()
            .map(|v| if v == "N" { String::new() } else { v })
            .collect();
        set_string_column(df, qslobxfl, values)?;
    }

    // Normalize date and compute study day
    if let Some(qsdtc) = col(domain, "QSDTC")
        && has_column(df, qsdtc)
    {
        let values = string_column(df, qsdtc)?
            .into_iter()
            .map(|v| normalize_iso8601(&v))
            .collect();
        set_string_column(df, qsdtc, values)?;
    }
    compute_study_days_batch(domain, df, context, &[("QSDTC", "QSDY")])?;

    // Clear QSTPTREF if no timing columns present
    clear_orphan_tptref(domain, df)?;

    // Batch CT normalization
    normalize_ct_batch(
        domain,
        df,
        context,
        &["QSTESTCD", "QSTEST", "QSCAT", "QSSCAT", "EPOCH"],
    )?;

    Ok(())
}

/// Clear QSTPTREF if no timing columns (QSELTM, QSTPTNUM, QSTPT) are present.
fn clear_orphan_tptref(domain: &Domain, df: &mut DataFrame) -> Result<()> {
    let Some(qstptref) = col(domain, "QSTPTREF") else {
        return Ok(());
    };
    if !has_column(df, qstptref) {
        return Ok(());
    }

    let has_timing = ["QSELTM", "QSTPTNUM", "QSTPT"]
        .into_iter()
        .filter_map(|name| col(domain, name))
        .any(|name| has_column(df, name));

    if !has_timing {
        set_string_column(df, qstptref, vec![String::new(); df.height()])?;
    }
    Ok(())
}
