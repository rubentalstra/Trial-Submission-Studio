use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use sdtm_model::Domain;

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
    if safe.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        safe = format!("Q{safe}");
    }
    safe.chars().take(8).collect()
}

fn column_value(df: &DataFrame, name: &str, idx: usize) -> String {
    match df.column(name) {
        Ok(series) => any_to_string(series.get(idx).unwrap_or(AnyValue::Null)),
        Err(_) => String::new(),
    }
}

pub fn build_suppqual(
    parent_domain: &Domain,
    suppqual_domain: &Domain,
    source_df: &DataFrame,
    mapped_df: Option<&DataFrame>,
    used_source_columns: &BTreeSet<String>,
    study_id: &str,
) -> Result<Option<SuppqualResult>> {
    let parent_domain_code = parent_domain.code.to_uppercase();
    let ordered_columns = ordered_variable_names(suppqual_domain);
    let core_variables = variable_name_set(parent_domain);
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
        extra_cols.push(name);
    }

    if extra_cols.is_empty() {
        return Ok(None);
    }

    let mut row_count = source_df.height();
    if let Some(mapped) = mapped_df {
        row_count = row_count.min(mapped.height());
    }

    let seq_var = format!("{parent_domain_code}SEQ");
    let idvar = if mapped_df
        .and_then(|df| df.column(&seq_var).ok())
        .is_some()
    {
        Some(seq_var)
    } else {
        None
    };

    let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for key in &ordered_columns {
        values.insert(key.to_string(), Vec::new());
    }

    for col in &extra_cols {
        for idx in 0..row_count {
            let raw_val = column_value(source_df, col, idx).trim().to_string();
            if raw_val.is_empty() {
                continue;
            }
            let study_value = column_value(source_df, "STUDYID", idx);
            let usubjid_value = column_value(source_df, "USUBJID", idx);
            let mapped_usubjid = mapped_df
                .map(|df| column_value(df, "USUBJID", idx))
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

            if let Some(entry) = values.get_mut("STUDYID") {
                entry.push(if !study_value.is_empty() {
                    study_value
                } else {
                    study_id.to_string()
                });
            }
            if let Some(entry) = values.get_mut("RDOMAIN") {
                entry.push(parent_domain_code.clone());
            }
            if let Some(entry) = values.get_mut("USUBJID") {
                entry.push(final_usubjid);
            }
            if let Some(entry) = values.get_mut("IDVAR") {
                entry.push(idvar_value);
            }
            if let Some(entry) = values.get_mut("IDVARVAL") {
                entry.push(idvarval);
            }
            if let Some(entry) = values.get_mut("QNAM") {
                entry.push(sanitize_qnam(col));
            }
            if let Some(entry) = values.get_mut("QLABEL") {
                entry.push(col.to_string());
            }
            if let Some(entry) = values.get_mut("QVAL") {
                entry.push(raw_val);
            }
            if let Some(entry) = values.get_mut("QORIG") {
                entry.push("CRF".to_string());
            }
            if let Some(entry) = values.get_mut("QEVAL") {
                entry.push(String::new());
            }
        }
    }

    let total_rows = values.get("QVAL").map(|vals| vals.len()).unwrap_or(0);
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
