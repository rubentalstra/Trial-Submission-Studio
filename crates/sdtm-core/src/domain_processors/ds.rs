//! Disposition (DS) domain processor.
//!
//! Processes DS domain data per SDTMIG v3.4 Section 6.2.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    col, compute_study_days_batch, has_column, normalize_ct_batch, normalize_ct_columns,
    normalize_iso8601, set_string_column, string_column, trim_columns,
};

pub(super) fn process_ds(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Trim string columns
    trim_columns(domain, df, &["DSDECOD", "DSTERM", "DSCAT", "EPOCH"])?;

    // Batch CT normalization
    normalize_ct_batch(domain, df, context, &["DSCAT", "DSSCAT", "EPOCH"])?;

    // Normalize DSDECOD via CT
    normalize_ct_columns(domain, df, context, "DSDECOD", &["DSDECOD"])?;

    // Normalize date columns
    for date_col in ["DSSTDTC", "DSDTC"] {
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
        &[("DSSTDTC", "DSSTDY"), ("DSDTC", "DSDY")],
    )?;

    Ok(())
}
