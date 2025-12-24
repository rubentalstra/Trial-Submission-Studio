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

fn normalize_cell(raw: &str) -> String {
    raw.trim().trim_matches('\u{feff}').to_string()
}

#[derive(Debug, Default, Clone, Copy)]
struct RowStats {
    total: usize,
    non_empty: usize,
    numeric: usize,
    alpha: usize,
    identifier: usize,
    with_space: usize,
}

impl RowStats {
    fn non_empty_ratio(self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.non_empty as f64 / self.total as f64
        }
    }

    fn numeric_ratio(self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.numeric as f64 / self.total as f64
        }
    }

    fn alpha_ratio(self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.alpha as f64 / self.total as f64
        }
    }

    fn identifier_ratio(self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.identifier as f64 / self.total as f64
        }
    }

    fn empty_ratio(self) -> f64 {
        1.0 - self.non_empty_ratio()
    }
}

fn row_stats(row: &[String]) -> RowStats {
    let mut stats = RowStats {
        total: row.len(),
        ..RowStats::default()
    };
    for cell in row {
        let trimmed = cell.trim();
        if trimmed.is_empty() {
            continue;
        }
        stats.non_empty += 1;
        if trimmed.parse::<f64>().is_ok() {
            stats.numeric += 1;
        }
        if trimmed.chars().any(|ch| ch.is_ascii_alphabetic()) {
            stats.alpha += 1;
        }
        if trimmed.contains(' ') {
            stats.with_space += 1;
        }
        if is_identifier_like(trimmed) {
            stats.identifier += 1;
        }
    }
    stats
}

fn is_identifier_like(value: &str) -> bool {
    if value.contains(' ') {
        return false;
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn is_data_like(stats: RowStats) -> bool {
    stats.numeric_ratio() >= 0.2 || stats.empty_ratio() >= 0.2
}

fn is_identifier_row(stats: RowStats) -> bool {
    stats.identifier_ratio() >= 0.6 && stats.numeric_ratio() <= 0.1
}

fn is_header_like(stats: RowStats) -> bool {
    stats.non_empty_ratio() >= 0.8 && stats.alpha_ratio() >= 0.5 && stats.numeric_ratio() <= 0.1
}

fn detect_header_row(rows: &[Vec<String>]) -> usize {
    if rows.is_empty() {
        return 0;
    }
    // Heuristic: pick the last header-like row before data starts, prefer identifier-style headers.
    let probe = rows.len().min(5);
    let stats: Vec<RowStats> = rows.iter().take(probe).map(|row| row_stats(row)).collect();
    let mut data_index = None;
    for (idx, stat) in stats.iter().enumerate() {
        if is_data_like(*stat) {
            data_index = Some(idx);
            break;
        }
    }
    let search_end = data_index.unwrap_or(1).max(1);
    let mut candidate = 0usize;
    let mut picked_identifier = false;
    for idx in 0..search_end {
        let stat = stats[idx];
        if is_identifier_row(stat) {
            candidate = idx;
            picked_identifier = true;
        } else if !picked_identifier && is_header_like(stat) {
            candidate = idx;
        }
    }
    candidate
}

pub fn read_csv_table(path: &Path) -> Result<CsvTable> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(path)
        .with_context(|| format!("read csv: {}", path.display()))?;
    let mut raw_rows: Vec<Vec<String>> = Vec::new();
    for record in reader.records() {
        let record = record.with_context(|| format!("read record: {}", path.display()))?;
        let row: Vec<String> = record.iter().map(normalize_cell).collect();
        if row.iter().all(|value| value.trim().is_empty()) {
            continue;
        }
        raw_rows.push(row);
    }
    if raw_rows.is_empty() {
        return Ok(CsvTable {
            headers: Vec::new(),
            rows: Vec::new(),
        });
    }
    let header_index = detect_header_row(&raw_rows);
    let headers: Vec<String> = raw_rows[header_index]
        .iter()
        .map(|value| normalize_header(value))
        .collect();
    let mut rows = Vec::new();
    for record in raw_rows.iter().skip(header_index + 1) {
        let mut row = Vec::with_capacity(headers.len());
        for idx in 0..headers.len() {
            let value = record.get(idx).map(String::as_str).unwrap_or("");
            row.push(normalize_cell(value));
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
