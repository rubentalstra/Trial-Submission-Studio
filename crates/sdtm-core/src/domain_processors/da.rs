use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    apply_map_upper, backward_fill_var, clear_unit_when_empty_var, col, compute_study_day,
    has_column, map_values, normalize_ct_columns, numeric_column_f64, parse_f64, set_f64_column,
    set_string_column, string_column,
};

pub(super) fn process_da(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Apply DASTAT mapping
    if let Some(dastat) = col(domain, "DASTAT") {
        let stat_map = map_values([
            ("NOT DONE", "NOT DONE"),
            ("ND", "NOT DONE"),
            ("DONE", ""),
            ("COMPLETED", ""),
            ("", ""),
            ("NAN", ""),
        ]);
        apply_map_upper(df, Some(dastat), &stat_map)?;
    }

    // Set DASTAT to "NOT DONE" when DAREASND is populated
    if let (Some(dastat), Some(dareasnd)) = (col(domain, "DASTAT"), col(domain, "DAREASND"))
        && has_column(df, dastat)
        && has_column(df, dareasnd)
    {
        let reason_vals = string_column(df, dareasnd)?;
        let mut stat_vals = string_column(df, dastat)?;
        for idx in 0..df.height() {
            if stat_vals[idx].is_empty() && !reason_vals[idx].is_empty() {
                stat_vals[idx] = "NOT DONE".to_string();
            }
        }
        set_string_column(df, dastat, stat_vals)?;
    }

    // Clear unit when result is empty
    clear_unit_when_empty_var(domain, df, "DAORRES", "DAORRESU")?;

    // Backward fill: DAORRES → DASTRESC
    backward_fill_var(domain, df, "DAORRES", "DASTRESC")?;

    // Derive numeric result from DASTRESC
    if let (Some(dastresc), Some(dastresn)) = (col(domain, "DASTRESC"), col(domain, "DASTRESN"))
        && has_column(df, dastresc)
    {
        let stresc = string_column(df, dastresc)?;
        let mut stresn_vals: Vec<Option<f64>> = vec![None; df.height()];
        if has_column(df, dastresn) {
            stresn_vals = numeric_column_f64(df, dastresn)?;
        }
        for (idx, value) in stresc.iter().enumerate() {
            if stresn_vals[idx].is_none()
                && let Some(parsed) = parse_f64(value)
            {
                stresn_vals[idx] = Some(parsed);
            }
            if !value.is_empty() && parse_f64(value).is_none() {
                stresn_vals[idx] = None;
            }
        }
        set_f64_column(df, dastresn, stresn_vals)?;
    }

    // Compute study day
    if let Some(dadtc) = col(domain, "DADTC")
        && has_column(df, dadtc)
        && let Some(dady) = col(domain, "DADY")
    {
        compute_study_day(domain, df, dadtc, dady, context, "RFSTDTC")?;
    }

    // Normalize CT columns
    // DASTAT: Status (Codelist C66789)
    normalize_ct_columns(domain, df, context, "DASTAT", &["DASTAT"])?;
    // DATESTCD: Test Code
    normalize_ct_columns(domain, df, context, "DATESTCD", &["DATESTCD"])?;
    // DATEST: Test Name
    normalize_ct_columns(domain, df, context, "DATEST", &["DATEST"])?;
    // DACAT: Category
    normalize_ct_columns(domain, df, context, "DACAT", &["DACAT"])?;
    // DASCAT: Subcategory
    normalize_ct_columns(domain, df, context, "DASCAT", &["DASCAT"])?;
    // DAORRESU: Original Result Unit
    normalize_ct_columns(domain, df, context, "DAORRESU", &["DAORRESU"])?;
    // DASTRESU: Standardized Result Unit
    normalize_ct_columns(domain, df, context, "DASTRESU", &["DASTRESU"])?;
    // EPOCH: Epoch (Codelist C99079)
    normalize_ct_columns(domain, df, context, "EPOCH", &["EPOCH"])?;

    Ok(())
}
