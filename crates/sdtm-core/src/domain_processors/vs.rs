use std::collections::HashMap;

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_vs(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if let Some(vsdtc) = col(domain, "VSDTC")
        && let Some(vsdy) = col(domain, "VSDY")
    {
        compute_study_day(domain, df, &vsdtc, &vsdy, context, "RFSTDTC")?;
        let values = numeric_column_f64(df, &vsdy)?;
        set_f64_column(df, &vsdy, values)?;
    }
    if let (Some(vsorres), Some(vsstresc)) = (col(domain, "VSORRES"), col(domain, "VSSTRESC"))
        && has_column(df, &vsorres)
        && has_column(df, &vsstresc)
    {
        let orres = string_column(df, &vsorres)?;
        let mut stresc = string_column(df, &vsstresc)?;
        for idx in 0..df.height() {
            if stresc[idx].is_empty() && !orres[idx].is_empty() {
                stresc[idx] = orres[idx].clone();
            }
        }
        set_string_column(df, &vsstresc, stresc)?;
    }
    if let (Some(vsorresu), Some(vsstresu)) = (col(domain, "VSORRESU"), col(domain, "VSSTRESU"))
        && has_column(df, &vsorresu)
        && has_column(df, &vsstresu)
    {
        let orresu = string_column(df, &vsorresu)?;
        let mut stresu = string_column(df, &vsstresu)?;
        for idx in 0..df.height() {
            if stresu[idx].is_empty() && !orresu[idx].is_empty() {
                stresu[idx] = orresu[idx].clone();
            }
        }
        set_string_column(df, &vsstresu, stresu)?;
    }
    if let (Some(vsorres), Some(vsorresu)) = (col(domain, "VSORRES"), col(domain, "VSORRESU"))
        && has_column(df, &vsorres)
        && has_column(df, &vsorresu)
    {
        let orres = string_column(df, &vsorres)?;
        let mut orresu = string_column(df, &vsorresu)?;
        for idx in 0..df.height() {
            if orres[idx].is_empty() {
                orresu[idx].clear();
            }
        }
        set_string_column(df, &vsorresu, orresu)?;
    }
    if let (Some(vsstresc), Some(vsstresu)) = (col(domain, "VSSTRESC"), col(domain, "VSSTRESU"))
        && has_column(df, &vsstresc)
        && has_column(df, &vsstresu)
    {
        let stresc = string_column(df, &vsstresc)?;
        let mut stresu = string_column(df, &vsstresu)?;
        for idx in 0..df.height() {
            if stresc[idx].is_empty() {
                stresu[idx].clear();
            }
        }
        set_string_column(df, &vsstresu, stresu)?;
    }
    if let (Some(vstest), Some(vstestcd)) = (col(domain, "VSTEST"), col(domain, "VSTESTCD"))
        && has_column(df, &vstest)
        && has_column(df, &vstestcd)
    {
        let mut test_vals = string_column(df, &vstest)?;
        let testcd_vals = string_column(df, &vstestcd)?;
        for idx in 0..df.height() {
            if test_vals[idx].is_empty() && !testcd_vals[idx].is_empty() {
                test_vals[idx] = testcd_vals[idx].clone();
            }
        }
        set_string_column(df, &vstest, test_vals)?;
    }
    if let (Some(vstest), Some(vstestcd)) = (col(domain, "VSTEST"), col(domain, "VSTESTCD"))
        && has_column(df, &vstest)
        && has_column(df, &vstestcd)
        && let Some(ct) = context.resolve_ct(domain, "VSTESTCD")
    {
        let test_vals = string_column(df, &vstest)?;
        let mut testcd_vals = string_column(df, &vstestcd)?;
        for (testcd, test) in testcd_vals.iter_mut().zip(test_vals.iter()) {
            let existing = testcd.clone();
            let valid =
                !existing.is_empty() && ct.submission_values().iter().any(|val| val == &existing);
            if valid {
                continue;
            }
            if let Some(mapped) = resolve_ct_value(ct, test, context.options.ct_matching) {
                *testcd = mapped;
            }
        }
        set_string_column(df, &vstestcd, testcd_vals)?;
    }
    if let Some(ct) = context.resolve_ct(domain, "VSORRESU") {
        for col_name in ["VSORRESU", "VSSTRESU"] {
            if let Some(name) = col(domain, col_name)
                && has_column(df, &name)
            {
                let mut values = string_column(df, &name)?;
                for value in &mut values {
                    *value = normalize_ct_value(ct, value, context.options.ct_matching);
                }
                set_string_column(df, &name, values)?;
            }
        }
    }
    if let Some(ct) = context.resolve_ct(domain, "VSTESTCD")
        && let Some(vstestcd) = col(domain, "VSTESTCD")
        && has_column(df, &vstestcd)
    {
        let mut values = string_column(df, &vstestcd)?;
        for value in &mut values {
            *value = normalize_ct_value(ct, value, context.options.ct_matching);
        }
        set_string_column(df, &vstestcd, values)?;
    }
    if let Some(ct) = context.resolve_ct(domain, "VSTEST")
        && let Some(vstest) = col(domain, "VSTEST")
        && has_column(df, &vstest)
    {
        let mut values = string_column(df, &vstest)?;
        for value in &mut values {
            *value = normalize_ct_value(ct, value, context.options.ct_matching);
        }
        set_string_column(df, &vstest, values)?;
    }
    if let (Some(vstest), Some(vstestcd)) = (col(domain, "VSTEST"), col(domain, "VSTESTCD"))
        && has_column(df, &vstest)
        && has_column(df, &vstestcd)
        && let Some(ct) = context.resolve_ct(domain, "VSTESTCD")
    {
        let ct_names = context.resolve_ct(domain, "VSTEST");
        let mut test_vals = string_column(df, &vstest)?;
        let testcd_vals = string_column(df, &vstestcd)?;
        for (test, testcd) in test_vals.iter_mut().zip(testcd_vals.iter()) {
            if testcd.is_empty() {
                continue;
            }
            let needs_label = test.is_empty() || test.eq_ignore_ascii_case(testcd);
            let valid_name = ct_names
                .map(|ct| {
                    let canonical = normalize_ct_value(ct, test, context.options.ct_matching);
                    ct.submission_values().iter().any(|val| val == &canonical)
                })
                .unwrap_or(true);
            if !needs_label && valid_name {
                continue;
            }
            if let Some(preferred) = preferred_term_for(ct, testcd) {
                *test = preferred;
            }
        }
        set_string_column(df, &vstest, test_vals)?;
    }
    if let (Some(vsorres), Some(vsstresn)) = (col(domain, "VSORRES"), col(domain, "VSSTRESN"))
        && has_column(df, &vsorres)
    {
        let orres_vals = string_column(df, &vsorres)?;
        let numeric_vals = orres_vals
            .iter()
            .map(|value| parse_f64(value))
            .collect::<Vec<_>>();
        set_f64_column(df, &vsstresn, numeric_vals)?;
    }
    if let Some(vslobxfl) = col(domain, "VSLOBXFL")
        && let (Some(usubjid), Some(vstestcd)) = (col(domain, "USUBJID"), col(domain, "VSTESTCD"))
        && has_column(df, &vslobxfl)
        && has_column(df, &usubjid)
        && has_column(df, &vstestcd)
    {
        let mut flags = vec![String::new(); df.height()];
        let usub_vals = string_column(df, &usubjid)?;
        let test_vals = string_column(df, &vstestcd)?;
        let pos_vals = col(domain, "VSPOS")
            .filter(|name| has_column(df, name))
            .and_then(|name| string_column(df, &name).ok());
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
    if let Some(vseltm) = col(domain, "VSELTM")
        && has_column(df, &vseltm)
    {
        let values = string_column(df, &vseltm)?
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
    Ok(())
}

fn is_valid_time(value: &str) -> bool {
    let trimmed = value.trim();
    let parts: Vec<&str> = trimmed.split(':').collect();
    match parts.as_slice() {
        [hh, mm] => {
            hh.len() == 2 && mm.len() == 2 && hh.parse::<u32>().is_ok() && mm.parse::<u32>().is_ok()
        }
        [hh, mm, ss] => {
            hh.len() == 2
                && mm.len() == 2
                && ss.len() == 2
                && hh.parse::<u32>().is_ok()
                && mm.parse::<u32>().is_ok()
                && ss.parse::<u32>().is_ok()
        }
        _ => false,
    }
}
