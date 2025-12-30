//! Procedures (PR) domain processor.
//!
//! Processes PR domain data per SDTMIG v3.4 Section 6.3.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    col, compute_study_days_batch, has_column, normalize_ct_batch, normalize_ct_value,
    normalize_numeric_i64, set_string_column, string_column,
};

pub(super) fn process_pr(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Compute study days
    compute_study_days_batch(
        domain,
        df,
        context,
        &[("PRSTDTC", "PRSTDY"), ("PRENDTC", "PRENDY")],
    )?;

    // Normalize PRDECOD to uppercase + CT
    if let Some(prdecod) = col(domain, "PRDECOD")
        && has_column(df, prdecod)
    {
        let mut values: Vec<String> = string_column(df, prdecod)?
            .into_iter()
            .map(|v| v.to_uppercase())
            .collect();
        if let Some(ct) = context.resolve_ct(domain, "PRDECOD") {
            for value in &mut values {
                *value = normalize_ct_value(ct, value, &context.options.normalization);
            }
        }
        set_string_column(df, prdecod, values)?;
    }

    // Normalize numeric columns
    normalize_numeric_i64(domain, df, &["PRTPTNUM", "VISITNUM"])?;

    // Batch CT normalization
    normalize_ct_batch(
        domain,
        df,
        context,
        &["PRCAT", "PRSCAT", "EPOCH", "PRROUTE", "PRDOSFRM", "PRDOSU"],
    )?;

    Ok(())
}
