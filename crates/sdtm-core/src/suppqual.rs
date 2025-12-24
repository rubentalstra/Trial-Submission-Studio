use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use sdtm_model::Domain;

use crate::domain_utils::{infer_seq_column, standard_columns};
pub struct SuppqualResult {
    pub domain_code: String,
    pub data: DataFrame,
    pub used_columns: Vec<String>,
}

pub fn suppqual_domain_code(parent_domain: &str) -> String {
    format!("SUPP{}", parent_domain.to_uppercase())
}

fn ordered_variable_names(domain: &Domain) -> Vec<String> {
    domain
        .variables
        .iter()
        .map(|variable| variable.name.clone())
        .collect()
}

fn variable_name_set(domain: &Domain) -> BTreeSet<String> {
    domain
        .variables
        .iter()
        .map(|variable| variable.name.to_uppercase())
        .collect()
}

fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::String(value) => value.to_string(),
        AnyValue::Null => String::new(),
        _ => value.to_string(),
    }
}

fn strip_wrapping_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
}

fn sanitize_qnam(name: &str) -> String {
    let mut safe = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            safe.push(ch.to_ascii_uppercase());
        } else {
            safe.push('_');
        }
    }
    while safe.contains("__") {
        safe = safe.replace("__", "_");
    }
    safe = safe.trim_matches('_').to_string();
    if safe.is_empty() {
        safe = "QVAL".to_string();
    }
    if safe
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        safe = format!("Q{safe}");
    }
    safe.chars().take(8).collect()
}

fn unique_qnam(name: &str, used: &mut BTreeMap<String, String>) -> String {
    let base = sanitize_qnam(name);
    if let Some(existing) = used.get(&base) {
        if existing.eq_ignore_ascii_case(name) {
            return base;
        }
    } else {
        used.insert(base.clone(), name.to_string());
        return base;
    }
    for idx in 1..=99 {
        let suffix = format!("{idx:02}");
        let prefix_len = 8usize.saturating_sub(suffix.len()).max(1);
        let mut prefix: String = base.chars().take(prefix_len).collect();
        if prefix.is_empty() {
            prefix = "Q".to_string();
        }
        let candidate = format!("{prefix}{suffix}");
        if !used.contains_key(&candidate) {
            used.insert(candidate.clone(), name.to_string());
            return candidate;
        }
    }
    base
}

fn column_value(df: &DataFrame, name: &str, idx: usize) -> String {
    match df.column(name) {
        Ok(series) => any_to_string(series.get(idx).unwrap_or(AnyValue::Null)),
        Err(_) => String::new(),
    }
}

fn populated_columns(df: &DataFrame) -> BTreeSet<String> {
    let mut populated = BTreeSet::new();
    for series in df.get_columns() {
        let mut has_value = false;
        for idx in 0..df.height() {
            let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
            if !value.trim().is_empty() {
                has_value = true;
                break;
            }
        }
        if has_value {
            populated.insert(series.name().to_uppercase());
        }
    }
    populated
}

fn is_duplicate_of_mapped(name: &str, populated: &BTreeSet<String>) -> bool {
    if populated.is_empty() {
        return false;
    }
    let upper = name.to_uppercase();
    if upper.ends_with("SEQ") && populated.iter().any(|col| col.ends_with("SEQ")) {
        return true;
    }
    if upper.ends_with("CD") && upper.len() > 2 {
        let base = &upper[..upper.len() - 2];
        if populated.contains(base) {
            return true;
        }
    }
    if let Some(prefix) = upper.strip_suffix("DATE") {
        if populated.contains(&format!("{prefix}DTC")) {
            return true;
        }
    }
    if let Some(prefix) = upper.strip_suffix("DAT") {
        if populated.contains(&format!("{prefix}DTC")) {
            return true;
        }
    }
    if let Some(prefix) = upper.strip_suffix("DT") {
        if populated.contains(&format!("{prefix}DTC")) {
            return true;
        }
    }
    false
}

pub fn build_suppqual(
    parent_domain: &Domain,
    suppqual_domain: &Domain,
    source_df: &DataFrame,
    mapped_df: Option<&DataFrame>,
    used_source_columns: &BTreeSet<String>,
    study_id: &str,
    exclusion_columns: Option<&BTreeSet<String>>,
) -> Result<Option<SuppqualResult>> {
    let parent_domain_code = parent_domain.code.to_uppercase();
    let ordered_columns = ordered_variable_names(suppqual_domain);
    let core_variables = variable_name_set(parent_domain);
    let suppqual_cols = standard_columns(suppqual_domain);
    let parent_cols = standard_columns(parent_domain);
    let populated = mapped_df.map(populated_columns).unwrap_or_default();
    if ordered_columns.is_empty() {
        return Ok(None);
    }
    let mut extra_cols: Vec<String> = Vec::new();
    for series in source_df.get_columns() {
        let name = series.name().to_string();
        if used_source_columns.contains(&name) {
            continue;
        }
        if core_variables.contains(&name.to_uppercase()) {
            continue;
        }
        if is_duplicate_of_mapped(&name, &populated) {
            continue;
        }
        if let Some(exclusions) = exclusion_columns {
            if exclusions.contains(&name.to_uppercase()) {
                continue;
            }
        }
        extra_cols.push(name);
    }

    let mut extra_upper: BTreeMap<String, String> = BTreeMap::new();
    let mut non_empty_upper = BTreeSet::new();
    for name in &extra_cols {
        extra_upper.insert(name.to_uppercase(), name.clone());
        let upper = name.to_uppercase();
        if let Ok(series) = source_df.column(name) {
            for idx in 0..source_df.height() {
                let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
                if !value.trim().is_empty() {
                    non_empty_upper.insert(upper.clone());
                    break;
                }
            }
        }
    }
    extra_cols = extra_cols
        .into_iter()
        .filter(|name| {
            let upper = name.to_uppercase();
            if upper.ends_with("CD") && upper.len() > 2 {
                let base = &upper[..upper.len() - 2];
                if extra_upper.contains_key(base) && non_empty_upper.contains(base) {
                    return false;
                }
            }
            true
        })
        .collect();

    if extra_cols.is_empty() {
        return Ok(None);
    }

    let mut row_count = source_df.height();
    if let Some(mapped) = mapped_df {
        row_count = row_count.min(mapped.height());
    }

    let idvar = infer_seq_column(parent_domain).and_then(|seq_var| {
        if mapped_df.and_then(|df| df.column(&seq_var).ok()).is_some() {
            Some(seq_var)
        } else {
            None
        }
    });

    let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for key in &ordered_columns {
        values.insert(key.to_string(), Vec::new());
    }

    let mut push_value = |key: Option<&str>, value: String| {
        if let Some(name) = key {
            if let Some(entry) = values.get_mut(name) {
                entry.push(value);
            }
        }
    };

    let mut qnam_used: BTreeMap<String, String> = BTreeMap::new();
    let mut qnam_map: BTreeMap<String, String> = BTreeMap::new();
    for col in &extra_cols {
        let qnam = unique_qnam(col, &mut qnam_used);
        qnam_map.insert(col.clone(), qnam);
    }

    let mut seen_keys: BTreeSet<String> = BTreeSet::new();
    for col in &extra_cols {
        let qnam = qnam_map
            .get(col)
            .cloned()
            .unwrap_or_else(|| sanitize_qnam(col));
        for idx in 0..row_count {
            let raw_val = strip_wrapping_quotes(&column_value(source_df, col, idx));
            if raw_val.is_empty() {
                continue;
            }
            let study_value = parent_cols
                .study_id
                .as_deref()
                .map(|name| strip_wrapping_quotes(&column_value(source_df, name, idx)))
                .unwrap_or_default();
            let usubjid_value = parent_cols
                .usubjid
                .as_deref()
                .map(|name| strip_wrapping_quotes(&column_value(source_df, name, idx)))
                .unwrap_or_default();
            let mapped_usubjid = mapped_df
                .and_then(|df| {
                    parent_cols
                        .usubjid
                        .as_deref()
                        .map(|name| strip_wrapping_quotes(&column_value(df, name, idx)))
                })
                .unwrap_or_default();
            let final_usubjid = if !usubjid_value.is_empty() {
                usubjid_value
            } else {
                mapped_usubjid
            };

            let idvar_value = idvar.clone().unwrap_or_default();
            let idvarval = if let (Some(mapped), Some(idvar_name)) = (mapped_df, &idvar) {
                column_value(mapped, idvar_name, idx)
            } else {
                String::new()
            };

            let dedupe_key = format!(
                "{}|{}|{}|{}|{}|{}",
                study_value.trim(),
                parent_domain_code,
                final_usubjid.trim(),
                idvar_value.trim(),
                idvarval.trim(),
                qnam
            );
            if !seen_keys.insert(dedupe_key) {
                continue;
            }

            push_value(
                suppqual_cols.study_id.as_deref(),
                if !study_value.is_empty() {
                    study_value
                } else {
                    study_id.to_string()
                },
            );
            push_value(suppqual_cols.rdomain.as_deref(), parent_domain_code.clone());
            push_value(suppqual_cols.usubjid.as_deref(), final_usubjid);
            push_value(suppqual_cols.idvar.as_deref(), idvar_value);
            push_value(suppqual_cols.idvarval.as_deref(), idvarval);
            push_value(suppqual_cols.qnam.as_deref(), qnam.clone());
            push_value(suppqual_cols.qlabel.as_deref(), qnam.clone());
            push_value(suppqual_cols.qval.as_deref(), raw_val);
            push_value(suppqual_cols.qorig.as_deref(), "CRF".to_string());
            push_value(suppqual_cols.qeval.as_deref(), String::new());
        }
    }

    let total_rows = suppqual_cols
        .qval
        .as_deref()
        .and_then(|name| values.get(name))
        .map(|vals| vals.len())
        .unwrap_or(0);
    if total_rows == 0 {
        return Ok(None);
    }

    let columns: Vec<Column> = ordered_columns
        .iter()
        .map(|name| {
            let mut vals = values.remove(name).unwrap_or_default();
            if vals.len() < total_rows {
                vals.resize(total_rows, String::new());
            }
            Series::new(name.as_str().into(), vals).into()
        })
        .collect();
    let data = DataFrame::new(columns)?;

    Ok(Some(SuppqualResult {
        domain_code: suppqual_domain_code(&parent_domain_code),
        data,
        used_columns: extra_cols,
    }))
}
