//! Subject Elements (SE) domain processor.
//!
//! Processes SE domain data per SDTMIG v3.4 Section 5.1.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{col, compute_study_days_batch, ensure_date_pair_order};

pub(super) fn process_se(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Ensure start date is before end date
    if let Some(sestdtc) = col(domain, "SESTDTC") {
        ensure_date_pair_order(df, sestdtc, col(domain, "SEENDTC"))?;
    }

    // Compute study days
    compute_study_days_batch(
        domain,
        df,
        context,
        &[("SESTDTC", "SESTDY"), ("SEENDTC", "SEENDY")],
    )?;

    Ok(())
}
