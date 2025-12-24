use std::collections::HashMap;

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_vs(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    if let Some(vsdtc) = col(domain, "VSDTC") {
        if let Some(vsdy) = col(domain, "VSDY") {
            compute_study_day(domain, df, &vsdtc, &vsdy, ctx, "RFSTDTC")?;
            let values = numeric_column_f64(df, &vsdy)?;
            set_f64_column(df, &vsdy, values)?;
        }
    }
    if let (Some(vsorres), Some(vsstresc)) = (col(domain, "VSORRES"), col(domain, "VSSTRESC")) {
        if has_column(df, &vsorres) && has_column(df, &vsstresc) {
            let orres = string_column(df, &vsorres, Trim::Both)?;
            let mut stresc = string_column(df, &vsstresc, Trim::Both)?;
            for idx in 0..df.height() {
                if stresc[idx].is_empty() && !orres[idx].is_empty() {
                    stresc[idx] = orres[idx].clone();
                }
            }
            set_string_column(df, &vsstresc, stresc)?;
        }
    }
    if let (Some(vsorresu), Some(vsstresu)) = (col(domain, "VSORRESU"), col(domain, "VSSTRESU")) {
        if has_column(df, &vsorresu) && has_column(df, &vsstresu) {
            let orresu = string_column(df, &vsorresu, Trim::Both)?;
            let mut stresu = string_column(df, &vsstresu, Trim::Both)?;
            for idx in 0..df.height() {
                if stresu[idx].is_empty() && !orresu[idx].is_empty() {
                    stresu[idx] = orresu[idx].clone();
                }
            }
            set_string_column(df, &vsstresu, stresu)?;
        }
    }
    if let (Some(vsorres), Some(vsorresu)) = (col(domain, "VSORRES"), col(domain, "VSORRESU")) {
        if has_column(df, &vsorres) && has_column(df, &vsorresu) {
            let orres = string_column(df, &vsorres, Trim::Both)?;
            let mut orresu = string_column(df, &vsorresu, Trim::Both)?;
            for idx in 0..df.height() {
                if orres[idx].is_empty() {
                    orresu[idx].clear();
                }
            }
            set_string_column(df, &vsorresu, orresu)?;
        }
    }
    if let (Some(vsstresc), Some(vsstresu)) = (col(domain, "VSSTRESC"), col(domain, "VSSTRESU")) {
        if has_column(df, &vsstresc) && has_column(df, &vsstresu) {
            let stresc = string_column(df, &vsstresc, Trim::Both)?;
            let mut stresu = string_column(df, &vsstresu, Trim::Both)?;
            for idx in 0..df.height() {
                if stresc[idx].is_empty() {
                    stresu[idx].clear();
                }
            }
            set_string_column(df, &vsstresu, stresu)?;
        }
    }
    if let Some(ct) = ctx.resolve_ct(domain, "VSORRESU") {
        for col_name in ["VSORRESU", "VSSTRESU"] {
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
    if let Some(ct) = ctx.resolve_ct(domain, "VSTESTCD") {
        if let Some(vstestcd) = col(domain, "VSTESTCD") {
            if has_column(df, &vstestcd) {
                let mut values = string_column(df, &vstestcd, Trim::Both)?;
                for idx in 0..values.len() {
                    let canonical = normalize_ct_value(ct, &values[idx]);
                    let valid = ct.submission_values.iter().any(|val| val == &canonical);
                    values[idx] = if valid { canonical } else { "".to_string() };
                }
                set_string_column(df, &vstestcd, values)?;
            }
        }
    }
    if let Some(ct) = ctx.resolve_ct(domain, "VSTEST") {
        if let Some(vstest) = col(domain, "VSTEST") {
            if has_column(df, &vstest) {
                let mut values = string_column(df, &vstest, Trim::Both)?;
                for idx in 0..values.len() {
                    let canonical = normalize_ct_value(ct, &values[idx]);
                    let valid = ct.submission_values.iter().any(|val| val == &canonical);
                    values[idx] = if valid { canonical } else { "".to_string() };
                }
                set_string_column(df, &vstest, values)?;
            }
        }
    }
    if let (Some(vsorres), Some(vsstresn)) = (col(domain, "VSORRES"), col(domain, "VSSTRESN")) {
        if has_column(df, &vsorres) {
            let orres_vals = string_column(df, &vsorres, Trim::Both)?;
            let numeric_vals = orres_vals
                .iter()
                .map(|value| parse_f64(value))
                .collect::<Vec<_>>();
            set_f64_column(df, &vsstresn, numeric_vals)?;
        }
    }
    if let (Some(vsseq), Some(usubjid)) = (col(domain, "VSSEQ"), col(domain, "USUBJID")) {
        assign_sequence(df, &vsseq, &usubjid)?;
    }
    if let Some(vslobxfl) = col(domain, "VSLOBXFL") {
        if let (Some(usubjid), Some(vstestcd)) = (col(domain, "USUBJID"), col(domain, "VSTESTCD")) {
            if has_column(df, &vslobxfl) && has_column(df, &usubjid) && has_column(df, &vstestcd) {
                let mut flags = vec![String::new(); df.height()];
                let usub_vals = string_column(df, &usubjid, Trim::Both)?;
                let test_vals = string_column(df, &vstestcd, Trim::Both)?;
                let pos_vals = col(domain, "VSPOS")
                    .filter(|name| has_column(df, name))
                    .map(|name| string_column(df, &name, Trim::Both).ok())
                    .flatten();
                let mut last_idx: HashMap<String, usize> = HashMap::new();
                for idx in 0..df.height() {
                    let mut key = format!("{}|{}", usub_vals[idx], test_vals[idx]);
                    if let Some(pos_vals) = pos_vals.as_ref() {
                        key.push('|');
                        key.push_str(&pos_vals[idx]);
                    }
                    last_idx.insert(key, idx);
                }
                for (_, idx) in last_idx {
                    flags[idx] = "Y".to_string();
                }
                set_string_column(df, &vslobxfl, flags)?;
            }
        }
    }
    if let Some(vseltm) = col(domain, "VSELTM") {
        if has_column(df, &vseltm) {
            let values = string_column(df, &vseltm, Trim::Both)?
                .into_iter()
                .map(|value| {
                    if is_valid_time(&value) {
                        value
                    } else {
                        String::new()
                    }
                })
                .collect();
            set_string_column(df, &vseltm, values)?;
        }
    }
    Ok(())
}
