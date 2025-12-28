use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_ie(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Normalize string columns
    for col_name in [
        "IEORRES", "IESTRESC", "IETESTCD", "IETEST", "IECAT", "IESCAT", "EPOCH",
    ] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, name)
        {
            let values = string_column(df, name)?;
            set_string_column(df, name, values)?;
        }
    }

    // Apply Y/N mapping to IEORRES
    if let Some(ieorres) = col(domain, "IEORRES") {
        apply_map_upper(df, Some(ieorres), &yn_mapping())?;
    }

    // Backward fill: IEORRES → IESTRESC
    backward_fill_var(domain, df, "IEORRES", "IESTRESC")?;

    // Set IESTRESC to "Y" for EXCLUSION category when empty
    if let (Some(iecat), Some(iestresc)) = (col(domain, "IECAT"), col(domain, "IESTRESC"))
        && has_column(df, iecat)
        && has_column(df, iestresc)
    {
        let cat_vals = string_column(df, iecat)?;
        let mut stresc_vals = string_column(df, iestresc)?;
        for idx in 0..df.height() {
            if cat_vals[idx].eq_ignore_ascii_case("EXCLUSION") && stresc_vals[idx].trim().is_empty()
            {
                stresc_vals[idx] = "Y".to_string();
            }
        }
        set_string_column(df, iestresc, stresc_vals)?;
    }

    // Compute study day
    if let (Some(iedtc), Some(iedy)) = (col(domain, "IEDTC"), col(domain, "IEDY"))
        && has_column(df, iedtc)
    {
        let values = string_column(df, iedtc)?;
        set_string_column(df, iedtc, values)?;
        compute_study_day(domain, df, iedtc, iedy, context, "RFSTDTC")?;
        let numeric = numeric_column_f64(df, iedy)?;
        set_f64_column(df, iedy, numeric)?;
    }

    Ok(())
}
