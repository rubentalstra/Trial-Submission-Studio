//! Shared CSV utilities for loading standards files.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use csv::ReaderBuilder;

/// Environment variable for overriding the standards directory.
pub const STANDARDS_ENV_VAR: &str = "CDISC_STANDARDS_DIR";

/// Get the default standards root directory.
///
/// Checks the `CDISC_STANDARDS_DIR` environment variable first,
/// then falls back to the `standards/` directory relative to the crate.
pub fn default_standards_root() -> PathBuf {
    if let Ok(root) = std::env::var(STANDARDS_ENV_VAR) {
        return PathBuf::from(root);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards")
}

/// Read a CSV file into a vector of row maps.
///
/// Each row is represented as a BTreeMap with column headers as keys.
/// Handles BOM characters and trims whitespace from values.
pub fn read_csv_rows(path: &Path) -> Result<Vec<BTreeMap<String, String>>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .with_context(|| format!("read csv: {}", path.display()))?;

    let headers = reader
        .headers()
        .with_context(|| format!("read headers: {}", path.display()))?
        .clone();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.with_context(|| format!("read record: {}", path.display()))?;
        let mut row = BTreeMap::new();
        for (idx, value) in record.iter().enumerate() {
            let key = headers
                .get(idx)
                .unwrap_or("")
                .trim_matches('\u{feff}')
                .to_string();
            row.insert(key, value.trim().to_string());
        }
        rows.push(row);
    }
    Ok(rows)
}

/// Get a field value from a row, returning empty string if not present.
pub fn get_field(row: &BTreeMap<String, String>, key: &str) -> String {
    row.get(key).cloned().unwrap_or_default()
}

/// Get an optional field value from a row (None if empty or missing).
pub fn get_optional(row: &BTreeMap<String, String>, key: &str) -> Option<String> {
    row.get(key).filter(|v| !v.is_empty()).cloned()
}
