use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_pr(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    for visit_col in ["VISIT", "VISITNUM"] {
        if let Some(name) = col(domain, visit_col)
            && has_column(df, name)
        {
            let values = string_column(df, name)?;
            set_string_column(df, name, values)?;
        }
    }
    if let Some(prstdtc) = col(domain, "PRSTDTC")
        && let Some(prstdy) = col(domain, "PRSTDY")
    {
        compute_study_day(domain, df, prstdtc, prstdy, context, "RFSTDTC")?;
    }
    if let Some(prendtc) = col(domain, "PRENDTC")
        && let Some(prendy) = col(domain, "PRENDY")
    {
        compute_study_day(domain, df, prendtc, prendy, context, "RFSTDTC")?;
    }
    if let Some(prdur) = col(domain, "PRDUR")
        && has_column(df, prdur)
    {
        let values = string_column(df, prdur)?;
        set_string_column(df, prdur, values)?;
    }
    if let Some(prrftdtc) = col(domain, "PRRFTDTC")
        && has_column(df, prrftdtc)
    {
        let values = string_column(df, prrftdtc)?;
        set_string_column(df, prrftdtc, values)?;
    }
    for col_name in ["PRTPTREF", "PRTPT", "PRELTM"] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, name)
        {
            let values = string_column(df, name)?;
            set_string_column(df, name, values)?;
        }
    }
    if let Some(prdecod) = col(domain, "PRDECOD") {
        if has_column(df, prdecod) {
            let values = string_column(df, prdecod)?
                .into_iter()
                .map(|value| value.to_uppercase())
                .collect::<Vec<_>>();
            set_string_column(df, prdecod, values)?;
        }
        if let Some(ct) = context.resolve_ct(domain, "PRDECOD") {
            let values = string_column(df, prdecod)?
                .into_iter()
                .map(|value| normalize_ct_value(ct, &value, context.options.ct_matching))
                .collect::<Vec<_>>();
            set_string_column(df, prdecod, values)?;
        }
    }
    if let Some(epoch) = col(domain, "EPOCH")
        && has_column(df, epoch)
    {
        let values = string_column(df, epoch)?;
        set_string_column(df, epoch, values)?;
    }
    if let Some(prtptnum) = col(domain, "PRTPTNUM")
        && has_column(df, prtptnum)
    {
        let values = numeric_column_i64(df, prtptnum)?;
        set_i64_column(df, prtptnum, values)?;
    }
    if let Some(visitnum) = col(domain, "VISITNUM")
        && has_column(df, visitnum)
    {
        let values = numeric_column_i64(df, visitnum)?;
        set_i64_column(df, visitnum, values)?;
    }

    // Normalize CT columns
    // PRCAT: Category
    normalize_ct_columns(domain, df, context, "PRCAT", &["PRCAT"])?;
    // PRSCAT: Subcategory
    normalize_ct_columns(domain, df, context, "PRSCAT", &["PRSCAT"])?;
    // EPOCH: Epoch (Codelist C99079)
    normalize_ct_columns(domain, df, context, "EPOCH", &["EPOCH"])?;
    // PRROUTE: Route of Administration (Codelist C66729)
    normalize_ct_columns(domain, df, context, "PRROUTE", &["PRROUTE"])?;
    // PRDOSFRM: Dose Form (Codelist C66726)
    normalize_ct_columns(domain, df, context, "PRDOSFRM", &["PRDOSFRM"])?;
    // PRDOSU: Dose Units (Codelist C71620)
    normalize_ct_columns(domain, df, context, "PRDOSU", &["PRDOSU"])?;

    Ok(())
}
