use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_pr(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    for visit_col in ["VISIT", "VISITNUM"] {
        if let Some(name) = col(domain, visit_col)
            && has_column(df, &name)
        {
            let values = string_column(df, &name, Trim::Both)?;
            set_string_column(df, &name, values)?;
        }
    }
    if let Some(prstdtc) = col(domain, "PRSTDTC")
        && let Some(prstdy) = col(domain, "PRSTDY")
    {
        compute_study_day(domain, df, &prstdtc, &prstdy, ctx, "RFSTDTC")?;
    }
    if let Some(prendtc) = col(domain, "PRENDTC")
        && let Some(prendy) = col(domain, "PRENDY")
    {
        compute_study_day(domain, df, &prendtc, &prendy, ctx, "RFSTDTC")?;
    }
    if let Some(prdur) = col(domain, "PRDUR")
        && has_column(df, &prdur)
    {
        let values = string_column(df, &prdur, Trim::Both)?;
        set_string_column(df, &prdur, values)?;
    }
    if let Some(prrftdtc) = col(domain, "PRRFTDTC")
        && has_column(df, &prrftdtc)
    {
        let values = string_column(df, &prrftdtc, Trim::Both)?;
        set_string_column(df, &prrftdtc, values)?;
    }
    for col_name in ["PRTPTREF", "PRTPT", "PRTPTNUM", "PRELTM"] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, &name)
        {
            let values = string_column(df, &name, Trim::Both)?;
            set_string_column(df, &name, values)?;
        }
    }
    if let Some(prdecod) = col(domain, "PRDECOD") {
        if has_column(df, &prdecod) {
            let mut values = string_column(df, &prdecod, Trim::Both)?
                .into_iter()
                .map(|value| value.to_uppercase())
                .collect::<Vec<_>>();
            if let Some(usubjid) = col(domain, "USUBJID")
                && has_column(df, &usubjid)
            {
                let prefixes = string_column(df, &usubjid, Trim::Both)?
                    .into_iter()
                    .map(|value| value.split('-').next().unwrap_or("").trim().to_uppercase())
                    .collect::<Vec<_>>();
                for idx in 0..df.height() {
                    if !prefixes[idx].is_empty() && values[idx] == prefixes[idx] {
                        values[idx].clear();
                    }
                }
            }
            set_string_column(df, &prdecod, values)?;
        }
        if let Some(ct) = ctx.resolve_ct(domain, "PRDECOD") {
            let values = string_column(df, &prdecod, Trim::Both)?
                .into_iter()
                .map(|value| normalize_ct_value_keep(ct, &value))
                .collect::<Vec<_>>();
            set_string_column(df, &prdecod, values)?;
        }
    }
    if let Some(epoch) = col(domain, "EPOCH")
        && has_column(df, &epoch)
    {
        let values = string_column(df, &epoch, Trim::Both)?;
        set_string_column(df, &epoch, values)?;
    }
    let timing_defaults = [
        ("PRTPTREF", "VISIT"),
        ("PRTPT", "VISIT"),
        ("PRELTM", "PT0H"),
    ];
    for (col_name, default) in timing_defaults {
        if let Some(name) = col(domain, col_name) {
            let mut values = if has_column(df, &name) {
                string_column(df, &name, Trim::Both)?
            } else {
                vec![String::new(); df.height()]
            };
            for value in &mut values {
                if value.is_empty() {
                    *value = default.to_string();
                }
            }
            set_string_column(df, &name, values)?;
        }
    }
    if let Some(prtptnum) = col(domain, "PRTPTNUM") {
        let values = if has_column(df, &prtptnum) {
            numeric_column_i64(df, &prtptnum)?
        } else {
            vec![Some(1); df.height()]
        };
        let normalized = values.into_iter().map(|value| value.or(Some(1))).collect();
        set_i64_column(df, &prtptnum, normalized)?;
    }
    if let Some(visitnum) = col(domain, "VISITNUM")
        && has_column(df, &visitnum)
    {
        let values = numeric_column_i64(df, &visitnum)?
            .into_iter()
            .map(|value| value.or(Some(1)))
            .collect::<Vec<_>>();
        set_i64_column(df, &visitnum, values.clone())?;
        if let Some(visit) = col(domain, "VISIT") {
            let labels = values
                .into_iter()
                .map(|value| format!("Visit {}", value.unwrap_or(1)))
                .collect::<Vec<_>>();
            set_string_column(df, &visit, labels)?;
        }
    }
    Ok(())
}
