use polars::prelude::{AnyValue, DataFrame};
use sdtm_ingest::CsvTable;
use sdtm_ingest::any_to_string;
use sdtm_model::MappingConfig;

pub fn column_value_string(df: &DataFrame, name: &str, idx: usize) -> String {
    match df.column(name) {
        Ok(series) => any_to_string(series.get(idx).unwrap_or(AnyValue::Null)),
        Err(_) => String::new(),
    }
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
