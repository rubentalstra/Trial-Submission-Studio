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
    // Compute study day
    if let Some(vsdtc) = col(domain, "VSDTC")
        && let Some(vsdy) = col(domain, "VSDY")
    {
        compute_study_day(domain, df, vsdtc, vsdy, context, "RFSTDTC")?;
        let values = numeric_column_f64(df, vsdy)?;
        set_f64_column(df, vsdy, values)?;
    }

    // Backward fill: VSORRES → VSSTRESC
    backward_fill_var(domain, df, "VSORRES", "VSSTRESC")?;

    // Backward fill: VSORRESU → VSSTRESU
    backward_fill_var(domain, df, "VSORRESU", "VSSTRESU")?;

    // Clear unit when result is empty (original)
    clear_unit_when_empty_var(domain, df, "VSORRES", "VSORRESU")?;

    // Clear unit when result is empty (standardized)
    clear_unit_when_empty_var(domain, df, "VSSTRESC", "VSSTRESU")?;

    // Backward fill: VSTESTCD → VSTEST
    backward_fill_var(domain, df, "VSTESTCD", "VSTEST")?;

    // Resolve VSTESTCD from VSTEST when invalid
    resolve_testcd_from_test(domain, df, context, "VSTESTCD", "VSTEST", "VSTESTCD")?;

    // Normalize unit columns via CT
    normalize_ct_columns(domain, df, context, "VSORRESU", &["VSORRESU", "VSSTRESU"])?;

    // Normalize VSTESTCD via CT
    normalize_ct_columns(domain, df, context, "VSTESTCD", &["VSTESTCD"])?;

    // Normalize VSTEST via CT
    normalize_ct_columns(domain, df, context, "VSTEST", &["VSTEST"])?;

    // Derive VSTEST from VSTESTCD using CT preferred terms
    derive_test_from_testcd(domain, df, context, "VSTEST", "VSTESTCD", "VSTESTCD")?;

    // Derive numeric result from VSORRES
    if let (Some(vsorres), Some(vsstresn)) = (col(domain, "VSORRES"), col(domain, "VSSTRESN"))
        && has_column(df, vsorres)
    {
        let orres_vals = string_column(df, vsorres)?;
        let numeric_vals = orres_vals
            .iter()
            .map(|value| parse_f64(value))
            .collect::<Vec<_>>();
        set_f64_column(df, vsstresn, numeric_vals)?;
    }

    // Compute VSLOBXFL (Last Observation Before flag)
    if let Some(vslobxfl) = col(domain, "VSLOBXFL")
        && let (Some(usubjid), Some(vstestcd)) = (col(domain, "USUBJID"), col(domain, "VSTESTCD"))
        && has_column(df, vslobxfl)
        && has_column(df, usubjid)
        && has_column(df, vstestcd)
    {
        let mut flags = vec![String::new(); df.height()];
        let usub_vals = string_column(df, usubjid)?;
        let test_vals = string_column(df, vstestcd)?;
        let pos_vals = col(domain, "VSPOS")
            .filter(|&name| has_column(df, name))
            .and_then(|name| string_column(df, name).ok());

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
        set_string_column(df, vslobxfl, flags)?;
    }

    // Validate VSELTM time format
    if let Some(vseltm) = col(domain, "VSELTM")
        && has_column(df, vseltm)
    {
        let values = string_column(df, vseltm)?
            .into_iter()
            .map(|value| {
                if is_valid_time(&value) {
                    value
                } else {
                    String::new()
                }
            })
            .collect();
        set_string_column(df, vseltm, values)?;
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
