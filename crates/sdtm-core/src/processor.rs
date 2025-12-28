use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use tracing::warn;

use sdtm_model::{CaseInsensitiveSet, Domain, VariableType};

use crate::ct_utils::{normalize_ct_value_safe, normalize_ct_value_strict};
use crate::domain_processors;
use crate::domain_utils::{infer_seq_column, standard_columns};
use crate::pipeline_context::PipelineContext;
use sdtm_ingest::any_to_string;

fn sanitize_identifier(raw: &str) -> String {
    let trimmed = raw.trim();
    if !trimmed.contains('"') {
        return trimmed.to_string();
    }
    trimmed.chars().filter(|ch| *ch != '"').collect()
}

/// Normalize controlled terminology values in a DataFrame.
///
/// This function iterates through columns with CT constraints and normalizes
/// values to their preferred terms.
///
/// When `allow_lenient_ct_matching` is enabled in options, lenient matching
/// (including compact key matching) is used. When disabled, only exact matches
/// and defined synonyms are normalized.
fn normalize_ct_columns(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if context.ct_registry.catalogs.is_empty() {
        return Ok(());
    }
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
    let use_strict = !context.options.allow_lenient_ct_matching;

    for variable in &domain.variables {
        if !matches!(variable.data_type, VariableType::Char) {
            continue;
        }
        let Some(ct) = context.resolve_ct(domain, &variable.name) else {
            continue;
        };
        let Some(column_name) = column_lookup.get(&variable.name) else {
            continue;
        };
        let Ok(series) = df.column(column_name) else {
            continue;
        };
        let mut values = Vec::with_capacity(df.height());
        let mut changed = false;
        for idx in 0..df.height() {
            let raw = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
            if raw.trim().is_empty() {
                values.push(raw);
                continue;
            }
            // Use strict or lenient matching based on options
            let normalized = if use_strict {
                normalize_ct_value_strict(ct, &raw)
            } else {
                normalize_ct_value_safe(ct, &raw)
            };
            if normalized != raw {
                changed = true;
            }
            values.push(normalized);
        }
        if changed {
            let new_series = Series::new(column_name.into(), values);
            df.with_column(new_series)?;
        }
    }
    Ok(())
}

fn apply_base_rules(domain: &Domain, df: &mut DataFrame, context: &PipelineContext) -> Result<()> {
    if !context.options.prefix_usubjid {
        return Ok(());
    }
    let columns = standard_columns(domain);
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
    let Some(usubjid_col) = columns
        .usubjid
        .as_deref()
        .and_then(|name| column_lookup.get(name))
    else {
        return Ok(());
    };
    let study_col = columns
        .study_id
        .as_deref()
        .and_then(|name| column_lookup.get(name));
    let usubjid_series = match df.column(usubjid_col) {
        Ok(series) => series.clone(),
        Err(_) => return Ok(()),
    };
    let study_series = study_col.and_then(|name| df.column(name).ok()).cloned();
    let mut updated = Vec::with_capacity(df.height());
    let mut changed = false;

    for idx in 0..df.height() {
        let raw_usubjid = any_to_string(usubjid_series.get(idx).unwrap_or(AnyValue::Null));
        let mut usubjid = sanitize_identifier(&raw_usubjid);
        let study_value = study_series
            .as_ref()
            .map(|series| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
            .unwrap_or_else(|| context.study_id.to_string());
        let study_value = sanitize_identifier(&study_value);
        if !study_value.is_empty() && !usubjid.is_empty() {
            let prefix = format!("{study_value}-");
            if !usubjid.starts_with(&prefix) {
                usubjid = format!("{prefix}{usubjid}");
            }
        }
        if usubjid != raw_usubjid {
            changed = true;
        }
        updated.push(usubjid);
    }

    let new_series = Series::new(usubjid_col.into(), updated);
    df.with_column(new_series)?;
    if changed && context.options.warn_on_rewrite {
        warn!(
            domain = %domain.code,
            "USUBJID values updated with study prefix"
        );
    }
    Ok(())
}

pub fn process_domain_with_context_and_tracker(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
    seq_tracker: Option<&mut BTreeMap<String, i64>>,
) -> Result<()> {
    apply_base_rules(domain, df, context)?;
    domain_processors::process_domain(domain, df, context)?;
    normalize_ct_columns(domain, df, context)?;
    assign_sequence(domain, df, context, seq_tracker)?;
    Ok(())
}

/// Assign --SEQ values based on USUBJID grouping.
///
/// Uses tracker if provided for cross-file sequence continuity.
/// Skips assignment if `context.options.assign_sequence` is false.
fn assign_sequence(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
    seq_tracker: Option<&mut BTreeMap<String, i64>>,
) -> Result<()> {
    if !context.options.assign_sequence {
        return Ok(());
    }
    let columns = standard_columns(domain);
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
    let (Some(seq_col), Some(usubjid_col)) = (infer_seq_column(domain), columns.usubjid) else {
        return Ok(());
    };
    let seq_col_name = column_lookup.get(&seq_col).unwrap_or(seq_col.as_str());
    let usubjid_col_name = column_lookup
        .get(&usubjid_col)
        .unwrap_or(usubjid_col.as_str());
    if !needs_sequence_assignment(df, seq_col_name, usubjid_col_name)? {
        return Ok(());
    }
    if let Some(tracker) = seq_tracker {
        assign_sequence_with_tracker(domain, df, seq_col_name, usubjid_col_name, tracker, context)?;
    } else {
        assign_sequence_values(domain, df, seq_col_name, usubjid_col_name, context)?;
    }
    Ok(())
}

fn assign_sequence_values(
    domain: &Domain,
    df: &mut DataFrame,
    seq_column: &str,
    group_column: &str,
    context: &PipelineContext,
) -> Result<()> {
    let group_series = match df.column(group_column) {
        Ok(series) => series.clone(),
        Err(_) => return Ok(()),
    };
    let had_existing = has_existing_sequence(df, seq_column);
    let mut counters: BTreeMap<String, i64> = BTreeMap::new();
    let mut values: Vec<Option<f64>> = Vec::with_capacity(df.height());

    for idx in 0..df.height() {
        let key = any_to_string(group_series.get(idx).unwrap_or(AnyValue::Null));
        let key = key.trim();
        if key.is_empty() {
            values.push(None);
            continue;
        }
        let entry = counters.entry(key.to_string()).or_insert(0);
        *entry += 1;
        values.push(Some(*entry as f64));
    }

    let series = Series::new(seq_column.into(), values);
    df.with_column(series)?;
    if had_existing && context.options.warn_on_rewrite {
        warn!(
            domain = %domain.code,
            sequence = %seq_column,
            "Sequence values recalculated"
        );
    }

    Ok(())
}

fn assign_sequence_with_tracker(
    domain: &Domain,
    df: &mut DataFrame,
    seq_column: &str,
    group_column: &str,
    tracker: &mut BTreeMap<String, i64>,
    context: &PipelineContext,
) -> Result<()> {
    if df.height() == 0 {
        return Ok(());
    }
    let group_series = match df.column(group_column) {
        Ok(series) => series.clone(),
        Err(_) => return Ok(()),
    };
    let seq_series = df.column(seq_column).ok().cloned();
    let had_existing = seq_series.as_ref().map(column_has_values).unwrap_or(false);
    let mut values: Vec<Option<f64>> = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let key = any_to_string(group_series.get(idx).unwrap_or(AnyValue::Null));
        let key = key.trim();
        if key.is_empty() {
            values.push(None);
            continue;
        }
        let entry = tracker.entry(key.to_string()).or_insert(0);
        let existing = seq_series
            .as_ref()
            .map(|series| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
            .unwrap_or_default();
        let parsed = parse_sequence_value(existing.trim());
        let value = match parsed {
            Some(seq) if seq > *entry => {
                *entry = seq;
                seq
            }
            _ => {
                *entry += 1;
                *entry
            }
        };
        values.push(Some(value as f64));
    }
    let series = Series::new(seq_column.into(), values);
    df.with_column(series)?;
    if had_existing && context.options.warn_on_rewrite {
        warn!(
            domain = %domain.code,
            sequence = %seq_column,
            "Sequence values recalculated with tracker"
        );
    }

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
        if let Some(parsed) = parse_sequence_value(trimmed) {
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

fn parse_sequence_value(text: &str) -> Option<i64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(value) = trimmed.parse::<i64>() {
        return Some(value);
    }
    if let Ok(value) = trimmed.parse::<f64>()
        && value.is_finite()
    {
        let rounded = value.round();
        if (value - rounded).abs() <= f64::EPSILON && rounded >= 0.0 && rounded <= i64::MAX as f64 {
            return Some(rounded as i64);
        }
    }
    None
}

fn has_existing_sequence(df: &DataFrame, seq_column: &str) -> bool {
    let Ok(series) = df.column(seq_column) else {
        return false;
    };
    column_has_values(series)
}

fn column_has_values(series: &Column) -> bool {
    for idx in 0..series.len() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        if !value.trim().is_empty() {
            return true;
        }
    }
    false
}
