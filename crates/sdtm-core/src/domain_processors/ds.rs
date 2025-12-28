use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;
use sdtm_model::ct::Codelist;

use crate::processing_context::ProcessingContext;

use super::common::*;

fn split_codelist_codes(raw: &str) -> Vec<String> {
    let text = raw.trim();
    if text.is_empty() {
        return Vec::new();
    }
    for sep in [';', ',', ' '] {
        if text.contains(sep) {
            return text
                .split(sep)
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect();
        }
    }
    vec![text.to_string()]
}

fn dsdecod_codelists(domain: &Domain) -> Vec<String> {
    domain
        .variables
        .iter()
        .find(|var| var.name.eq_ignore_ascii_case("DSDECOD"))
        .and_then(|var| var.codelist_code.as_ref())
        .map(|raw| split_codelist_codes(raw))
        .unwrap_or_default()
}

fn resolve_dsdecod_codelist<'a>(
    ctx: &'a ProcessingContext<'a>,
    codes: &[String],
    value: &str,
) -> Option<&'a Codelist> {
    let registry = ctx.ct_registry?;
    for code in codes {
        if let Some(resolved) = registry.resolve(code, None)
            && resolve_ct_lenient(resolved.codelist, value).is_some()
        {
            return Some(resolved.codelist);
        }
    }
    None
}

fn dscat_value_for_codelist(dscat_ct: &Codelist, dsdecod_ct: &Codelist) -> Option<String> {
    let name_upper = dsdecod_ct.name.to_uppercase();
    let hint = if name_upper.contains("MILESTONE") {
        "PROTOCOL MILESTONE"
    } else if name_upper.contains("OTHER") {
        "OTHER EVENT"
    } else {
        "DISPOSITION EVENT"
    };
    resolve_ct_lenient(dscat_ct, hint).or_else(|| {
        let upper = hint.to_uppercase();
        dscat_ct
            .submission_values()
            .iter()
            .find(|val| val.to_uppercase() == upper)
            .map(|v| v.to_string())
    })
}

pub(super) fn process_ds(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    for col_name in ["DSDECOD", "DSTERM", "DSCAT", "EPOCH"] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, &name)
        {
            let values = string_column(df, &name)?;
            set_string_column(df, &name, values)?;
        }
    }
    if let (Some(usubjid), Some(dsdecod), Some(dsterm)) = (
        col(domain, "USUBJID"),
        col(domain, "DSDECOD"),
        col(domain, "DSTERM"),
    ) && has_column(df, &usubjid)
        && has_column(df, &dsdecod)
        && has_column(df, &dsterm)
    {
        let usub_vals = string_column(df, &usubjid)?;
        let mut decod_vals = string_column(df, &dsdecod)?;
        let mut term_vals = string_column(df, &dsterm)?;
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
                let screen_failure =
                    term_upper.contains("SCREEN FAILURE") || term_upper.contains("FAILURE TO MEET");
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
    if let (Some(dscat), Some(dsterm)) = (col(domain, "DSCAT"), col(domain, "DSTERM"))
        && has_column(df, &dscat)
        && has_column(df, &dsterm)
    {
        let dscat_vals = string_column(df, &dscat)?;
        let mut dsterm_vals = string_column(df, &dsterm)?;
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
                let is_valid = ct.submission_values().iter().any(|val| val == &canonical);
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
    if let (Some(dsdecod), Some(dsterm)) = (col(domain, "DSDECOD"), col(domain, "DSTERM"))
        && has_column(df, &dsdecod)
        && has_column(df, &dsterm)
    {
        let mut decod_vals = string_column(df, &dsdecod)?;
        let term_vals = string_column(df, &dsterm)?;
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
    if let (Some(dsterm), Some(dsdecod)) = (col(domain, "DSTERM"), col(domain, "DSDECOD"))
        && has_column(df, &dsterm)
        && has_column(df, &dsdecod)
    {
        let mut term_vals = string_column(df, &dsterm)?;
        let decod_vals = string_column(df, &dsdecod)?;
        for (term, decod) in term_vals.iter_mut().zip(decod_vals.iter()) {
            let term_upper = term.to_uppercase();
            if term_upper.contains("SITE") && !decod.is_empty() {
                *term = decod.clone();
            }
        }
        set_string_column(df, &dsterm, term_vals)?;
    }
    if let (Some(dsterm), Some(dsdecod)) = (col(domain, "DSTERM"), col(domain, "DSDECOD"))
        && has_column(df, &dsterm)
        && has_column(df, &dsdecod)
    {
        let mut term_vals = string_column(df, &dsterm)?;
        let mut decod_vals = string_column(df, &dsdecod)?;
        for (term, decod) in term_vals.iter_mut().zip(decod_vals.iter_mut()) {
            if term.is_empty() && !decod.is_empty() {
                *term = decod.clone();
            } else if decod.is_empty() && !term.is_empty() {
                *decod = term.clone();
            }
        }
        set_string_column(df, &dsterm, term_vals)?;
        set_string_column(df, &dsdecod, decod_vals)?;
    }
    if let Some(dsdecod) = col(domain, "DSDECOD")
        && has_column(df, &dsdecod)
        && let Some(ct) = ctx.resolve_ct(domain, "DSDECOD")
    {
        let mut decod_vals = string_column(df, &dsdecod)?;
        for value in decod_vals.iter_mut() {
            let canonical = normalize_ct_value(ct, value);
            let mut mapped = match canonical.to_uppercase().as_str() {
                "Y" | "YES" => "COMPLETED".to_string(),
                "N" | "NO" => "SCREENING NOT COMPLETED".to_string(),
                _ => canonical.to_uppercase(),
            };
            if !ct.submission_values().iter().any(|val| val == &mapped) {
                let compact: String = mapped
                    .chars()
                    .filter(|ch| ch.is_ascii_alphanumeric())
                    .collect();
                if compact.contains("FAILURETOMEET") || compact.contains("SCREENFAILURE") {
                    mapped = "SCREEN FAILURE".to_string();
                } else if mapped.contains("WITHDRAW") && mapped.contains("CONSENT") {
                    mapped = "WITHDRAWAL OF CONSENT".to_string();
                } else if mapped.contains("WITHDRAW") && mapped.contains("SUBJECT") {
                    mapped = "WITHDRAWAL BY SUBJECT".to_string();
                } else if mapped.contains("LOST") && mapped.contains("FOLLOW") {
                    mapped = "LOST TO FOLLOW-UP".to_string();
                }
            }
            *value = mapped;
        }
        if let Some(dsterm) = col(domain, "DSTERM")
            && has_column(df, &dsterm)
        {
            let term_vals = string_column(df, &dsterm)?;
            for idx in 0..df.height() {
                let term_code = normalize_ct_value(ct, &term_vals[idx]).to_uppercase();
                let valid_from_term = ct.submission_values().iter().any(|val| *val == term_code);
                let valid_from_decod = ct
                    .submission_values()
                    .iter()
                    .any(|val| *val == decod_vals[idx]);
                if valid_from_term && !valid_from_decod {
                    decod_vals[idx] = term_code;
                }
            }
        }
        set_string_column(df, &dsdecod, decod_vals)?;
    }
    if let (Some(dsdecod), Some(dscat)) = (col(domain, "DSDECOD"), col(domain, "DSCAT"))
        && has_column(df, &dsdecod)
        && has_column(df, &dscat)
    {
        let codes = dsdecod_codelists(domain);
        let decod_vals = string_column(df, &dsdecod)?;
        let mut dscat_vals = string_column(df, &dscat)?;
        if let (Some(dscat_ct), Some(_)) = (ctx.resolve_ct(domain, "DSCAT"), ctx.ct_registry) {
            for idx in 0..df.height() {
                if !dscat_vals[idx].trim().is_empty() {
                    continue;
                }
                let decod = decod_vals[idx].trim();
                if decod.is_empty() {
                    continue;
                }
                if let Some(dsdecod_ct) = resolve_dsdecod_codelist(ctx, &codes, decod)
                    && let Some(value) = dscat_value_for_codelist(dscat_ct, dsdecod_ct)
                {
                    dscat_vals[idx] = value;
                }
            }
            set_string_column(df, &dscat, dscat_vals)?;
        }
    }
    if let Some(dsstdtc) = col(domain, "DSSTDTC")
        && has_column(df, &dsstdtc)
    {
        let values = string_column(df, &dsstdtc)?
            .into_iter()
            .map(|value| normalize_iso8601(&value))
            .collect();
        set_string_column(df, &dsstdtc, values)?;
        if let Some(dsstudy) = col(domain, "DSSTDY") {
            compute_study_day(domain, df, &dsstdtc, &dsstudy, ctx, "RFSTDTC")?;
        }
    }
    if let Some(dsdtc) = col(domain, "DSDTC")
        && has_column(df, &dsdtc)
    {
        let values = string_column(df, &dsdtc)?
            .into_iter()
            .map(|value| normalize_iso8601(&value))
            .collect();
        set_string_column(df, &dsdtc, values)?;
        if let Some(dsdy) = col(domain, "DSDY") {
            compute_study_day(domain, df, &dsdtc, &dsdy, ctx, "RFSTDTC")?;
        }
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
