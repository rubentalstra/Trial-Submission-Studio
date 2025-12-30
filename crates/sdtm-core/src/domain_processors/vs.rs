//! Vital Signs (VS) domain processor.
//!
//! Processes VS domain data per SDTMIG v3.4 Section 6.3.8.

use std::collections::HashMap;

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    backward_fill_batch, clear_units_batch, col, compute_study_days_batch, derive_test_from_testcd,
    has_column, normalize_ct_batch, normalize_ct_columns, normalize_numeric_f64, parse_f64,
    resolve_testcd_from_test, set_f64_column, set_string_column, string_column,
};

pub(super) fn process_vs(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Compute study day
    compute_study_days_batch(domain, df, context, &[("VSDTC", "VSDY")])?;
    normalize_numeric_f64(domain, df, &["VSDY"])?;

    // Backward fills
    backward_fill_batch(
        domain,
        df,
        &[
            ("VSORRES", "VSSTRESC"),
            ("VSORRESU", "VSSTRESU"),
            ("VSTESTCD", "VSTEST"),
        ],
    )?;

    // Clear unit when result is empty
    clear_units_batch(
        domain,
        df,
        &[("VSORRES", "VSORRESU"), ("VSSTRESC", "VSSTRESU")],
    )?;

    // Resolve and derive test code/name
    resolve_testcd_from_test(domain, df, context, "VSTESTCD", "VSTEST", "VSTESTCD")?;
    normalize_ct_columns(domain, df, context, "VSORRESU", &["VSORRESU", "VSSTRESU"])?;
    normalize_ct_columns(domain, df, context, "VSTESTCD", &["VSTESTCD"])?;
    normalize_ct_columns(domain, df, context, "VSTEST", &["VSTEST"])?;
    derive_test_from_testcd(domain, df, context, "VSTEST", "VSTESTCD", "VSTESTCD")?;

    // Batch CT normalization
    normalize_ct_batch(
        domain,
        df,
        context,
        &[
            "VSCAT", "VSSCAT", "VSPOS", "VSLOC", "VSLAT", "VSMETHOD", "EPOCH",
        ],
    )?;

    // Derive numeric result from VSORRES
    if let (Some(vsorres), Some(vsstresn)) = (col(domain, "VSORRES"), col(domain, "VSSTRESN"))
        && has_column(df, vsorres)
    {
        let orres_vals = string_column(df, vsorres)?;
        let numeric_vals = orres_vals.iter().map(|v| parse_f64(v)).collect();
        set_f64_column(df, vsstresn, numeric_vals)?;
    }

    // Compute VSLOBXFL (Last Observation Before flag)
    compute_vslobxfl(domain, df)?;

    // Validate VSELTM time format
    validate_vseltm(domain, df)?;

    Ok(())
}

/// Compute VSLOBXFL (Last Observation Before flag) per SDTMIG v3.4.
fn compute_vslobxfl(domain: &Domain, df: &mut DataFrame) -> Result<()> {
    let vslobxfl = match col(domain, "VSLOBXFL") {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };
    let usubjid = match col(domain, "USUBJID") {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };
    let vstestcd = match col(domain, "VSTESTCD") {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };

    let mut flags = vec![String::new(); df.height()];
    let usub_vals = string_column(df, usubjid)?;
    let test_vals = string_column(df, vstestcd)?;
    let pos_vals = col(domain, "VSPOS")
        .filter(|&name| has_column(df, name))
        .and_then(|name| string_column(df, name).ok());

    // Track last index for each subject|test|position combination
    let mut last_idx: HashMap<String, usize> = HashMap::new();
    for idx in 0..df.height() {
        let mut key = format!("{}|{}", usub_vals[idx], test_vals[idx]);
        if let Some(ref pos) = pos_vals {
            key.push('|');
            key.push_str(&pos[idx]);
        }
        last_idx.insert(key, idx);
    }

    for idx in last_idx.into_values() {
        flags[idx] = "Y".to_string();
    }
    set_string_column(df, vslobxfl, flags)?;

    Ok(())
}

/// Validate and clear invalid VSELTM time values.
fn validate_vseltm(domain: &Domain, df: &mut DataFrame) -> Result<()> {
    let vseltm = match col(domain, "VSELTM") {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };

    let values = string_column(df, vseltm)?
        .into_iter()
        .map(|v| if is_valid_time(&v) { v } else { String::new() })
        .collect();
    set_string_column(df, vseltm, values)?;

    Ok(())
}

/// Check if a string is a valid HH:MM or HH:MM:SS time format.
fn is_valid_time(value: &str) -> bool {
    let parts: Vec<&str> = value.trim().split(':').collect();
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
