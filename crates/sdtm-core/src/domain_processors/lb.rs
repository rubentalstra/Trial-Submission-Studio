//! Laboratory Results (LB) domain processor.
//!
//! Processes LB domain data per SDTMIG v3.4 Section 6.3.5.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use sdtm_normalization::normalization::ct::is_yes_no_token;
use crate::pipeline_context::PipelineContext;

use super::common::{
    apply_map_upper, backward_fill, backward_fill_var, clean_na_values, clean_na_values_vars,
    clear_units_batch, col, compute_study_days_batch, derive_test_from_testcd, has_column,
    normalize_ct_batch, normalize_ct_columns, parse_f64, resolve_testcd_from_test, set_f64_column,
    set_string_column, string_column, yn_mapping,
};

pub(super) fn process_lb(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Clean NA-like values from unit columns
    clean_na_values_vars(domain, df, &["LBORRESU", "LBSTRESU"])?;

    // Backward fill: LBORRESU → LBSTRESU
    backward_fill_var(domain, df, "LBORRESU", "LBSTRESU")?;

    // Normalize and uppercase LBTESTCD
    if let Some(lbtestcd) = col(domain, "LBTESTCD")
        && has_column(df, lbtestcd)
    {
        let values = string_column(df, lbtestcd)?
            .into_iter()
            .map(|v| v.to_uppercase())
            .collect::<Vec<_>>();
        set_string_column(df, lbtestcd, values)?;
    }

    // Resolve and derive test code/name
    normalize_ct_columns(domain, df, context, "LBTESTCD", &["LBTESTCD"])?;
    resolve_testcd_from_test(domain, df, context, "LBTESTCD", "LBTEST", "LBTESTCD")?;
    backward_fill_var(domain, df, "LBTESTCD", "LBTEST")?;
    derive_test_from_testcd(domain, df, context, "LBTEST", "LBTESTCD", "LBTESTCD")?;

    // Compute study days
    compute_study_days_batch(
        domain,
        df,
        context,
        &[("LBDTC", "LBDY"), ("LBENDTC", "LBENDY")],
    )?;

    // Normalize result values (Positive/Negative)
    if let Some(lbstresc) = col(domain, "LBSTRESC")
        && has_column(df, lbstresc)
    {
        let values = string_column(df, lbstresc)?
            .into_iter()
            .map(|v| match v.as_str() {
                "Positive" => "POSITIVE".to_string(),
                "Negative" => "NEGATIVE".to_string(),
                _ => v,
            })
            .collect();
        set_string_column(df, lbstresc, values)?;
    }

    // Clean and backward fill LBORRES → LBSTRESC
    if let (Some(lborres), Some(lbstresc)) = (col(domain, "LBORRES"), col(domain, "LBSTRESC"))
        && has_column(df, lborres)
        && has_column(df, lbstresc)
    {
        clean_na_values(df, lborres)?;
        backward_fill(df, lborres, lbstresc)?;
    }

    // Derive numeric result from LBSTRESC
    if let (Some(lbstresc), Some(lbstresn)) = (col(domain, "LBSTRESC"), col(domain, "LBSTRESN"))
        && has_column(df, lbstresc)
    {
        let stresc_vals = string_column(df, lbstresc)?;
        let numeric_vals = stresc_vals.iter().map(|v| parse_f64(v)).collect();
        set_f64_column(df, lbstresn, numeric_vals)?;
    }

    // Apply Y/N mapping to LBCLSIG
    if let Some(lbclsig) = col(domain, "LBCLSIG") {
        apply_map_upper(df, Some(lbclsig), &yn_mapping())?;
    }

    // Batch CT normalization
    normalize_ct_batch(
        domain,
        df,
        context,
        &[
            "LBORRESU", "LBTEST", "LBCAT", "LBSCAT", "LBSPEC", "LBMETHOD", "LBLOC", "LBFAST",
            "LBNRIND",
        ],
    )?;

    // Clear LBCOLSRT if it contains Y/N tokens
    if let Some(lbcolsrt) = col(domain, "LBCOLSRT")
        && has_column(df, lbcolsrt)
    {
        let mut values = string_column(df, lbcolsrt)?;
        for value in &mut values {
            if is_yes_no_token(value) {
                value.clear();
            }
        }
        set_string_column(df, lbcolsrt, values)?;
    }

    // Clear unit when result is empty
    clear_units_batch(
        domain,
        df,
        &[("LBORRES", "LBORRESU"), ("LBSTRESC", "LBSTRESU")],
    )?;

    Ok(())
}
