use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_qs(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    if let (Some(qsseq), Some(usubjid)) = (col(domain, "QSSEQ"), col(domain, "USUBJID")) {
        assign_sequence(df, &qsseq, &usubjid)?;
    }
    for col_name in [
        "QSTESTCD", "QSTEST", "QSCAT", "QSSCAT", "QSORRES", "QSSTRESC", "QSLOBXFL", "VISIT",
        "EPOCH",
    ] {
        if let Some(name) = col(domain, col_name) {
            if has_column(df, &name) {
                let values = string_column(df, &name, Trim::Both)?;
                set_string_column(df, &name, values)?;
            }
        }
    }

    let mut pga_score: Option<Vec<String>> = None;
    let mut score_from_qsgrpid = false;
    if let Some(qspgars) = col(domain, "QSPGARS") {
        if has_column(df, &qspgars) {
            pga_score = Some(string_column(df, &qspgars, Trim::Both)?);
        }
    }
    if pga_score.is_none() {
        if let Some(qspgarscd) = col(domain, "QSPGARSCD") {
            if has_column(df, &qspgarscd) {
                pga_score = Some(string_column(df, &qspgarscd, Trim::Both)?);
            }
        }
    }
    if pga_score.is_none() {
        if let (Some(qsorres), Some(qsgrpid)) = (col(domain, "QSORRES"), col(domain, "QSGRPID")) {
            if has_column(df, &qsorres) && has_column(df, &qsgrpid) {
                let orres_vals = string_column(df, &qsorres, Trim::Both)?;
                let grpid_vals = string_column(df, &qsgrpid, Trim::Both)?;
                let all_orres_empty = orres_vals.iter().all(|value| value.is_empty());
                let any_grpid = grpid_vals.iter().any(|value| !value.is_empty());
                if all_orres_empty && any_grpid {
                    pga_score = Some(grpid_vals);
                    score_from_qsgrpid = true;
                }
            }
        }
    }

    if let Some(score) = pga_score {
        if let Some(qsorres) = col(domain, "QSORRES") {
            let mut orres_vals = if has_column(df, &qsorres) {
                string_column(df, &qsorres, Trim::Both)?
            } else {
                vec![String::new(); df.height()]
            };
            for idx in 0..df.height() {
                if orres_vals[idx].is_empty() {
                    orres_vals[idx] = score[idx].clone();
                }
            }
            set_string_column(df, &qsorres, orres_vals)?;
        }
        if score_from_qsgrpid {
            if let Some(qsgrpid) = col(domain, "QSGRPID") {
                if has_column(df, &qsgrpid) {
                    let cleared = vec![String::new(); df.height()];
                    set_string_column(df, &qsgrpid, cleared)?;
                }
            }
        }
        if let Some(qstestcd) = col(domain, "QSTESTCD") {
            if has_column(df, &qstestcd) {
                let mut values = string_column(df, &qstestcd, Trim::Both)?;
                if let Some(usubjid) = col(domain, "USUBJID") {
                    if has_column(df, &usubjid) {
                        let usub_vals = string_column(df, &usubjid, Trim::Both)?;
                        for idx in 0..df.height() {
                            if values[idx].is_empty() {
                                values[idx] = "PGAS".to_string();
                                continue;
                            }
                            let site_part = usub_vals[idx]
                                .split('-')
                                .nth(1)
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            if !site_part.is_empty() && values[idx] == site_part {
                                values[idx] = "PGAS".to_string();
                            }
                        }
                    }
                } else {
                    for value in &mut values {
                        if value.is_empty() {
                            *value = "PGAS".to_string();
                        }
                    }
                }
                set_string_column(df, &qstestcd, values)?;
            }
        }
        if let Some(qstest) = col(domain, "QSTEST") {
            if has_column(df, &qstest) {
                let mut values = string_column(df, &qstest, Trim::Both)?;
                for value in &mut values {
                    if value.is_empty() {
                        *value = "PHYSICIAN GLOBAL ASSESSMENT".to_string();
                    }
                }
                set_string_column(df, &qstest, values)?;
            }
        }
        if let Some(qscat) = col(domain, "QSCAT") {
            if has_column(df, &qscat) {
                let mut values = string_column(df, &qscat, Trim::Both)?;
                for value in &mut values {
                    if value.is_empty() {
                        *value = "PGI".to_string();
                    }
                }
                set_string_column(df, &qscat, values)?;
            }
        }
    }

    if let (Some(qsstresc), Some(qsorres)) = (col(domain, "QSSTRESC"), col(domain, "QSORRES")) {
        if has_column(df, &qsstresc) && has_column(df, &qsorres) {
            let orres = string_column(df, &qsorres, Trim::Both)?;
            let mut stresc = string_column(df, &qsstresc, Trim::Both)?;
            for idx in 0..df.height() {
                if stresc[idx].is_empty() {
                    stresc[idx] = orres[idx].clone();
                }
            }
            set_string_column(df, &qsstresc, stresc)?;
        }
    }
    if let Some(qslobxfl) = col(domain, "QSLOBXFL") {
        if has_column(df, &qslobxfl) {
            let values = string_column(df, &qslobxfl, Trim::Both)?
                .into_iter()
                .map(|value| if value == "N" { "".to_string() } else { value })
                .collect();
            set_string_column(df, &qslobxfl, values)?;
        }
    }
    if let Some(qsdtc) = col(domain, "QSDTC") {
        if has_column(df, &qsdtc) {
            let values = string_column(df, &qsdtc, Trim::Both)?
                .into_iter()
                .map(|value| coerce_iso8601(&value))
                .collect();
            set_string_column(df, &qsdtc, values)?;
            if let Some(qsdy) = col(domain, "QSDY") {
                compute_study_day(domain, df, &qsdtc, &qsdy, ctx, "RFSTDTC")?;
            }
        }
    }
    if let Some(qstptref) = col(domain, "QSTPTREF") {
        if has_column(df, &qstptref) {
            let has_timing = ["QSELTM", "QSTPTNUM", "QSTPT"]
                .into_iter()
                .filter_map(|name| col(domain, name))
                .any(|name| has_column(df, &name));
            if !has_timing {
                let values = vec![String::new(); df.height()];
                set_string_column(df, &qstptref, values)?;
            }
        }
    }
    Ok(())
}
