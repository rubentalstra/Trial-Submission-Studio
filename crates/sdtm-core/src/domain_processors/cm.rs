//! Concomitant Medications (CM) domain processor.
//!
//! Processes CM domain data per SDTMIG v3.4 Section 6.1.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    apply_map_upper, col, compute_study_days_batch, has_column, map_values, normalize_ct_batch,
    parse_f64, set_string_column, string_column, trim_columns,
};

pub(super) fn process_cm(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Lowercase CMDOSU
    if let Some(cmdosu) = col(domain, "CMDOSU")
        && has_column(df, cmdosu)
    {
        let values = string_column(df, cmdosu)?
            .into_iter()
            .map(|v| v.to_lowercase())
            .collect();
        set_string_column(df, cmdosu, values)?;
    }

    // Trim CMDUR
    trim_columns(domain, df, &["CMDUR"])?;

    // Prefix numeric CMDOSTXT with "DOSE "
    if let Some(cmdostxt) = col(domain, "CMDOSTXT")
        && has_column(df, cmdostxt)
    {
        let values = string_column(df, cmdostxt)?
            .into_iter()
            .map(|v| {
                let trimmed = v.trim();
                if parse_f64(trimmed).is_some() {
                    format!("DOSE {}", trimmed)
                } else {
                    trimmed.to_string()
                }
            })
            .collect();
        set_string_column(df, cmdostxt, values)?;
    }

    // Map CMSTAT values
    if let Some(cmstat) = col(domain, "CMSTAT") {
        let stat_map =
            map_values([("NOT DONE", "NOT DONE"), ("ND", "NOT DONE"), ("", ""), ("NAN", "")]);
        apply_map_upper(df, Some(cmstat), &stat_map)?;
    }

    // Batch CT normalization
    normalize_ct_batch(domain, df, context, &["CMDOSFRQ", "CMROUTE"])?;

    // Clean NA-like values from CMDOSU
    if let Some(cmdosu) = col(domain, "CMDOSU")
        && has_column(df, cmdosu)
    {
        let values = string_column(df, cmdosu)?
            .into_iter()
            .map(|v| {
                let upper = v.trim().to_uppercase();
                match upper.as_str() {
                    "" | "UNK" | "UNKNOWN" | "NA" | "N/A" | "NONE" | "NAN" | "<NA>" => String::new(),
                    _ => v.trim().to_string(),
                }
            })
            .collect();
        set_string_column(df, cmdosu, values)?;
    }

    // Compute study days
    compute_study_days_batch(
        domain,
        df,
        context,
        &[("CMSTDTC", "CMSTDY"), ("CMENDTC", "CMENDY")],
    )?;

    Ok(())
}
