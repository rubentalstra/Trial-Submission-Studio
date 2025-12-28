use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};
use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};

use sdtm_model::ColumnHint;

/// Options for controlling CSV ingest behavior.
///
/// These options allow overriding heuristic detection of header rows
/// and providing explicit schema hints for columns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IngestOptions {
    /// Explicit 0-based index of the header row.
    /// When set, bypasses heuristic header detection.
    pub header_row_index: Option<usize>,

    /// Optional 0-based index of the label row (typically immediately before headers).
    /// When set, extracts variable labels from this row.
    pub label_row_index: Option<usize>,

    /// Explicit schema definition for columns.
    /// When provided, uses these headers/labels instead of detecting from file.
    pub schema: Option<SchemaHint>,

    /// Maximum number of rows to scan for header detection.
    /// Defaults to 12 if not specified.
    pub max_header_scan_rows: Option<usize>,
}

impl IngestOptions {
    /// Create options with an explicit header row index.
    pub fn with_header_row(index: usize) -> Self {
        Self {
            header_row_index: Some(index),
            ..Default::default()
        }
    }

    /// Create options with explicit schema.
    pub fn with_schema(headers: Vec<String>, labels: Option<Vec<String>>) -> Self {
        Self {
            schema: Some(SchemaHint { headers, labels }),
            ..Default::default()
        }
    }

    /// Set the label row index.
    pub fn label_row(mut self, index: usize) -> Self {
        self.label_row_index = Some(index);
        self
    }
}

/// Explicit schema hint for a CSV file.
///
/// When provided, this schema is used instead of detecting headers from the file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaHint {
    /// Column headers in order.
    pub headers: Vec<String>,
    /// Optional column labels (variable labels).
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct CsvTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct CsvSchema {
    pub headers: Vec<String>,
    pub labels: Option<Vec<String>>,
}

pub(crate) fn normalize_header(raw: &str) -> String {
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

fn read_csv_rows_internal(path: &Path, max_rows: Option<usize>) -> Result<Vec<Vec<String>>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(path)
        .with_context(|| format!("read csv: {}", path.display()))?;
    let mut rows: Vec<Vec<String>> = Vec::new();
    for record in reader.records() {
        let record = record.with_context(|| format!("read record: {}", path.display()))?;
        let row: Vec<String> = record.iter().map(normalize_cell).collect();
        if row.iter().all(|value| value.trim().is_empty()) {
            continue;
        }
        rows.push(row);
        if let Some(limit) = max_rows
            && rows.len() >= limit
        {
            break;
        }
    }
    Ok(rows)
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
    let probe = rows.len().min(10);
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
    for (idx, stat) in stats.iter().take(search_end).enumerate() {
        let stat = *stat;
        if is_identifier_row(stat) {
            candidate = idx;
            picked_identifier = true;
        } else if !picked_identifier && is_header_like(stat) {
            candidate = idx;
        }
    }
    candidate
}

fn build_headers_and_labels(
    raw_rows: &[Vec<String>],
    header_index: usize,
) -> (Vec<String>, Option<Vec<String>>) {
    let labels = if header_index > 0 {
        let candidate = &raw_rows[header_index - 1];
        let stats = row_stats(candidate);
        if is_header_like(stats) && !is_identifier_row(stats) {
            Some(
                candidate
                    .iter()
                    .map(|value| normalize_header(value))
                    .collect(),
            )
        } else {
            None
        }
    } else {
        None
    };
    let headers: Vec<String> = raw_rows[header_index]
        .iter()
        .map(|value| normalize_header(value))
        .collect();
    (headers, labels)
}

fn build_csv_schema_from_rows(raw_rows: &[Vec<String>], header_index: usize) -> CsvSchema {
    let (headers, labels) = build_headers_and_labels(raw_rows, header_index);
    CsvSchema { headers, labels }
}

fn build_csv_table_from_rows(raw_rows: &[Vec<String>], header_index: usize) -> CsvTable {
    let (headers, labels) = build_headers_and_labels(raw_rows, header_index);
    let mut rows = Vec::new();
    for record in raw_rows.iter().skip(header_index + 1) {
        let mut row = Vec::with_capacity(headers.len());
        for idx in 0..headers.len() {
            row.push(record.get(idx).cloned().unwrap_or_default());
        }
        rows.push(row);
    }
    CsvTable {
        headers,
        rows,
        labels,
    }
}

pub(crate) fn find_header_row_by_match<F>(
    raw_rows: &[Vec<String>],
    max_scan_rows: usize,
    mut match_header: F,
) -> Option<usize>
where
    F: FnMut(&[String]) -> bool,
{
    let limit = raw_rows.len().min(max_scan_rows.max(1));
    for (idx, row) in raw_rows.iter().take(limit).enumerate() {
        let headers: Vec<String> = row.iter().map(|value| normalize_header(value)).collect();
        if match_header(&headers) {
            return Some(idx);
        }
    }
    None
}

pub fn read_csv_schema(path: &Path) -> Result<CsvSchema> {
    let raw_rows = read_csv_rows_internal(path, Some(12))?;
    if raw_rows.is_empty() {
        return Ok(CsvSchema {
            headers: Vec::new(),
            labels: None,
        });
    }
    let header_index = detect_header_row(&raw_rows);
    Ok(build_csv_schema_from_rows(&raw_rows, header_index))
}

pub fn read_csv_table(path: &Path) -> Result<CsvTable> {
    let raw_rows = read_csv_rows_internal(path, None)?;
    if raw_rows.is_empty() {
        return Ok(CsvTable {
            headers: Vec::new(),
            rows: Vec::new(),
            labels: None,
        });
    }
    let header_index = detect_header_row(&raw_rows);
    Ok(build_csv_table_from_rows(&raw_rows, header_index))
}

pub(crate) fn read_csv_table_with_header_match<F>(
    path: &Path,
    max_scan_rows: usize,
    match_header: F,
) -> Result<CsvTable>
where
    F: FnMut(&[String]) -> bool,
{
    let raw_rows = read_csv_rows_internal(path, None)?;
    if raw_rows.is_empty() {
        return Ok(CsvTable {
            headers: Vec::new(),
            rows: Vec::new(),
            labels: None,
        });
    }
    let header_index = find_header_row_by_match(&raw_rows, max_scan_rows, match_header)
        .unwrap_or_else(|| detect_header_row(&raw_rows));
    Ok(build_csv_table_from_rows(&raw_rows, header_index))
}

pub fn build_column_hints(table: &CsvTable) -> BTreeMap<String, ColumnHint> {
    let mut hints = BTreeMap::new();
    let row_count = table.rows.len();
    for (col_idx, header) in table.headers.iter().enumerate() {
        let label = table
            .labels
            .as_ref()
            .and_then(|labels| labels.get(col_idx))
            .cloned();
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
                label,
            },
        );
    }
    hints
}

/// Read CSV table with explicit options for header/label detection.
///
/// When `options.header_row_index` is set, uses that row as headers.
/// When `options.schema` is set, uses the provided schema directly.
/// Otherwise falls back to heuristic detection.
pub fn read_csv_table_with_options(path: &Path, options: &IngestOptions) -> Result<CsvTable> {
    // If explicit schema provided, read all rows as data
    if let Some(schema) = &options.schema {
        let raw_rows = read_csv_rows_internal(path, None)?;
        let rows: Vec<Vec<String>> = raw_rows
            .into_iter()
            .map(|row| {
                let mut padded = Vec::with_capacity(schema.headers.len());
                for idx in 0..schema.headers.len() {
                    padded.push(row.get(idx).cloned().unwrap_or_default());
                }
                padded
            })
            .collect();
        return Ok(CsvTable {
            headers: schema.headers.clone(),
            rows,
            labels: schema.labels.clone(),
        });
    }

    let raw_rows = read_csv_rows_internal(path, None)?;
    if raw_rows.is_empty() {
        return Ok(CsvTable {
            headers: Vec::new(),
            rows: Vec::new(),
            labels: None,
        });
    }

    let header_index = resolve_header_index(&raw_rows, options);
    let labels = resolve_labels(&raw_rows, header_index, options);
    let headers: Vec<String> = raw_rows
        .get(header_index)
        .map(|row| row.iter().map(|v| normalize_header(v)).collect())
        .unwrap_or_default();

    let mut rows = Vec::new();
    for record in raw_rows.iter().skip(header_index + 1) {
        let mut row = Vec::with_capacity(headers.len());
        for idx in 0..headers.len() {
            row.push(record.get(idx).cloned().unwrap_or_default());
        }
        rows.push(row);
    }

    Ok(CsvTable {
        headers,
        rows,
        labels,
    })
}

/// Resolve the header row index from options or heuristics.
fn resolve_header_index(raw_rows: &[Vec<String>], options: &IngestOptions) -> usize {
    if let Some(explicit_index) = options.header_row_index {
        // Clamp to valid range
        return explicit_index.min(raw_rows.len().saturating_sub(1));
    }
    detect_header_row(raw_rows)
}

/// Resolve labels from explicit index or heuristics.
fn resolve_labels(
    raw_rows: &[Vec<String>],
    header_index: usize,
    options: &IngestOptions,
) -> Option<Vec<String>> {
    // Explicit label row takes precedence
    if let Some(label_index) = options.label_row_index
        && label_index < raw_rows.len()
        && label_index != header_index
    {
        return Some(
            raw_rows[label_index]
                .iter()
                .map(|v| normalize_header(v))
                .collect(),
        );
    }

    // Fall back to heuristic: check row before header
    if header_index > 0 {
        let candidate = &raw_rows[header_index - 1];
        let stats = row_stats(candidate);
        if is_header_like(stats) && !is_identifier_row(stats) {
            return Some(candidate.iter().map(|v| normalize_header(v)).collect());
        }
    }

    None
}
