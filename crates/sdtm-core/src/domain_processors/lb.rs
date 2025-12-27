use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::ct_utils::resolve_ct_value_from_hint;
use crate::{ProcessingContext, is_yes_no_token};

use super::common::*;

pub(super) fn process_lb(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    for col_name in ["LBORRESU", "LBSTRESU"] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, &name)
        {
            let values = string_column(df, &name, Trim::Both)?
                .into_iter()
                .map(|value| normalize_empty_tokens(&value))
                .collect();
            set_string_column(df, &name, values)?;
        }
    }
    if let (Some(lborresu), Some(lbstresu)) = (col(domain, "LBORRESU"), col(domain, "LBSTRESU"))
        && has_column(df, &lborresu)
        && has_column(df, &lbstresu)
    {
        let orresu_vals = string_column(df, &lborresu, Trim::Both)?;
        let mut stresu_vals = string_column(df, &lbstresu, Trim::Both)?;
        for (stresu, orresu) in stresu_vals.iter_mut().zip(orresu_vals.iter()) {
            if stresu.is_empty() && !orresu.is_empty() {
                *stresu = orresu.clone();
            }
        }
        set_string_column(df, &lbstresu, stresu_vals)?;
    }
    if let Some(lbtestcd) = col(domain, "LBTESTCD")
        && has_column(df, &lbtestcd)
    {
        let mut values = string_column(df, &lbtestcd, Trim::Both)?
            .into_iter()
            .map(|value| value.to_uppercase())
            .collect::<Vec<_>>();
        if let Some(ct) = ctx.resolve_ct(domain, "LBTESTCD") {
            for value in &mut values {
                *value = normalize_ct_value_safe(ct, value);
            }
        }
        set_string_column(df, &lbtestcd, values)?;
    }
    if let (Some(lbtest), Some(lbtestcd)) = (col(domain, "LBTEST"), col(domain, "LBTESTCD"))
        && has_column(df, &lbtest)
        && has_column(df, &lbtestcd)
        && let Some(ct) = ctx.resolve_ct(domain, "LBTESTCD")
    {
        let test_vals = string_column(df, &lbtest, Trim::Both)?;
        let mut testcd_vals = string_column(df, &lbtestcd, Trim::Both)?;
        for (testcd, test) in testcd_vals.iter_mut().zip(test_vals.iter()) {
            let existing = testcd.clone();
            let valid =
                !existing.is_empty() && ct.submission_values.iter().any(|val| val == &existing);
            if valid {
                continue;
            }
            if let Some(mapped) = resolve_ct_lenient(ct, test) {
                *testcd = mapped;
            } else if let Some(mapped) = resolve_ct_value_from_hint(ct, test) {
                *testcd = mapped;
            }
        }
        set_string_column(df, &lbtestcd, testcd_vals)?;
    }
    if let (Some(lbtest), Some(lbtestcd)) = (col(domain, "LBTEST"), col(domain, "LBTESTCD"))
        && has_column(df, &lbtest)
        && has_column(df, &lbtestcd)
    {
        let mut lbtest_vals = string_column(df, &lbtest, Trim::Both)?;
        let testcd_vals = string_column(df, &lbtestcd, Trim::Both)?;
        for (test, testcd) in lbtest_vals.iter_mut().zip(testcd_vals.iter()) {
            if test.is_empty() && !testcd.is_empty() {
                *test = testcd.clone();
            }
        }
        set_string_column(df, &lbtest, lbtest_vals)?;
    }
    if let (Some(lbtest), Some(lbtestcd)) = (col(domain, "LBTEST"), col(domain, "LBTESTCD"))
        && has_column(df, &lbtest)
        && has_column(df, &lbtestcd)
        && let Some(ct) = ctx.resolve_ct(domain, "LBTESTCD")
    {
        let mut test_vals = string_column(df, &lbtest, Trim::Both)?;
        let testcd_vals = string_column(df, &lbtestcd, Trim::Both)?;
        for (test, testcd) in test_vals.iter_mut().zip(testcd_vals.iter()) {
            if testcd.is_empty() {
                continue;
            }
            let test_in_ct = resolve_ct_lenient(ct, test).is_some();
            let needs_label = test.is_empty() || test.eq_ignore_ascii_case(testcd) || !test_in_ct;
            if !needs_label {
                continue;
            }
            if let Some(preferred) = preferred_term_for(ct, testcd) {
                *test = preferred;
            }
        }
        set_string_column(df, &lbtest, test_vals)?;
    }
    if let Some(lbdtc) = col(domain, "LBDTC")
        && let Some(lbdy) = col(domain, "LBDY")
    {
        compute_study_day(domain, df, &lbdtc, &lbdy, ctx, "RFSTDTC")?;
    }
    if let Some(lbendtc) = col(domain, "LBENDTC")
        && let Some(lbendy) = col(domain, "LBENDY")
    {
        compute_study_day(domain, df, &lbendtc, &lbendy, ctx, "RFSTDTC")?;
    }
    if let Some(lbstresc) = col(domain, "LBSTRESC")
        && has_column(df, &lbstresc)
    {
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
    if let (Some(lborres), Some(lbstresc)) = (col(domain, "LBORRES"), col(domain, "LBSTRESC"))
        && has_column(df, &lborres)
        && has_column(df, &lbstresc)
    {
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
    if let (Some(lbstresc), Some(lbstresn)) = (col(domain, "LBSTRESC"), col(domain, "LBSTRESN"))
        && has_column(df, &lbstresc)
    {
        let stresc_vals = string_column(df, &lbstresc, Trim::Both)?;
        let numeric_vals = stresc_vals
            .iter()
            .map(|value| parse_f64(value))
            .collect::<Vec<_>>();
        set_f64_column(df, &lbstresn, numeric_vals)?;
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
            if let Some(name) = col(domain, col_name)
                && has_column(df, &name)
            {
                let mut values = string_column(df, &name, Trim::Both)?;
                for value in &mut values {
                    *value = normalize_ct_value_safe(ct, value);
                }
                set_string_column(df, &name, values)?;
            }
        }
    }
    if let Some(lbcolsrt) = col(domain, "LBCOLSRT")
        && has_column(df, &lbcolsrt)
    {
        let mut values = string_column(df, &lbcolsrt, Trim::Both)?;
        for value in &mut values {
            if is_yes_no_token(value) {
                value.clear();
            }
        }
        set_string_column(df, &lbcolsrt, values)?;
    }
    if let (Some(lborres), Some(lborresu)) = (col(domain, "LBORRES"), col(domain, "LBORRESU"))
        && has_column(df, &lborres)
        && has_column(df, &lborresu)
    {
        let orres = string_column(df, &lborres, Trim::Both)?;
        let mut orresu = string_column(df, &lborresu, Trim::Both)?;
        for (orres_val, orresu_val) in orres.iter().zip(orresu.iter_mut()) {
            if orres_val.is_empty() {
                orresu_val.clear();
            }
        }
        set_string_column(df, &lborresu, orresu)?;
    }
    if let (Some(lbstresc), Some(lbstresu)) = (col(domain, "LBSTRESC"), col(domain, "LBSTRESU"))
        && has_column(df, &lbstresc)
        && has_column(df, &lbstresu)
    {
        let stresc = string_column(df, &lbstresc, Trim::Both)?;
        let mut stresu = string_column(df, &lbstresu, Trim::Both)?;
        for (stresc_val, stresu_val) in stresc.iter().zip(stresu.iter_mut()) {
            if stresc_val.is_empty() {
                stresu_val.clear();
            }
        }
        set_string_column(df, &lbstresu, stresu)?;
    }
    Ok(())
}
