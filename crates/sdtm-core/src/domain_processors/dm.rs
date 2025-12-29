//! Demographics (DM) domain processor.
//!
//! Processes DM domain data per SDTMIG v3.4 Section 6.3.1.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    compute_study_days_batch, has_column, normalize_ct_batch, normalize_numeric_f64, trim_columns,
};

pub(super) fn process_dm(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Normalize AGE as numeric
    normalize_numeric_f64(domain, df, &["AGE"])?;

    // Batch CT normalization
    normalize_ct_batch(domain, df, context, &["AGEU", "ETHNIC", "RACE", "SEX"])?;

    // Trim date columns
    trim_columns(
        domain,
        df,
        &[
            "COUNTRY", "RFICDTC", "RFSTDTC", "RFENDTC", "RFXSTDTC", "RFXENDTC", "DMDTC",
        ],
    )?;

    // Compute study day (only if both DTC and reference date exist)
    if has_column(df, "RFSTDTC") {
        compute_study_days_batch(domain, df, context, &[("DMDTC", "DMDY")])?;
    }

    Ok(())
}
