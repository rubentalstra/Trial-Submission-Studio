//! Inclusion/Exclusion Criteria (IE) domain processor.
//!
//! Processes IE domain data per SDTMIG v3.4 Section 6.3.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    apply_map_upper, backward_fill_var, col, compute_study_days_batch, has_column,
    normalize_ct_batch, normalize_ct_columns, normalize_numeric_f64, set_string_column,
    string_column, yn_mapping,
};

pub(super) fn process_ie(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Apply Y/N mapping to IEORRES
    if let Some(ieorres) = col(domain, "IEORRES") {
        apply_map_upper(df, Some(ieorres), &yn_mapping())?;
    }

    // Backward fill: IEORRES → IESTRESC
    backward_fill_var(domain, df, "IEORRES", "IESTRESC")?;

    // Set IESTRESC to "Y" for EXCLUSION category when empty
    set_exclusion_default(domain, df)?;

    // Compute study day
    compute_study_days_batch(domain, df, context, &[("IEDTC", "IEDY")])?;
    normalize_numeric_f64(domain, df, &["IEDY"])?;

    // Batch CT normalization
    normalize_ct_batch(
        domain,
        df,
        context,
        &["IETESTCD", "IETEST", "IECAT", "IESCAT", "EPOCH"],
    )?;
    // IEORRES/IESTRESC share codelist
    normalize_ct_columns(domain, df, context, "IEORRES", &["IEORRES", "IESTRESC"])?;

    Ok(())
}

/// Set IESTRESC to "Y" for EXCLUSION category when empty.
fn set_exclusion_default(domain: &Domain, df: &mut DataFrame) -> Result<()> {
    let (Some(iecat), Some(iestresc)) = (col(domain, "IECAT"), col(domain, "IESTRESC")) else {
        return Ok(());
    };
    if !has_column(df, iecat) || !has_column(df, iestresc) {
        return Ok(());
    }

    let cat_vals = string_column(df, iecat)?;
    let mut stresc_vals = string_column(df, iestresc)?;
    for idx in 0..df.height() {
        if cat_vals[idx].eq_ignore_ascii_case("EXCLUSION") && stresc_vals[idx].trim().is_empty() {
            stresc_vals[idx] = "Y".to_string();
        }
    }
    set_string_column(df, iestresc, stresc_vals)
}
