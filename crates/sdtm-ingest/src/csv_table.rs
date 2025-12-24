use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};
use csv::ReaderBuilder;

use sdtm_model::ColumnHint;

#[derive(Debug, Clone)]
pub struct CsvTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

fn normalize_header(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('\u{feff}');
    let mut parts = trimmed.split_whitespace();
    let mut normalized = String::new();
    if let Some(first) = parts.next() {
        normalized.push_str(first);
        for part in parts {
            normalized.push(' ');
            normalized.push_str(part);
        }
    }
    normalized
}

pub fn read_csv_table(path: &Path) -> Result<CsvTable> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .with_context(|| format!("read csv: {}", path.display()))?;
    let raw_headers = reader
        .headers()
        .with_context(|| format!("read headers: {}", path.display()))?
        .clone();
    let headers: Vec<String> = raw_headers.iter().map(normalize_header).collect();
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.with_context(|| format!("read record: {}", path.display()))?;
        let mut row = Vec::with_capacity(headers.len());
        for idx in 0..headers.len() {
            let value = record.get(idx).unwrap_or("").trim().to_string();
            row.push(value);
        }
        rows.push(row);
    }
    Ok(CsvTable { headers, rows })
}

pub fn build_column_hints(table: &CsvTable) -> BTreeMap<String, ColumnHint> {
    let mut hints = BTreeMap::new();
    let row_count = table.rows.len();
    for (col_idx, header) in table.headers.iter().enumerate() {
        let mut non_null = 0usize;
        let mut numeric = 0usize;
        let mut uniques = BTreeSet::new();
        for row in &table.rows {
            let value = row.get(col_idx).map(String::as_str).unwrap_or("");
            let trimmed = value.trim();
            if trimmed.is_empty() {
                continue;
            }
            non_null += 1;
            uniques.insert(trimmed.to_string());
            if trimmed.parse::<f64>().is_ok() {
                numeric += 1;
            }
        }
        let null_ratio = if row_count == 0 {
            1.0
        } else {
            (row_count.saturating_sub(non_null)) as f64 / row_count as f64
        };
        let unique_ratio = if non_null == 0 {
            0.0
        } else {
            uniques.len() as f64 / non_null as f64
        };
        let is_numeric = non_null > 0 && numeric == non_null;
        hints.insert(
            header.clone(),
            ColumnHint {
                is_numeric,
                unique_ratio,
                null_ratio,
            },
        );
    }
    hints
}
