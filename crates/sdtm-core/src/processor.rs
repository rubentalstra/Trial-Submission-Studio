use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, anyhow};
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};

use sdtm_model::Domain;

use crate::domain_processors;
use crate::domain_utils::{infer_seq_column, standard_columns};
use crate::frame::DomainFrame;
use crate::processing_context::ProcessingContext;
fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::String(value) => value.to_string(),
        AnyValue::Null => String::new(),
        _ => value.to_string(),
    }
}

fn sanitize_identifier(raw: &str) -> String {
    let trimmed = raw.trim();
    if !trimmed.contains('"') {
        return trimmed.to_string();
    }
    trimmed.chars().filter(|ch| *ch != '"').collect()
}

pub fn apply_base_rules(domain: &Domain, df: &mut DataFrame, study_id: &str) -> Result<()> {
    let columns = standard_columns(domain);
    let usubjid_col = match columns.usubjid.as_ref() {
        Some(name) => name.clone(),
        None => return Ok(()),
    };
    let study_col = columns.study_id;
    let usubjid_series = match df.column(&usubjid_col) {
        Ok(series) => series.clone(),
        Err(_) => return Ok(()),
    };
    let study_series = study_col
        .as_deref()
        .and_then(|name| df.column(name).ok())
        .cloned();
    let mut updated = Vec::with_capacity(df.height());

    for idx in 0..df.height() {
        let raw_usubjid = any_to_string(usubjid_series.get(idx).unwrap_or(AnyValue::Null));
        let mut usubjid = sanitize_identifier(&raw_usubjid);
        let study_value = study_series
            .as_ref()
            .map(|series| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
            .unwrap_or_else(|| study_id.to_string());
        let study_value = sanitize_identifier(&study_value);
        if !study_value.is_empty() && !usubjid.is_empty() {
            let prefix = format!("{study_value}-");
            if !usubjid.starts_with(&prefix) {
                usubjid = format!("{prefix}{usubjid}");
            }
        }
        updated.push(usubjid);
    }

    let new_series = Series::new(usubjid_col.into(), updated);
    df.with_column(new_series)?;
    Ok(())
}

pub fn process_domain(domain: &Domain, df: &mut DataFrame, study_id: &str) -> Result<()> {
    let ctx = ProcessingContext::new(study_id);
    process_domain_with_context(domain, df, &ctx)
}

pub fn process_domain_with_context(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    apply_base_rules(domain, df, ctx.study_id)?;
    domain_processors::process_domain(domain, df, ctx)?;
    let columns = standard_columns(domain);
    if let (Some(seq_col), Some(usubjid_col)) = (infer_seq_column(domain), columns.usubjid) {
        if needs_sequence_assignment(df, &seq_col, &usubjid_col)? {
            assign_sequence(df, &seq_col, &usubjid_col)?;
        }
    }
    Ok(())
}

pub fn process_domains_with_context(
    domains: &[Domain],
    frames: &mut [DomainFrame],
    ctx: &ProcessingContext,
) -> Result<()> {
    let mut domain_map: BTreeMap<String, &Domain> = BTreeMap::new();
    for domain in domains {
        domain_map.insert(domain.code.to_uppercase(), domain);
    }
    frames.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));
    for frame in frames.iter_mut() {
        let key = frame.domain_code.to_uppercase();
        let domain = domain_map
            .get(&key)
            .ok_or_else(|| anyhow!("missing standards metadata for domain {}", key))?;
        process_domain_with_context(domain, &mut frame.data, ctx)?;
    }
    Ok(())
}

pub fn process_domains(
    domains: &[Domain],
    frames: &mut [DomainFrame],
    study_id: &str,
) -> Result<()> {
    let ctx = ProcessingContext::new(study_id);
    process_domains_with_context(domains, frames, &ctx)
}

fn assign_sequence(df: &mut DataFrame, seq_column: &str, group_column: &str) -> Result<()> {
    let group_series = match df.column(group_column) {
        Ok(series) => series.clone(),
        Err(_) => return Ok(()),
    };
    let mut counters: BTreeMap<String, i64> = BTreeMap::new();
    let mut values: Vec<Option<i64>> = Vec::with_capacity(df.height());

    for idx in 0..df.height() {
        let key = any_to_string(group_series.get(idx).unwrap_or(AnyValue::Null));
        let key = key.trim();
        if key.is_empty() {
            values.push(None);
            continue;
        }
        let entry = counters.entry(key.to_string()).or_insert(0);
        *entry += 1;
        values.push(Some(*entry));
    }

    let series = Series::new(seq_column.into(), values);
    df.with_column(series)?;
    Ok(())
}

fn needs_sequence_assignment(df: &DataFrame, seq_column: &str, group_column: &str) -> Result<bool> {
    let series = match df.column(seq_column) {
        Ok(series) => series,
        Err(_) => return Ok(true),
    };
    let group_series = match df.column(group_column) {
        Ok(series) => series,
        Err(_) => return Ok(true),
    };
    let mut groups: BTreeMap<String, BTreeSet<i64>> = BTreeMap::new();
    let mut has_value = false;
    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        let group = any_to_string(group_series.get(idx).unwrap_or(AnyValue::Null));
        let group = group.trim().to_string();
        if group.is_empty() {
            continue;
        }
        let value_str = any_to_string(value);
        let trimmed = value_str.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(parsed) = trimmed.parse::<i64>() {
            has_value = true;
            let entry = groups.entry(group).or_default();
            if entry.contains(&parsed) {
                return Ok(true);
            }
            entry.insert(parsed);
        } else {
            return Ok(true);
        }
    }
    Ok(!has_value)
}
