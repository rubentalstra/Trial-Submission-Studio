use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};
use tracing::warn;

use sdtm_model::{CaseInsensitiveSet, Domain, VariableType};

use crate::ct_utils::normalize_ct_value;
use crate::data_utils::column_trimmed_values;
use crate::domain_processors;
use crate::pipeline_context::{PipelineContext, SequenceAssignmentMode, UsubjidPrefixMode};
use sdtm_ingest::any_to_string;

pub struct DomainProcessInput<'a> {
    pub domain: &'a Domain,
    pub data: &'a mut DataFrame,
    pub context: &'a PipelineContext,
    pub sequence_tracker: Option<&'a mut BTreeMap<String, i64>>,
}

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
/// When `ct_matching` is `Lenient`, compact-key matching is allowed. When
/// `ct_matching` is `Strict`, only exact matches and defined synonyms
/// are normalized.
fn normalize_ct_columns(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if context.ct_registry.catalogs.is_empty() {
        return Ok(());
    }
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
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
            let normalized = normalize_ct_value(ct, &raw, context.options.ct_matching);
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
    if matches!(context.options.usubjid_prefix, UsubjidPrefixMode::Skip) {
        return Ok(());
    }
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
    let Some(usubjid_col) = domain
        .column_name("USUBJID")
        .and_then(|name| column_lookup.get(name))
    else {
        return Ok(());
    };
    let study_col = domain
        .column_name("STUDYID")
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

    if changed {
        let new_series = Series::new(usubjid_col.into(), updated);
        df.with_column(new_series)?;
        if context.options.warn_on_rewrite {
            warn!(
                domain = %domain.code,
                "USUBJID values updated with study prefix"
            );
        }
    }
    Ok(())
}

pub fn process_domain(input: DomainProcessInput<'_>) -> Result<()> {
    let DomainProcessInput {
        domain,
        data,
        context,
        sequence_tracker,
    } = input;
    apply_base_rules(domain, data, context)?;
    domain_processors::process_domain(domain, data, context)?;
    normalize_ct_columns(domain, data, context)?;
    assign_sequence(domain, data, context, sequence_tracker)?;
    Ok(())
}

/// Assign --SEQ values based on USUBJID grouping.
///
/// Uses tracker if provided for cross-file sequence continuity.
/// Skips assignment if `context.options.sequence_assignment` is `Skip`.
fn assign_sequence(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
    sequence_tracker: Option<&mut BTreeMap<String, i64>>,
) -> Result<()> {
    if matches!(
        context.options.sequence_assignment,
        SequenceAssignmentMode::Skip
    ) {
        return Ok(());
    }
    let column_lookup = CaseInsensitiveSet::new(df.get_column_names_owned());
    let Some(seq_col) = domain.infer_seq_column() else {
        return Ok(());
    };
    let Some(usubjid_col) = domain.column_name("USUBJID") else {
        return Ok(());
    };
    let seq_col_name = column_lookup.get(seq_col).unwrap_or(seq_col);
    let usubjid_col_name = column_lookup.get(usubjid_col).unwrap_or(usubjid_col);
    if !needs_sequence_assignment(df, seq_col_name, usubjid_col_name)? {
        return Ok(());
    }
    if let Some(tracker) = sequence_tracker {
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
    let Some(group_values) = column_trimmed_values(df, group_column) else {
        return Ok(());
    };
    let had_existing = column_trimmed_values(df, seq_column)
        .map(|values| values.iter().any(|value| !value.is_empty()))
        .unwrap_or(false);
    let mut counters: BTreeMap<String, i64> = BTreeMap::new();
    let mut values: Vec<Option<f64>> = Vec::with_capacity(df.height());

    for key in &group_values {
        if key.is_empty() {
            values.push(None);
            continue;
        }
        let entry = counters.entry(key.clone()).or_insert(0);
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
    let Some(group_values) = column_trimmed_values(df, group_column) else {
        return Ok(());
    };
    let seq_values =
        column_trimmed_values(df, seq_column).unwrap_or_else(|| vec![String::new(); df.height()]);
    let had_existing = seq_values.iter().any(|value| !value.is_empty());
    let mut values: Vec<Option<f64>> = Vec::with_capacity(df.height());
    for (idx, key) in group_values.iter().enumerate() {
        if key.is_empty() {
            values.push(None);
            continue;
        }
        let entry = tracker.entry(key.clone()).or_insert(0);
        let parsed = parse_sequence_value(seq_values[idx].as_str());
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
    let Some(seq_values) = column_trimmed_values(df, seq_column) else {
        return Ok(true);
    };
    let Some(group_values) = column_trimmed_values(df, group_column) else {
        return Ok(true);
    };
    let mut groups: BTreeMap<String, BTreeSet<i64>> = BTreeMap::new();
    let mut has_value = false;
    for (group, value_str) in group_values.iter().zip(seq_values.iter()) {
        let group = group.trim().to_string();
        if group.is_empty() {
            continue;
        }
        if value_str.is_empty() {
            continue;
        }
        if let Some(parsed) = parse_sequence_value(value_str) {
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
