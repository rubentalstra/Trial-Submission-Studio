use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;
use sdtm_map::types::ColumnHint;

#[derive(Debug, Clone)]
pub struct CsvSchema {
    pub headers: Vec<String>,
    pub labels: Option<Vec<String>>,
}

pub fn read_csv_schema(path: &Path) -> Result<CsvSchema> {
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .with_infer_schema_length(Some(100))
        .with_n_rows(Some(1)) // Read only 1 row to get headers
        .with_ignore_errors(true)
        .try_into_reader_with_file_path(Some(path.into()))
        .context(format!("Failed to create reader for {}", path.display()))?
        .finish()
        .context(format!("Failed to read schema from {}", path.display()))?;

    let headers: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    Ok(CsvSchema {
        headers,
        labels: None, // Label detection removed for optimization
    })
}

pub fn read_csv_table(path: &Path) -> Result<DataFrame> {
    // First, detect if this CSV has a double header (label row + variable code row)
    let skip_label_row = detect_double_header(path)?;

    let mut df = CsvReadOptions::default()
        .with_has_header(true)
        .with_skip_rows(if skip_label_row { 1 } else { 0 })
        .with_infer_schema_length(Some(100))
        .with_ignore_errors(true)
        .try_into_reader_with_file_path(Some(path.into()))
        .context(format!("Failed to create reader for {}", path.display()))?
        .finish()
        .context(format!("Failed to read CSV from {}", path.display()))?;

    // Normalize headers (trim whitespace)
    let new_columns: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|name| name.trim().to_string())
        .collect();

    df.set_column_names(&new_columns)?;

    Ok(df)
}

/// Detect if a CSV has a double header pattern common in EDC exports.
///
/// Many EDC systems export CSVs with:
/// - Row 0: Human-readable labels (e.g., "Site sequence number", "Subject Id")
/// - Row 1: Variable codes/names (e.g., "SiteSeq", "SubjectId", "MENOSTAT")
/// - Row 2+: Actual data
///
/// This function detects this pattern by checking if:
/// 1. Row 0 (would-be headers) has human-readable labels (longer, often with spaces)
/// 2. Row 1 (first data row) has short codes that look like variable names
///
/// Returns true if the first row should be skipped (it's a label row, not real data).
fn detect_double_header(path: &Path) -> Result<bool> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(path).context(format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Read first two lines
    let line0 = match lines.next() {
        Some(Ok(l)) => l,
        _ => return Ok(false), // Can't detect, use standard behavior
    };
    let line1 = match lines.next() {
        Some(Ok(l)) => l,
        _ => return Ok(false), // Only one line, no double header
    };

    // Parse both lines as CSV fields
    let headers: Vec<String> = parse_csv_line(&line0);
    let row1: Vec<String> = parse_csv_line(&line1);

    // Must have same number of columns
    if headers.len() != row1.len() || headers.is_empty() {
        return Ok(false);
    }

    // Heuristic: Check if row 0 looks like labels and row 1 looks like variable codes
    // Labels tend to be longer, have spaces, mixed case
    // Variable codes tend to be shorter, uppercase, no spaces

    let label_like_headers = headers.iter().filter(|h| looks_like_label(h)).count();
    let code_like_row1 = row1.iter().filter(|v| looks_like_variable_code(v)).count();

    // If most headers look like labels AND most row1 values look like codes,
    // then this is likely a double header
    let threshold = headers.len() / 2;
    let is_double_header = label_like_headers > threshold && code_like_row1 > threshold;

    if is_double_header {
        tracing::debug!(
            "Detected double header in {}: {} label-like headers, {} code-like row1 values",
            path.display(),
            label_like_headers,
            code_like_row1
        );
    }

    Ok(is_double_header)
}

/// Check if a string looks like a human-readable label (long, may have spaces)
pub fn looks_like_label(s: &str) -> bool {
    // Labels are typically longer than 10 chars or contain spaces
    s.len() > 10 || s.contains(' ') || s.contains('-')
}

/// Check if a string looks like a variable code (short, uppercase-ish, no spaces)
pub fn looks_like_variable_code(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    // Variable codes are typically:
    // - Relatively short (< 20 chars)
    // - No spaces
    // - Often uppercase or CamelCase
    // - May contain underscores

    if s.len() > 20 || s.contains(' ') {
        return false;
    }

    // Check if it's mostly uppercase or typical variable name pattern
    let uppercase_ratio = s.chars().filter(|c| c.is_uppercase()).count() as f64 / s.len() as f64;
    let is_mostly_uppercase = uppercase_ratio > 0.5;

    // Also check for common patterns like CamelCase or snake_case
    let is_identifier_like = s
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-');

    is_mostly_uppercase || (is_identifier_like && !s.chars().any(|c| c == ' '))
}

/// Simple CSV line parser (handles quoted fields)
pub fn parse_csv_line(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                result.push(current.trim().trim_matches('"').to_string());
                current = String::new();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Don't forget the last field
    result.push(current.trim().trim_matches('"').to_string());

    result
}

pub fn read_csv_table_with_header_match<F>(
    path: &Path,
    scan_lines: usize,
    matcher: F,
) -> Result<DataFrame>
where
    F: Fn(&[String]) -> bool,
{
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(path).context(format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut skip_rows = 0;
    let mut found = false;

    for (idx, line) in reader.lines().enumerate() {
        if idx >= scan_lines {
            break;
        }
        let line = line.context("read line")?;
        // Simple CSV split (not robust but maybe enough for header detection)
        // Note: This doesn't handle quotes, but metadata files are usually simple.
        let headers: Vec<String> = line
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .collect();
        if matcher(&headers) {
            skip_rows = idx;
            found = true;
            break;
        }
    }

    if !found {
        skip_rows = 0;
    }

    let mut df = CsvReadOptions::default()
        .with_has_header(true)
        .with_skip_rows(skip_rows)
        .with_infer_schema_length(Some(100))
        .with_ignore_errors(true)
        .try_into_reader_with_file_path(Some(path.into()))
        .context(format!("Failed to create reader for {}", path.display()))?
        .finish()
        .context(format!("Failed to read CSV from {}", path.display()))?;

    // Normalize headers
    let new_columns: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|name| name.trim().to_string())
        .collect();

    df.set_column_names(&new_columns)?;

    Ok(df)
}

pub fn build_column_hints(df: &DataFrame) -> BTreeMap<String, ColumnHint> {
    let mut hints = BTreeMap::new();
    let row_count = df.height();

    for col_name in df.get_column_names() {
        let series = df.column(col_name).unwrap();
        let null_count = series.null_count();
        let non_null = row_count - null_count;

        let null_ratio = if row_count == 0 {
            1.0
        } else {
            null_count as f64 / row_count as f64
        };

        let n_unique = series.n_unique().unwrap_or(0);
        let unique_ratio = if non_null == 0 {
            0.0
        } else {
            n_unique as f64 / non_null as f64
        };

        let is_numeric = series.dtype().is_numeric();

        hints.insert(
            col_name.to_string(),
            ColumnHint {
                is_numeric,
                unique_ratio,
                null_ratio,
                label: None, // Labels not available from DataFrame directly
            },
        );
    }
    hints
}

/// Get sample unique values from a DataFrame column.
///
/// Returns up to `limit` unique non-empty values from the specified column.
/// Useful for previewing data or building UI displays.
pub fn get_sample_values(df: &DataFrame, column: &str, limit: usize) -> Vec<String> {
    let Ok(col) = df.column(column) else {
        return Vec::new();
    };

    let mut samples = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Try to get unique sample values
    if let Ok(str_col) = col.str() {
        for i in 0..df.height().min(limit * 3) {
            if let Some(val) = str_col.get(i) {
                if !val.is_empty() && seen.insert(val.to_string()) {
                    samples.push(val.to_string());
                    if samples.len() >= limit {
                        break;
                    }
                }
            }
        }
    } else {
        // For non-string columns, format as string using Display
        for i in 0..df.height().min(limit) {
            if let Ok(val) = col.get(i) {
                let formatted = format!("{val}");
                if formatted != "null" && !formatted.is_empty() && seen.insert(formatted.clone()) {
                    samples.push(formatted);
                    if samples.len() >= limit {
                        break;
                    }
                }
            }
        }
    }

    samples
}
