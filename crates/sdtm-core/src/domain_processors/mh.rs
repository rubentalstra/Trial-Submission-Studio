//! Medical History (MH) domain processor.
//!
//! Processes MH domain data per SDTMIG v3.4 Section 6.3.6.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    col, compute_study_days_batch, has_column, normalize_ct_batch, normalize_iso8601,
    normalize_numeric_f64, set_string_column, string_column,
};

pub(super) fn process_mh(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Normalize MHSEQ as numeric
    normalize_numeric_f64(domain, df, &["MHSEQ"])?;

    // Backward fill MHTERM from MHDECOD
    if let Some(mhterm) = col(domain, "MHTERM")
        && has_column(df, mhterm)
    {
        let mut terms = string_column(df, mhterm)?;
        if let Some(mhdecod) = col(domain, "MHDECOD")
            && has_column(df, mhdecod)
        {
            let decods = string_column(df, mhdecod)?;
            for idx in 0..df.height() {
                if terms[idx].is_empty() && !decods[idx].is_empty() {
                    terms[idx] = decods[idx].clone();
                }
            }
        }
        set_string_column(df, mhterm, terms)?;
    }

    // Normalize date columns
    for date_col in ["MHSTDTC", "MHENDTC", "MHDTC"] {
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

    // Process MHENRF (End Relative to Reference Period)
    if let Some(mhenrf) = col(domain, "MHENRF")
        && has_column(df, mhenrf)
    {
        let values = string_column(df, mhenrf)?
            .into_iter()
            .map(|v| match v.to_uppercase().as_str() {
                "Y" | "YES" | "TRUE" | "1" => "ONGOING".to_string(),
                "N" | "NO" | "FALSE" | "0" => String::new(),
                "PRIOR" => "BEFORE".to_string(),
                "POST" => "AFTER".to_string(),
                "CONCURRENT" => "COINCIDENT".to_string(),
                "UNK" | "U" => "UNKNOWN".to_string(),
                _ => v.to_uppercase(),
            })
            .collect();
        set_string_column(df, mhenrf, values)?;
    }

    // Compute study day
    compute_study_days_batch(domain, df, context, &[("MHDTC", "MHDY")])?;
    normalize_numeric_f64(domain, df, &["MHDY"])?;

    // Batch CT normalization
    normalize_ct_batch(
        domain,
        df,
        context,
        &["MHCAT", "MHSCAT", "MHENRF", "MHSTRF", "MHPRESP", "MHOCCUR"],
    )?;

    Ok(())
}
