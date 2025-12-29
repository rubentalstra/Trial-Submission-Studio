//! Physical Examination (PE) domain processor.
//!
//! Processes PE domain data per SDTMIG v3.4 Section 6.3.7.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    apply_map_upper, backward_fill_var, col, compute_study_days_batch, map_values,
    normalize_ct_batch, trim_columns,
};

pub(super) fn process_pe(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Map PESTAT values
    if let Some(pestat) = col(domain, "PESTAT") {
        let stat_map = map_values([
            ("NOT DONE", "NOT DONE"),
            ("ND", "NOT DONE"),
            ("DONE", ""),
            ("COMPLETED", ""),
            ("", ""),
            ("NAN", ""),
        ]);
        apply_map_upper(df, Some(pestat), &stat_map)?;
    }

    // Backward fill PEORRES → PESTRESC
    backward_fill_var(domain, df, "PEORRES", "PESTRESC")?;

    // Compute study day
    compute_study_days_batch(domain, df, context, &[("PEDTC", "PEDY")])?;

    // Trim EPOCH
    trim_columns(domain, df, &["EPOCH"])?;

    // Batch CT normalization
    normalize_ct_batch(
        domain,
        df,
        context,
        &["PESTAT", "PELOC", "PEBODSYS", "PECAT", "PESCAT", "EPOCH"],
    )?;

    Ok(())
}
