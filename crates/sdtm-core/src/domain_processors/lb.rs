use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::ct_utils::is_yes_no_token;
use crate::pipeline_context::PipelineContext;

use super::common::{
    apply_map_upper, backward_fill, backward_fill_var, clean_na_values, clean_na_values_vars,
    clear_unit_when_empty_var, col, compute_study_day, derive_test_from_testcd, has_column,
    normalize_ct_columns, parse_f64, resolve_testcd_from_test, set_f64_column, set_string_column,
    string_column, yn_mapping,
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
            .map(|value| value.to_uppercase())
            .collect::<Vec<_>>();
        set_string_column(df, lbtestcd, values)?;
    }

    // Normalize LBTESTCD via CT
    normalize_ct_columns(domain, df, context, "LBTESTCD", &["LBTESTCD"])?;

    // Resolve LBTESTCD from LBTEST when invalid
    resolve_testcd_from_test(domain, df, context, "LBTESTCD", "LBTEST", "LBTESTCD")?;

    // Backward fill: LBTESTCD → LBTEST
    backward_fill_var(domain, df, "LBTESTCD", "LBTEST")?;

    // Derive LBTEST from LBTESTCD using CT preferred terms
    derive_test_from_testcd(domain, df, context, "LBTEST", "LBTESTCD", "LBTESTCD")?;

    // Compute study day for LBDTC
    if let Some(lbdtc) = col(domain, "LBDTC")
        && let Some(lbdy) = col(domain, "LBDY")
    {
        compute_study_day(domain, df, lbdtc, lbdy, context, "RFSTDTC")?;
    }

    // Compute study day for LBENDTC
    if let Some(lbendtc) = col(domain, "LBENDTC")
        && let Some(lbendy) = col(domain, "LBENDY")
    {
        compute_study_day(domain, df, lbendtc, lbendy, context, "RFSTDTC")?;
    }

    // Normalize result values (Positive/Negative)
    if let Some(lbstresc) = col(domain, "LBSTRESC")
        && has_column(df, lbstresc)
    {
        let values = string_column(df, lbstresc)?
            .into_iter()
            .map(|value| match value.as_str() {
                "Positive" => "POSITIVE".to_string(),
                "Negative" => "NEGATIVE".to_string(),
                _ => value,
            })
            .collect();
        set_string_column(df, lbstresc, values)?;
    }

    // Clean NA values from LBORRES and backward fill to LBSTRESC
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
        let numeric_vals = stresc_vals
            .iter()
            .map(|value| parse_f64(value))
            .collect::<Vec<_>>();
        set_f64_column(df, lbstresn, numeric_vals)?;
    }

    // Apply Y/N mapping to LBCLSIG
    if let Some(lbclsig) = col(domain, "LBCLSIG") {
        apply_map_upper(df, Some(lbclsig), &yn_mapping())?;
    }

    // Normalize unit columns via CT
    normalize_ct_columns(domain, df, context, "LBORRESU", &["LBORRESU", "LBSTRESU"])?;

    // LBTEST: Test Name
    normalize_ct_columns(domain, df, context, "LBTEST", &["LBTEST"])?;
    // LBCAT: Category
    normalize_ct_columns(domain, df, context, "LBCAT", &["LBCAT"])?;
    // LBSCAT: Subcategory
    normalize_ct_columns(domain, df, context, "LBSCAT", &["LBSCAT"])?;
    // LBSPEC: Specimen (Codelist C78734)
    normalize_ct_columns(domain, df, context, "LBSPEC", &["LBSPEC"])?;
    // LBMETHOD: Method (Codelist C85492)
    normalize_ct_columns(domain, df, context, "LBMETHOD", &["LBMETHOD"])?;
    // LBLOC: Location (Codelist C74456)
    normalize_ct_columns(domain, df, context, "LBLOC", &["LBLOC"])?;
    // LBFAST: Fasting Status (Codelist C66742)
    normalize_ct_columns(domain, df, context, "LBFAST", &["LBFAST"])?;
    // LBNRIND: Normal Range Indicator (Codelist C78736)
    normalize_ct_columns(domain, df, context, "LBNRIND", &["LBNRIND"])?;

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

    // Clear unit when result is empty (original result)
    clear_unit_when_empty_var(domain, df, "LBORRES", "LBORRESU")?;

    // Clear unit when result is empty (standardized result)
    clear_unit_when_empty_var(domain, df, "LBSTRESC", "LBSTRESU")?;

    Ok(())
}
