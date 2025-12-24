use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_lb(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    for col_name in ["LBORRESU", "LBSTRESU"] {
        if let Some(name) = col(domain, col_name) {
            if has_column(df, &name) {
                let values = string_column(df, &name, Trim::Both)?
                    .into_iter()
                    .map(|value| normalize_empty_tokens(&value))
                    .collect();
                set_string_column(df, &name, values)?;
            }
        }
    }
    if let Some(lbtestcd) = col(domain, "LBTESTCD") {
        if has_column(df, &lbtestcd) {
            let mut values = string_column(df, &lbtestcd, Trim::Both)?
                .into_iter()
                .map(|value| value.to_uppercase())
                .collect::<Vec<_>>();
            if let Some(ct) = ctx.resolve_ct(domain, "LBTESTCD") {
                for idx in 0..values.len() {
                    let canonical = normalize_ct_value(ct, &values[idx]);
                    let valid = ct.submission_values.iter().any(|val| val == &canonical);
                    values[idx] = if valid { canonical } else { "".to_string() };
                }
            }
            set_string_column(df, &lbtestcd, values)?;
        }
    }
    if let (Some(lbtest), Some(lbtestcd)) = (col(domain, "LBTEST"), col(domain, "LBTESTCD")) {
        if has_column(df, &lbtest) && has_column(df, &lbtestcd) {
            let mut lbtest_vals = string_column(df, &lbtest, Trim::Both)?;
            let testcd_vals = string_column(df, &lbtestcd, Trim::Both)?;
            for idx in 0..df.height() {
                if lbtest_vals[idx].is_empty() && !testcd_vals[idx].is_empty() {
                    lbtest_vals[idx] = testcd_vals[idx].clone();
                }
            }
            set_string_column(df, &lbtest, lbtest_vals)?;
        }
    }
    if let Some(lbdtc) = col(domain, "LBDTC") {
        if let Some(lbdy) = col(domain, "LBDY") {
            compute_study_day(domain, df, &lbdtc, &lbdy, ctx, "RFSTDTC")?;
        }
    }
    if let Some(lbendtc) = col(domain, "LBENDTC") {
        if let Some(lbendy) = col(domain, "LBENDY") {
            compute_study_day(domain, df, &lbendtc, &lbendy, ctx, "RFSTDTC")?;
        }
    }
    if let Some(lbstresc) = col(domain, "LBSTRESC") {
        if has_column(df, &lbstresc) {
            let values = string_column(df, &lbstresc, Trim::Both)?
                .into_iter()
                .map(|value| match value.as_str() {
                    "Positive" => "POSITIVE".to_string(),
                    "Negative" => "NEGATIVE".to_string(),
                    _ => value,
                })
                .collect();
            set_string_column(df, &lbstresc, values)?;
        }
    }
    if let (Some(lborres), Some(lbstresc)) = (col(domain, "LBORRES"), col(domain, "LBSTRESC")) {
        if has_column(df, &lborres) && has_column(df, &lbstresc) {
            let orres = string_column(df, &lborres, Trim::Both)?
                .into_iter()
                .map(|value| normalize_empty_tokens(&value))
                .collect::<Vec<_>>();
            let mut stresc = string_column(df, &lbstresc, Trim::Both)?;
            for idx in 0..df.height() {
                if stresc[idx].is_empty() && !orres[idx].is_empty() {
                    stresc[idx] = orres[idx].clone();
                }
            }
            set_string_column(df, &lbstresc, stresc)?;
        }
    }
    if let (Some(lbstresc), Some(lbstresn)) = (col(domain, "LBSTRESC"), col(domain, "LBSTRESN")) {
        if has_column(df, &lbstresc) {
            let stresc_vals = string_column(df, &lbstresc, Trim::Both)?;
            let numeric_vals = stresc_vals
                .iter()
                .map(|value| parse_f64(value))
                .collect::<Vec<_>>();
            set_f64_column(df, &lbstresn, numeric_vals)?;
        }
    }
    if let Some(lbclsig) = col(domain, "LBCLSIG") {
        let yn_map = map_values([
            ("YES", "Y"),
            ("Y", "Y"),
            ("1", "Y"),
            ("TRUE", "Y"),
            ("NO", "N"),
            ("N", "N"),
            ("0", "N"),
            ("FALSE", "N"),
            ("CS", "Y"),
            ("NCS", "N"),
            ("", ""),
            ("NAN", ""),
        ]);
        apply_map_upper(df, Some(&lbclsig), &yn_map)?;
    }
    if let Some(ct) = ctx.resolve_ct(domain, "LBORRESU") {
        for col_name in ["LBORRESU", "LBSTRESU"] {
            if let Some(name) = col(domain, col_name) {
                if has_column(df, &name) {
                    let mut values = string_column(df, &name, Trim::Both)?;
                    for idx in 0..values.len() {
                        let canonical = normalize_ct_value(ct, &values[idx]);
                        let valid = ct.submission_values.iter().any(|val| val == &canonical);
                        values[idx] = if valid { canonical } else { "".to_string() };
                    }
                    set_string_column(df, &name, values)?;
                }
            }
        }
    }
    if let (Some(lborres), Some(lborresu)) = (col(domain, "LBORRES"), col(domain, "LBORRESU")) {
        if has_column(df, &lborres) && has_column(df, &lborresu) {
            let orres = string_column(df, &lborres, Trim::Both)?;
            let mut orresu = string_column(df, &lborresu, Trim::Both)?;
            for idx in 0..df.height() {
                if orres[idx].is_empty() {
                    orresu[idx].clear();
                }
            }
            set_string_column(df, &lborresu, orresu)?;
        }
    }
    if let (Some(lbstresc), Some(lbstresu)) = (col(domain, "LBSTRESC"), col(domain, "LBSTRESU")) {
        if has_column(df, &lbstresc) && has_column(df, &lbstresu) {
            let stresc = string_column(df, &lbstresc, Trim::Both)?;
            let mut stresu = string_column(df, &lbstresu, Trim::Both)?;
            for idx in 0..df.height() {
                if stresc[idx].is_empty() {
                    stresu[idx].clear();
                }
            }
            set_string_column(df, &lbstresu, stresu)?;
        }
    }
    if let (Some(lbseq), Some(usubjid)) = (col(domain, "LBSEQ"), col(domain, "USUBJID")) {
        assign_sequence(df, &lbseq, &usubjid)?;
    }
    Ok(())
}
