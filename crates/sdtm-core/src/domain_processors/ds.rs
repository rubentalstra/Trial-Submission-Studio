use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_ds(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    for col_name in ["DSDECOD", "DSTERM", "DSCAT", "EPOCH"] {
        if let Some(name) = col(domain, col_name) {
            if has_column(df, &name) {
                let values = string_column(df, &name, Trim::Both)?;
                set_string_column(df, &name, values)?;
            }
        }
    }
    if let (Some(usubjid), Some(dsdecod), Some(dsterm)) = (
        col(domain, "USUBJID"),
        col(domain, "DSDECOD"),
        col(domain, "DSTERM"),
    ) {
        if has_column(df, &usubjid) && has_column(df, &dsdecod) && has_column(df, &dsterm) {
            let usub_vals = string_column(df, &usubjid, Trim::Both)?;
            let mut decod_vals = string_column(df, &dsdecod, Trim::Both)?;
            let mut term_vals = string_column(df, &dsterm, Trim::Both)?;
            for idx in 0..df.height() {
                let site_part = usub_vals[idx]
                    .split('-')
                    .rev()
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_uppercase();
                if site_part.is_empty() {
                    continue;
                }
                let decod_upper = decod_vals[idx].to_uppercase();
                let term_upper = term_vals[idx].to_uppercase();
                if decod_upper == site_part {
                    let screen_failure = term_upper.contains("SCREEN FAILURE")
                        || term_upper.contains("FAILURE TO MEET");
                    if screen_failure {
                        decod_vals[idx] = "SCREEN FAILURE".to_string();
                        term_vals[idx] = "SCREEN FAILURE".to_string();
                    } else {
                        decod_vals[idx].clear();
                        if term_upper == site_part {
                            term_vals[idx].clear();
                        }
                    }
                }
            }
            set_string_column(df, &dsdecod, decod_vals)?;
            set_string_column(df, &dsterm, term_vals)?;
        }
    }
    if let (Some(dscat), Some(dsterm)) = (col(domain, "DSCAT"), col(domain, "DSTERM")) {
        if has_column(df, &dscat) && has_column(df, &dsterm) {
            let dscat_vals = string_column(df, &dscat, Trim::Both)?;
            let mut dsterm_vals = string_column(df, &dsterm, Trim::Both)?;
            let looks_like_site: Vec<bool> = dsterm_vals
                .iter()
                .map(|value| value.to_uppercase().contains("SITE"))
                .collect();
            let ct_dscat = ctx.resolve_ct(domain, "DSCAT");
            let mut invalid = vec![false; df.height()];
            if let Some(ct) = ct_dscat {
                for idx in 0..df.height() {
                    if dscat_vals[idx].is_empty() {
                        continue;
                    }
                    let canonical = normalize_ct_value(ct, &dscat_vals[idx]);
                    let is_valid = ct.submission_values.iter().any(|val| val == &canonical);
                    invalid[idx] = !is_valid;
                }
            } else {
                for idx in 0..df.height() {
                    invalid[idx] = !dscat_vals[idx].is_empty();
                }
            }
            for idx in 0..df.height() {
                if !invalid[idx] {
                    continue;
                }
                let move_reason = dsterm_vals[idx].is_empty() || looks_like_site[idx];
                if move_reason {
                    dsterm_vals[idx] = dscat_vals[idx].clone();
                }
            }
            set_string_column(df, &dscat, dscat_vals)?;
            set_string_column(df, &dsterm, dsterm_vals)?;
        }
    }
    if let (Some(dsdecod), Some(dsterm)) = (col(domain, "DSDECOD"), col(domain, "DSTERM")) {
        if has_column(df, &dsdecod) && has_column(df, &dsterm) {
            let mut decod_vals = string_column(df, &dsdecod, Trim::Both)?;
            let term_vals = string_column(df, &dsterm, Trim::Both)?;
            for idx in 0..df.height() {
                if !decod_vals[idx].is_empty() {
                    continue;
                }
                let term_upper = term_vals[idx].to_uppercase();
                if term_upper.contains("SCREEN FAILURE") || term_upper.contains("FAILURE TO MEET") {
                    decod_vals[idx] = "SCREEN FAILURE".to_string();
                } else if term_upper.contains("WITHDRAW") && term_upper.contains("CONSENT") {
                    decod_vals[idx] = "WITHDRAWAL OF CONSENT".to_string();
                } else if term_upper.contains("WITHDRAW") && term_upper.contains("SUBJECT") {
                    decod_vals[idx] = "WITHDRAWAL BY SUBJECT".to_string();
                } else if term_upper.contains("LOST") && term_upper.contains("FOLLOW") {
                    decod_vals[idx] = "LOST TO FOLLOW-UP".to_string();
                }
            }
            set_string_column(df, &dsdecod, decod_vals)?;
        }
    }
    if let (Some(dsterm), Some(dsdecod)) = (col(domain, "DSTERM"), col(domain, "DSDECOD")) {
        if has_column(df, &dsterm) && has_column(df, &dsdecod) {
            let mut term_vals = string_column(df, &dsterm, Trim::Both)?;
            let decod_vals = string_column(df, &dsdecod, Trim::Both)?;
            for idx in 0..df.height() {
                let term_upper = term_vals[idx].to_uppercase();
                if term_upper.contains("SITE") && !decod_vals[idx].is_empty() {
                    term_vals[idx] = decod_vals[idx].clone();
                }
            }
            set_string_column(df, &dsterm, term_vals)?;
        }
    }
    if let (Some(dsterm), Some(dsdecod)) = (col(domain, "DSTERM"), col(domain, "DSDECOD")) {
        if has_column(df, &dsterm) && has_column(df, &dsdecod) {
            let mut term_vals = string_column(df, &dsterm, Trim::Both)?;
            let mut decod_vals = string_column(df, &dsdecod, Trim::Both)?;
            for idx in 0..df.height() {
                if term_vals[idx].is_empty() && !decod_vals[idx].is_empty() {
                    term_vals[idx] = decod_vals[idx].clone();
                } else if decod_vals[idx].is_empty() && !term_vals[idx].is_empty() {
                    decod_vals[idx] = term_vals[idx].clone();
                }
            }
            set_string_column(df, &dsterm, term_vals)?;
            set_string_column(df, &dsdecod, decod_vals)?;
        }
    }
    if let Some(dsdecod) = col(domain, "DSDECOD") {
        if has_column(df, &dsdecod) {
            if let Some(ct) = ctx.resolve_ct(domain, "DSDECOD") {
                let mut decod_vals = string_column(df, &dsdecod, Trim::Both)?;
                for idx in 0..df.height() {
                    let canonical = normalize_ct_value(ct, &decod_vals[idx]);
                    let mapped = match canonical.to_uppercase().as_str() {
                        "Y" | "YES" => "COMPLETED".to_string(),
                        "N" | "NO" => "SCREENING NOT COMPLETED".to_string(),
                        _ => canonical.to_uppercase(),
                    };
                    decod_vals[idx] = mapped;
                }
                if let Some(dsterm) = col(domain, "DSTERM") {
                    if has_column(df, &dsterm) {
                        let term_vals = string_column(df, &dsterm, Trim::Both)?;
                        for idx in 0..df.height() {
                            let term_code = normalize_ct_value(ct, &term_vals[idx]).to_uppercase();
                            let valid_from_term =
                                ct.submission_values.iter().any(|val| val == &term_code);
                            let valid_from_decod = ct
                                .submission_values
                                .iter()
                                .any(|val| val == &decod_vals[idx]);
                            if valid_from_term && !valid_from_decod {
                                decod_vals[idx] = term_code;
                            }
                        }
                    }
                }
                set_string_column(df, &dsdecod, decod_vals)?;
            }
        }
    }
    if let Some(dsstdtc) = col(domain, "DSSTDTC") {
        if has_column(df, &dsstdtc) {
            let values = string_column(df, &dsstdtc, Trim::Both)?
                .into_iter()
                .map(|value| coerce_iso8601(&value))
                .collect();
            set_string_column(df, &dsstdtc, values)?;
            if let Some(dsstudy) = col(domain, "DSSTDY") {
                compute_study_day(domain, df, &dsstdtc, &dsstudy, ctx, "RFSTDTC")?;
            }
        }
    }
    if let Some(dsdtc) = col(domain, "DSDTC") {
        if has_column(df, &dsdtc) {
            let values = string_column(df, &dsdtc, Trim::Both)?
                .into_iter()
                .map(|value| coerce_iso8601(&value))
                .collect();
            set_string_column(df, &dsdtc, values)?;
            if let Some(dsdy) = col(domain, "DSDY") {
                compute_study_day(domain, df, &dsdtc, &dsdy, ctx, "RFSTDTC")?;
            }
        }
    }
    if let (Some(dsseq), Some(usubjid)) = (col(domain, "DSSEQ"), col(domain, "USUBJID")) {
        assign_sequence(df, &dsseq, &usubjid)?;
    }
    let dedup_keys = ["USUBJID", "DSDECOD", "DSTERM", "DSCAT", "DSSTDTC"]
        .into_iter()
        .filter_map(|name| col(domain, name))
        .filter(|name| has_column(df, name))
        .collect::<Vec<_>>();
    if !dedup_keys.is_empty() {
        deduplicate(df, &dedup_keys)?;
    }
    Ok(())
}
