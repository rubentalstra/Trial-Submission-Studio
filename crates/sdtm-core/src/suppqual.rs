use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use sdtm_model::Domain;

use crate::domain_utils::{infer_seq_column, standard_columns};
use sdtm_ingest::any_to_string;
pub struct SuppqualResult {
    pub domain_code: String,
    pub data: DataFrame,
    pub used_columns: Vec<String>,
}

pub struct SuppqualInput<'a> {
    pub parent_domain: &'a Domain,
    pub suppqual_domain: &'a Domain,
    pub source_df: &'a DataFrame,
    pub mapped_df: Option<&'a DataFrame>,
    pub used_source_columns: &'a BTreeSet<String>,
    pub study_id: &'a str,
    pub exclusion_columns: Option<&'a BTreeSet<String>>,
    pub source_labels: Option<&'a BTreeMap<String, String>>,
    pub derived_columns: Option<&'a BTreeSet<String>>,
}

pub fn suppqual_domain_code(parent_domain: &str) -> String {
    let parent = parent_domain.to_uppercase();
    let candidate = format!("SUPP{parent}");
    if candidate.len() <= 8 {
        return candidate;
    }
    let short = format!("SQ{parent}");
    if short.len() <= 8 {
        return short;
    }
    short.chars().take(8).collect()
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
    if let Some(prefix) = upper.strip_suffix("DATE")
        && populated.contains(&format!("{prefix}DTC"))
    {
        return true;
    }
    if let Some(prefix) = upper.strip_suffix("DAT")
        && populated.contains(&format!("{prefix}DTC"))
    {
        return true;
    }
    if let Some(prefix) = upper.strip_suffix("DT")
        && populated.contains(&format!("{prefix}DTC"))
    {
        return true;
    }
    false
}

pub fn build_suppqual(input: SuppqualInput<'_>) -> Result<Option<SuppqualResult>> {
    let parent_domain_code = input.parent_domain.code.to_uppercase();
    let ordered_columns = ordered_variable_names(input.suppqual_domain);
    let core_variables = variable_name_set(input.parent_domain);
    let suppqual_cols = standard_columns(input.suppqual_domain);
    let parent_cols = standard_columns(input.parent_domain);
    let populated = input.mapped_df.map(populated_columns).unwrap_or_default();
    if ordered_columns.is_empty() {
        return Ok(None);
    }
    let mut extra_cols: Vec<String> = Vec::new();
    for series in input.source_df.get_columns() {
        let name = series.name().to_string();
        if input.used_source_columns.contains(&name) {
            continue;
        }
        if core_variables.contains(&name.to_uppercase()) {
            continue;
        }
        if is_duplicate_of_mapped(&name, &populated) {
            continue;
        }
        if let Some(exclusions) = input.exclusion_columns
            && exclusions.contains(&name.to_uppercase())
        {
            continue;
        }
        extra_cols.push(name);
    }

    let mut extra_upper: BTreeMap<String, String> = BTreeMap::new();
    let mut non_empty_upper = BTreeSet::new();
    for name in &extra_cols {
        extra_upper.insert(name.to_uppercase(), name.clone());
        let upper = name.to_uppercase();
        if let Ok(series) = input.source_df.column(name) {
            for idx in 0..input.source_df.height() {
                let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
                if !value.trim().is_empty() {
                    non_empty_upper.insert(upper.clone());
                    break;
                }
            }
        }
    }
    extra_cols.retain(|name| {
        let upper = name.to_uppercase();
        if upper.ends_with("CD") && upper.len() > 2 {
            let base = &upper[..upper.len() - 2];
            if extra_upper.contains_key(base) && non_empty_upper.contains(base) {
                return false;
            }
        }
        true
    });

    if extra_cols.is_empty() {
        return Ok(None);
    }

    let mut row_count = input.source_df.height();
    if let Some(mapped) = input.mapped_df {
        row_count = row_count.min(mapped.height());
    }

    let idvar = infer_seq_column(input.parent_domain).and_then(|seq_var| {
        if input
            .mapped_df
            .and_then(|df| df.column(&seq_var).ok())
            .is_some()
        {
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
        if let Some(name) = key
            && let Some(entry) = values.get_mut(name)
        {
            entry.push(value);
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
        let qlabel = qlabel_for_column(col, input.source_labels);
        let qorig = qorig_for_column(col, input.derived_columns);
        for idx in 0..row_count {
            let raw_val = strip_wrapping_quotes(&column_value(input.source_df, col, idx));
            if raw_val.is_empty() {
                continue;
            }
            let study_value = parent_cols
                .study_id
                .as_deref()
                .map(|name| strip_wrapping_quotes(&column_value(input.source_df, name, idx)))
                .unwrap_or_default();
            let usubjid_value = parent_cols
                .usubjid
                .as_deref()
                .map(|name| strip_wrapping_quotes(&column_value(input.source_df, name, idx)))
                .unwrap_or_default();
            let mapped_usubjid = input
                .mapped_df
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
            let idvarval = if let (Some(mapped), Some(idvar_name)) = (input.mapped_df, &idvar) {
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
                    input.study_id.to_string()
                },
            );
            push_value(suppqual_cols.rdomain.as_deref(), parent_domain_code.clone());
            push_value(suppqual_cols.usubjid.as_deref(), final_usubjid);
            push_value(suppqual_cols.idvar.as_deref(), idvar_value);
            push_value(suppqual_cols.idvarval.as_deref(), idvarval);
            push_value(suppqual_cols.qnam.as_deref(), qnam.clone());
            push_value(suppqual_cols.qlabel.as_deref(), qlabel.clone());
            push_value(suppqual_cols.qval.as_deref(), raw_val);
            push_value(suppqual_cols.qorig.as_deref(), qorig.clone());
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

fn qlabel_for_column(name: &str, labels: Option<&BTreeMap<String, String>>) -> String {
    let label = labels
        .and_then(|map| map.get(&name.to_uppercase()))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| name.to_string());
    label.chars().take(40).collect()
}

fn qorig_for_column(name: &str, derived_columns: Option<&BTreeSet<String>>) -> String {
    if derived_columns
        .map(|set| set.contains(&name.to_uppercase()))
        .unwrap_or(false)
    {
        "Derived".to_string()
    } else {
        "CRF".to_string()
    }
}
