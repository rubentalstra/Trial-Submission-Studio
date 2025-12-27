use std::collections::{HashMap, HashSet};

use anyhow::Result;
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use polars::prelude::{
    AnyValue, BooleanChunked, DataFrame, NamedFrom, NewChunkedArray, Series, UInt32Chunked,
};

use sdtm_model::{ControlledTerminology, Domain};

use crate::domain_utils::column_name;
use crate::processing_context::ProcessingContext;

pub(super) fn col(domain: &Domain, name: &str) -> Option<String> {
    column_name(domain, name)
}

pub(super) fn has_column(df: &DataFrame, name: &str) -> bool {
    df.column(name).is_ok()
}

fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        AnyValue::Null => String::new(),
        _ => value.to_string(),
    }
}

pub(super) fn string_value(df: &DataFrame, name: &str, idx: usize) -> String {
    match df.column(name) {
        Ok(series) => any_to_string(series.get(idx).unwrap_or(AnyValue::Null)),
        Err(_) => String::new(),
    }
}

pub(super) enum Trim {
    Both,
}

pub(super) fn string_column(df: &DataFrame, name: &str, trim: Trim) -> Result<Vec<String>> {
    let series = df.column(name)?;
    let mut values = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let mut value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        if matches!(trim, Trim::Both) {
            value = value.trim().to_string();
        }
        values.push(value);
    }
    Ok(values)
}

fn strip_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if !trimmed.contains('"') {
        return trimmed.to_string();
    }
    trimmed.chars().filter(|ch| *ch != '"').collect()
}

pub(super) fn numeric_column_f64(df: &DataFrame, name: &str) -> Result<Vec<Option<f64>>> {
    let series = df.column(name)?;
    let mut values = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        values.push(any_to_f64(value));
    }
    Ok(values)
}

pub(super) fn numeric_column_i64(df: &DataFrame, name: &str) -> Result<Vec<Option<i64>>> {
    let series = df.column(name)?;
    let mut values = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        values.push(any_to_i64(value));
    }
    Ok(values)
}

pub(super) fn set_string_column(df: &mut DataFrame, name: &str, values: Vec<String>) -> Result<()> {
    let series = Series::new(name.into(), values);
    df.with_column(series)?;
    Ok(())
}

pub(super) fn set_f64_column(
    df: &mut DataFrame,
    name: &str,
    values: Vec<Option<f64>>,
) -> Result<()> {
    let series = Series::new(name.into(), values);
    df.with_column(series)?;
    Ok(())
}

pub(super) fn set_i64_column(
    df: &mut DataFrame,
    name: &str,
    values: Vec<Option<i64>>,
) -> Result<()> {
    let series = Series::new(name.into(), values);
    df.with_column(series)?;
    Ok(())
}

pub(super) fn filter_rows(df: &mut DataFrame, keep: &[bool]) -> Result<()> {
    let mask = BooleanChunked::from_slice("keep".into(), keep);
    *df = df.filter(&mask)?;
    Ok(())
}

pub(super) fn deduplicate<S: AsRef<str>>(df: &mut DataFrame, keys: &[S]) -> Result<()> {
    if keys.is_empty() || df.height() == 0 {
        return Ok(());
    }
    let mut key_columns = Vec::with_capacity(keys.len());
    for key in keys {
        let key = key.as_ref();
        if !has_column(df, key) {
            continue;
        }
        key_columns.push(string_column(df, key, Trim::Both)?);
    }
    if key_columns.is_empty() {
        return Ok(());
    }
    let mut seen: HashSet<String> = HashSet::new();
    let mut keep = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let mut key = String::new();
        for col_vals in &key_columns {
            key.push_str(&col_vals[idx]);
            key.push('|');
        }
        if seen.insert(key) {
            keep.push(true);
        } else {
            keep.push(false);
        }
    }
    filter_rows(df, &keep)?;
    Ok(())
}

pub(super) fn apply_map_upper(
    df: &mut DataFrame,
    column: Option<&str>,
    mapping: &HashMap<String, String>,
) -> Result<()> {
    let Some(column) = column else {
        return Ok(());
    };
    if !has_column(df, column) {
        return Ok(());
    }
    let values = string_column(df, column, Trim::Both)?
        .into_iter()
        .map(|value| {
            let upper = value.to_uppercase();
            mapping.get(&upper).cloned().unwrap_or(upper)
        })
        .collect();
    set_string_column(df, column, values)?;
    Ok(())
}

pub(super) fn map_values<const N: usize>(pairs: [(&str, &str); N]) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(N);
    for (key, value) in pairs {
        map.insert(key.to_string(), value.to_string());
    }
    map
}

pub(super) fn normalize_empty_tokens(value: &str) -> String {
    match value.trim() {
        "<NA>" | "nan" | "None" => String::new(),
        _ => value.trim().to_string(),
    }
}

pub(super) fn replace_unknown(value: &str, default: &str) -> String {
    let upper = value.trim().to_uppercase();
    match upper.as_str() {
        "" | "UNK" | "UNKNOWN" | "NA" | "N/A" | "NONE" | "NAN" | "<NA>" => default.to_string(),
        _ => value.trim().to_string(),
    }
}

pub(super) fn normalize_ct_value(ct: &ControlledTerminology, raw: &str) -> String {
    let text = raw.trim();
    if text.is_empty() {
        return String::new();
    }
    let key = text.to_uppercase();
    ct.synonyms
        .get(&key)
        .cloned()
        .unwrap_or_else(|| text.to_string())
}

pub(super) fn normalize_ct_value_keep(ct: &ControlledTerminology, raw: &str) -> String {
    let text = raw.trim();
    if text.is_empty() {
        return String::new();
    }
    let canonical = normalize_ct_value(ct, text);
    if ct.submission_values.iter().any(|val| val == &canonical) {
        canonical
    } else {
        text.to_string()
    }
}

fn compact_key(value: &str, expand_bp: bool) -> String {
    let mut out = String::new();
    let mut token = String::new();
    let flush = |token: &mut String, out: &mut String| {
        if token.is_empty() {
            return;
        }
        let upper = token.to_uppercase();
        if expand_bp && upper == "BP" {
            out.push_str("BLOODPRESSURE");
        } else {
            out.push_str(&upper);
        }
        token.clear();
    };
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            token.push(ch);
        } else {
            flush(&mut token, &mut out);
        }
    }
    flush(&mut token, &mut out);
    out
}

fn matches_compact(a: &str, b: &str) -> bool {
    let a_compact = compact_key(a, false);
    let b_compact = compact_key(b, false);
    if a_compact == b_compact {
        return true;
    }
    let a_bp = compact_key(a, true);
    let b_bp = compact_key(b, true);
    a_bp == b_bp
}

pub(super) fn resolve_ct_submission_value(ct: &ControlledTerminology, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let canonical = normalize_ct_value(ct, trimmed);
    if ct.submission_values.iter().any(|val| val == &canonical) {
        return Some(canonical);
    }
    for (synonym, submission) in &ct.synonyms {
        if matches_compact(trimmed, synonym) {
            return Some(submission.clone());
        }
    }
    for submission in &ct.submission_values {
        if matches_compact(trimmed, submission) {
            return Some(submission.clone());
        }
    }
    for (submission, preferred) in &ct.preferred_terms {
        if matches_compact(trimmed, preferred) {
            return Some(submission.clone());
        }
    }
    None
}

pub(super) fn preferred_term_for(ct: &ControlledTerminology, submission: &str) -> Option<String> {
    ct.preferred_terms.get(submission).cloned()
}

fn any_to_f64(value: AnyValue) -> Option<f64> {
    match value {
        AnyValue::Null => None,
        AnyValue::Float32(value) => Some(value as f64),
        AnyValue::Float64(value) => Some(value),
        AnyValue::Int8(value) => Some(value as f64),
        AnyValue::Int16(value) => Some(value as f64),
        AnyValue::Int32(value) => Some(value as f64),
        AnyValue::Int64(value) => Some(value as f64),
        AnyValue::UInt8(value) => Some(value as f64),
        AnyValue::UInt16(value) => Some(value as f64),
        AnyValue::UInt32(value) => Some(value as f64),
        AnyValue::UInt64(value) => Some(value as f64),
        AnyValue::String(value) => parse_f64(value),
        AnyValue::StringOwned(value) => parse_f64(&value),
        _ => None,
    }
}

fn any_to_i64(value: AnyValue) -> Option<i64> {
    match value {
        AnyValue::Null => None,
        AnyValue::Int8(value) => Some(value as i64),
        AnyValue::Int16(value) => Some(value as i64),
        AnyValue::Int32(value) => Some(value as i64),
        AnyValue::Int64(value) => Some(value),
        AnyValue::UInt8(value) => Some(value as i64),
        AnyValue::UInt16(value) => Some(value as i64),
        AnyValue::UInt32(value) => Some(value as i64),
        AnyValue::UInt64(value) => Some(value as i64),
        AnyValue::Float32(value) => Some(value as i64),
        AnyValue::Float64(value) => Some(value as i64),
        AnyValue::String(value) => parse_i64(value),
        AnyValue::StringOwned(value) => parse_i64(&value),
        _ => None,
    }
}

pub(super) fn parse_f64(value: &str) -> Option<f64> {
    if value.trim().is_empty() {
        return None;
    }
    value.trim().parse::<f64>().ok()
}

pub(super) fn parse_i64(value: &str) -> Option<i64> {
    if value.trim().is_empty() {
        return None;
    }
    value.trim().parse::<i64>().ok()
}

pub(super) fn is_numeric_string(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    trimmed.parse::<f64>().is_ok()
}

pub(super) fn drop_placeholder_rows(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    let Some(usubjid_col) = col(domain, "USUBJID") else {
        return Ok(());
    };
    if !has_column(df, &usubjid_col) {
        return Ok(());
    }
    let mut usubjid_vals = string_column(df, &usubjid_col, Trim::Both)?;
    for value in &mut usubjid_vals {
        *value = strip_quotes(value);
    }
    let mut missing = vec![false; df.height()];
    for idx in 0..df.height() {
        missing[idx] = is_missing_usubjid(&usubjid_vals[idx]);
    }

    if missing.iter().any(|value| *value) {
        if let Some(subjid_col) = col(domain, "SUBJID")
            && has_column(df, &subjid_col)
        {
            let subjid_vals = string_column(df, &subjid_col, Trim::Both)?;
            let studyid_vals = col(domain, "STUDYID")
                .filter(|name| has_column(df, name))
                .and_then(|name| string_column(df, &name, Trim::Both).ok())
                .unwrap_or_else(|| vec![String::new(); df.height()]);
            for idx in 0..df.height() {
                if !missing[idx] {
                    continue;
                }
                let subjid = strip_quotes(&subjid_vals[idx]);
                let subjid = subjid.trim();
                let placeholder = matches!(
                    subjid.to_uppercase().as_str(),
                    "SUBJID" | "SUBJECTID" | "SUBJECT ID"
                );
                if subjid.is_empty() || placeholder {
                    continue;
                }
                let studyid = strip_quotes(studyid_vals[idx].trim());
                if studyid.is_empty() {
                    usubjid_vals[idx] = subjid.to_string();
                } else {
                    usubjid_vals[idx] = format!("{}-{}", studyid, subjid);
                }
            }
        }
        for idx in 0..df.height() {
            missing[idx] = is_missing_usubjid(&usubjid_vals[idx]);
        }
    }

    if missing.iter().any(|value| *value) {
        let keep = missing.iter().map(|value| !*value).collect::<Vec<_>>();
        set_string_column(df, &usubjid_col, usubjid_vals)?;
        filter_rows(df, &keep)?;
    } else {
        set_string_column(df, &usubjid_col, usubjid_vals.clone())?;
    }

    let mut study_id = String::new();
    if let Some(studyid_col) = col(domain, "STUDYID")
        && has_column(df, &studyid_col)
    {
        let study_vals = string_column(df, &studyid_col, Trim::Both)?;
        if let Some(found) = study_vals.iter().find(|value| !value.is_empty()) {
            study_id = strip_quotes(found);
        }
    }
    if study_id.is_empty() {
        study_id = ctx.study_id.to_string();
    }
    if !study_id.is_empty() {
        let prefix = format!("{study_id}-");
        let mut updated = string_column(df, &usubjid_col, Trim::Both)?;
        for value in &mut updated {
            if !value.is_empty() && !value.starts_with(&prefix) {
                *value = format!("{prefix}{value}");
            }
        }
        set_string_column(df, &usubjid_col, updated)?;
    }
    Ok(())
}

fn is_missing_usubjid(value: &str) -> bool {
    matches!(
        value.trim().to_uppercase().as_str(),
        "" | "NAN" | "<NA>" | "NONE" | "NULL"
    )
}

pub(super) fn normalize_iso8601_value(raw_value: &str) -> String {
    raw_value.trim().to_string()
}

fn parse_date(raw: &str) -> Option<NaiveDate> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Some(date);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt.date());
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M") {
        return Some(dt.date());
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.date_naive());
    }
    None
}

pub(super) fn ensure_date_pair_order(
    df: &mut DataFrame,
    start_col: &str,
    end_col: Option<&str>,
) -> Result<()> {
    if !has_column(df, start_col) {
        return Ok(());
    }
    let start_vals = string_column(df, start_col, Trim::Both)?
        .into_iter()
        .map(|value| normalize_iso8601_value(&value))
        .collect::<Vec<_>>();
    set_string_column(df, start_col, start_vals.clone())?;
    if let Some(end_col) = end_col
        && has_column(df, end_col)
    {
        let mut end_vals = string_column(df, end_col, Trim::Both)?
            .into_iter()
            .map(|value| normalize_iso8601_value(&value))
            .collect::<Vec<_>>();
        for idx in 0..df.height() {
            if end_vals[idx].is_empty() {
                continue;
            }
            let start_date = parse_date(&start_vals[idx]);
            let end_date = parse_date(&end_vals[idx]);
            if let (Some(start), Some(end)) = (start_date, end_date)
                && end < start
            {
                end_vals[idx] = start_vals[idx].clone();
            }
        }
        set_string_column(df, end_col, end_vals)?;
    }
    Ok(())
}

pub(super) fn compute_study_day(
    domain: &Domain,
    df: &mut DataFrame,
    dtc_col: &str,
    dy_col: &str,
    ctx: &ProcessingContext,
    reference_col: &str,
) -> Result<()> {
    if !has_column(df, dtc_col) {
        return Ok(());
    }
    let dtc_vals = string_column(df, dtc_col, Trim::Both)?;
    let mut baseline_vals: Vec<Option<NaiveDate>> = vec![None; df.height()];
    if let (Some(reference_starts), Some(usubjid_col)) =
        (ctx.reference_starts, col(domain, "USUBJID"))
        && has_column(df, &usubjid_col)
    {
        let usub_vals = string_column(df, &usubjid_col, Trim::Both)?;
        for idx in 0..df.height() {
            if let Some(start) = reference_starts.get(&usub_vals[idx]) {
                baseline_vals[idx] = parse_date(start);
            }
        }
    }
    if baseline_vals.iter().all(|value| value.is_none()) && has_column(df, reference_col) {
        let ref_vals = string_column(df, reference_col, Trim::Both)?;
        for idx in 0..df.height() {
            baseline_vals[idx] = parse_date(&ref_vals[idx]);
        }
    }
    if baseline_vals.iter().all(|value| value.is_none()) {
        return Ok(());
    }
    let mut dy_vals: Vec<Option<f64>> = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let obs_date = parse_date(&dtc_vals[idx]);
        let baseline = baseline_vals[idx];
        if let (Some(obs), Some(base)) = (obs_date, baseline) {
            let delta = obs.signed_duration_since(base).num_days();
            let adjusted = if delta >= 0 { delta + 1 } else { delta };
            dy_vals.push(Some(adjusted as f64));
        } else {
            dy_vals.push(None);
        }
    }
    set_f64_column(df, dy_col, dy_vals)?;
    Ok(())
}

pub(super) fn sort_by_numeric(df: &mut DataFrame, column: &str) -> Result<()> {
    let values = numeric_column_f64(df, column)?;
    let mut indices: Vec<u32> = (0..df.height()).map(|idx| idx as u32).collect();
    indices.sort_by(|a, b| {
        let left = values[*a as usize];
        let right = values[*b as usize];
        left.partial_cmp(&right)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let idx = UInt32Chunked::from_vec("idx".into(), indices);
    *df = df.take(&idx)?;
    Ok(())
}

pub(super) fn is_valid_time(value: &str) -> bool {
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
