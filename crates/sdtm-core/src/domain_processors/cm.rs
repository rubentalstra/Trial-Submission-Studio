//! Concomitant Medications (CM) domain processor.
//!
//! Processes CM domain data per SDTMIG v3.4 Section 6.1.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    apply_map_upper, col, compute_study_day, has_column, map_values, normalize_ct_columns,
    parse_f64, set_string_column, string_column,
};

pub(super) fn process_cm(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if let Some(cmdosu) = col(domain, "CMDOSU")
        && has_column(df, cmdosu)
    {
        let values = string_column(df, cmdosu)?
            .into_iter()
            .map(|v| v.to_lowercase())
            .collect();
        set_string_column(df, cmdosu, values)?;
    }
    if let Some(cmdur) = col(domain, "CMDUR")
        && has_column(df, cmdur)
    {
        let values = string_column(df, cmdur)?;
        set_string_column(df, cmdur, values)?;
    }
    if let Some(cmdostxt) = col(domain, "CMDOSTXT")
        && has_column(df, cmdostxt)
    {
        let values = string_column(df, cmdostxt)?
            .into_iter()
            .map(|value| {
                let trimmed = value.trim();
                if parse_f64(trimmed).is_some() {
                    format!("DOSE {}", trimmed)
                } else {
                    trimmed.to_string()
                }
            })
            .collect();
        set_string_column(df, cmdostxt, values)?;
    }
    if let Some(cmstat) = col(domain, "CMSTAT") {
        let stat_map = map_values([
            ("NOT DONE", "NOT DONE"),
            ("ND", "NOT DONE"),
            ("", ""),
            ("NAN", ""),
        ]);
        apply_map_upper(df, Some(cmstat), &stat_map)?;
    }
    // Normalize dosing frequency via CT (Codelist C71113)
    normalize_ct_columns(domain, df, context, "CMDOSFRQ", &["CMDOSFRQ"])?;
    // Normalize administration route via CT (Codelist C66729)
    normalize_ct_columns(domain, df, context, "CMROUTE", &["CMROUTE"])?;
    if let Some(cmdosu) = col(domain, "CMDOSU")
        && has_column(df, cmdosu)
    {
        let values = string_column(df, cmdosu)?
            .into_iter()
            .map(|value| {
                let trimmed = value.trim();
                let upper = trimmed.to_uppercase();
                match upper.as_str() {
                    "" | "UNK" | "UNKNOWN" | "NA" | "N/A" | "NONE" | "NAN" | "<NA>" => {
                        String::new()
                    }
                    _ => trimmed.to_string(),
                }
            })
            .collect();
        set_string_column(df, cmdosu, values)?;
    }
    if let Some(cmstdtc) = col(domain, "CMSTDTC")
        && let Some(cmstdy) = col(domain, "CMSTDY")
    {
        compute_study_day(domain, df, cmstdtc, cmstdy, context, "RFSTDTC")?;
    }
    if let Some(cmendtc) = col(domain, "CMENDTC")
        && let Some(cmendy) = col(domain, "CMENDY")
    {
        compute_study_day(domain, df, cmendtc, cmendy, context, "RFSTDTC")?;
    }
    Ok(())
}
