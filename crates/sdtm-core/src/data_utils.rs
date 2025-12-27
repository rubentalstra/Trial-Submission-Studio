use anyhow::Result;
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};
use sdtm_ingest::CsvTable;
use sdtm_model::{Domain, MappingConfig};

pub fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        AnyValue::Null => String::new(),
        _ => value.to_string(),
    }
}

pub fn any_to_string_for_output(value: AnyValue) -> String {
    match value {
        AnyValue::Null => String::new(),
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        AnyValue::Float64(value) => format_numeric(value),
        AnyValue::Float32(value) => format_numeric(value as f64),
        AnyValue::Int64(value) => value.to_string(),
        AnyValue::Int32(value) => value.to_string(),
        AnyValue::Int16(value) => value.to_string(),
        AnyValue::Int8(value) => value.to_string(),
        AnyValue::UInt64(value) => value.to_string(),
        AnyValue::UInt32(value) => value.to_string(),
        AnyValue::UInt16(value) => value.to_string(),
        AnyValue::UInt8(value) => value.to_string(),
        AnyValue::Boolean(value) => {
            if value {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
        value => value.to_string(),
    }
}

pub fn format_numeric(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}

pub fn any_to_f64(value: AnyValue) -> Option<f64> {
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

pub fn any_to_f64_for_output(value: AnyValue) -> Option<f64> {
    match value {
        AnyValue::Boolean(value) => Some(if value { 1.0 } else { 0.0 }),
        _ => any_to_f64(value),
    }
}

pub fn any_to_i64(value: AnyValue) -> Option<i64> {
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

pub fn parse_f64(value: &str) -> Option<f64> {
    if value.trim().is_empty() {
        return None;
    }
    value.trim().parse::<f64>().ok()
}

pub fn parse_i64(value: &str) -> Option<i64> {
    if value.trim().is_empty() {
        return None;
    }
    value.trim().parse::<i64>().ok()
}

pub fn column_value_string(df: &DataFrame, name: &str, idx: usize) -> String {
    match df.column(name) {
        Ok(series) => any_to_string(series.get(idx).unwrap_or(AnyValue::Null)),
        Err(_) => String::new(),
    }
}

pub fn fill_string_column(df: &mut DataFrame, name: &str, fill: &str) -> Result<()> {
    if fill.is_empty() {
        return Ok(());
    }
    let mut values = if let Ok(series) = df.column(name) {
        (0..df.height())
            .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
            .collect::<Vec<_>>()
    } else {
        vec![String::new(); df.height()]
    };
    for value in &mut values {
        if value.trim().is_empty() {
            *value = fill.to_string();
        }
    }
    let series = Series::new(name.into(), values);
    df.with_column(series)?;
    Ok(())
}

pub fn table_label(table: &CsvTable, column: &str) -> Option<String> {
    let labels = table.labels.as_ref()?;
    let idx = table
        .headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(column))?;
    let label = labels.get(idx)?.trim();
    if label.is_empty() {
        None
    } else {
        Some(label.to_string())
    }
}

pub fn column_hint_for_domain(
    table: &CsvTable,
    domain: &Domain,
    column: &str,
) -> Option<(String, bool)> {
    let idx = table
        .headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(column))?;
    if let Some(labels) = table.labels.as_ref()
        && let Some(label) = labels.get(idx)
    {
        let trimmed = label.trim();
        if !trimmed.is_empty() {
            return Some((trimmed.to_string(), true));
        }
    }
    let header = table.headers.get(idx)?.clone();
    let is_standard = domain
        .variables
        .iter()
        .any(|var| var.name.eq_ignore_ascii_case(&header));
    if is_standard {
        None
    } else {
        Some((header, false))
    }
}

pub fn table_column_values(table: &CsvTable, column: &str) -> Option<Vec<String>> {
    let idx = table
        .headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(column))?;
    let mut values = Vec::with_capacity(table.rows.len());
    for row in &table.rows {
        values.push(row.get(idx).cloned().unwrap_or_default());
    }
    Some(values)
}

pub fn mapping_source_for_target(mapping: &MappingConfig, target: &str) -> Option<String> {
    mapping
        .mappings
        .iter()
        .find(|entry| entry.target_variable.eq_ignore_ascii_case(target))
        .map(|entry| entry.source_column.clone())
}

pub fn sanitize_test_code(raw: &str) -> String {
    let mut safe = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            safe.push(ch.to_ascii_uppercase());
        } else {
            safe.push('_');
        }
    }
    if safe.is_empty() {
        safe = "TEST".to_string();
    }
    if safe
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        safe.insert(0, 'T');
    }
    safe.chars().take(8).collect()
}
