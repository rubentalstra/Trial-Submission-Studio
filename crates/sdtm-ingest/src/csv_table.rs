use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;
use sdtm_model::ColumnHint;

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

    let headers: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();
    
    Ok(CsvSchema {
        headers,
        labels: None, // Label detection removed for optimization
    })
}

pub fn read_csv_table(path: &Path) -> Result<DataFrame> {
    let mut df = CsvReadOptions::default()
        .with_has_header(true)
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

pub fn read_csv_table_with_header_match<F>(
    path: &Path,
    scan_lines: usize,
    matcher: F,
) -> Result<DataFrame>
where
    F: Fn(&[String]) -> bool,
{
    use std::io::{BufRead, BufReader};
    use std::fs::File;

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
        let headers: Vec<String> = line.split(',').map(|s| s.trim().trim_matches('"').to_string()).collect();
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
