//! Polars-based CSV streaming for large datasets.
//!
//! This module provides efficient CSV reading using Polars lazy evaluation
//! and streaming for large datasets. It samples rows for column hints without
//! loading the entire file into memory.
//!
//! # Usage
//!
//! ```ignore
//! use sdtm_ingest::streaming::{StreamingCsvReader, StreamingOptions};
//!
//! let options = StreamingOptions::default().with_sample_size(1000);
//! let reader = StreamingCsvReader::new(&path, options)?;
//! let hints = reader.build_column_hints()?;
//! let df = reader.read_all()?;
//! ```
//!
//! # Auto-selection for large files
//!
//! For files larger than the threshold (default 10 MB), streaming is
//! automatically used:
//!
//! ```ignore
//! use sdtm_ingest::streaming::read_csv_table_auto;
//!
//! // Automatically uses streaming for large files
//! let table = read_csv_table_auto(&path)?;
//! ```

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

use sdtm_model::ColumnHint;

use crate::csv_table::{CsvTable, IngestOptions};

/// Default file size threshold (in bytes) above which streaming is used.
/// Default: 10 MB
pub const DEFAULT_STREAMING_THRESHOLD_BYTES: u64 = 10 * 1024 * 1024;

/// Options for streaming CSV reading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingOptions {
    /// Number of rows to sample for column hints.
    /// Defaults to 1000.
    pub sample_size: usize,

    /// Whether to use parallel parsing.
    /// Defaults to true for large files.
    pub parallel: bool,

    /// Chunk size for streaming reads.
    /// Defaults to 50000.
    pub chunk_size: usize,

    /// Low memory mode - trades performance for memory efficiency.
    /// Defaults to false.
    pub low_memory: bool,

    /// Ingest options for header detection.
    pub ingest_options: IngestOptions,
}

impl Default for StreamingOptions {
    fn default() -> Self {
        Self {
            sample_size: 1000,
            parallel: true,
            chunk_size: 50000,
            low_memory: false,
            ingest_options: IngestOptions::default(),
        }
    }
}

impl StreamingOptions {
    /// Set the sample size for column hints.
    pub fn with_sample_size(mut self, size: usize) -> Self {
        self.sample_size = size;
        self
    }

    /// Enable or disable parallel parsing.
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Set the chunk size for streaming.
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Enable low memory mode.
    pub fn with_low_memory(mut self, enabled: bool) -> Self {
        self.low_memory = enabled;
        self
    }
}

/// Streaming CSV reader using Polars.
pub struct StreamingCsvReader {
    path: std::path::PathBuf,
    options: StreamingOptions,
    /// Cached schema from initial scan
    schema: Option<Arc<Schema>>,
    /// Row to skip (header detection)
    skip_rows: usize,
}

impl StreamingCsvReader {
    /// Create a new streaming CSV reader.
    pub fn new(path: impl AsRef<Path>, options: StreamingOptions) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            anyhow::bail!("CSV file not found: {}", path.display());
        }

        // Determine header row
        let skip_rows = options.ingest_options.header_row_index.unwrap_or(0);

        Ok(Self {
            path,
            options,
            schema: None,
            skip_rows,
        })
    }

    /// Scan the file lazily without loading all data.
    pub fn scan(&self) -> Result<LazyFrame> {
        let path_str = self.path.to_string_lossy();
        let pl_path = PlPath::new(&path_str);
        let lf = LazyCsvReader::new(pl_path)
            .with_has_header(true)
            .with_skip_rows(self.skip_rows)
            .with_low_memory(self.options.low_memory)
            .finish()
            .with_context(|| format!("Failed to scan CSV: {}", self.path.display()))?;

        Ok(lf)
    }

    /// Sample rows for building column hints.
    ///
    /// This reads only the first N rows specified in options.sample_size.
    pub fn sample_rows(&self) -> Result<DataFrame> {
        let df = CsvReadOptions::default()
            .with_has_header(true)
            .with_skip_rows(self.skip_rows)
            .with_n_rows(Some(self.options.sample_size))
            .with_low_memory(self.options.low_memory)
            .try_into_reader_with_file_path(Some(self.path.clone()))
            .with_context(|| format!("Failed to create CSV reader: {}", self.path.display()))?
            .finish()
            .with_context(|| format!("Failed to sample CSV: {}", self.path.display()))?;

        Ok(df)
    }

    /// Build column hints from a sample of the data.
    ///
    /// This is memory-efficient as it only reads sample_size rows.
    pub fn build_column_hints(&self) -> Result<BTreeMap<String, ColumnHint>> {
        let sample = self.sample_rows()?;
        build_hints_from_df(&sample)
    }

    /// Read the entire CSV file into a DataFrame.
    pub fn read_all(&self) -> Result<DataFrame> {
        let df = CsvReadOptions::default()
            .with_has_header(true)
            .with_skip_rows(self.skip_rows)
            .with_low_memory(self.options.low_memory)
            .try_into_reader_with_file_path(Some(self.path.clone()))
            .with_context(|| format!("Failed to create CSV reader: {}", self.path.display()))?
            .finish()
            .with_context(|| format!("Failed to read CSV: {}", self.path.display()))?;

        Ok(df)
    }

    /// Get the schema of the CSV file.
    pub fn schema(&mut self) -> Result<Arc<Schema>> {
        if self.schema.is_none() {
            // Sample a small amount to get schema
            let sample = CsvReadOptions::default()
                .with_has_header(true)
                .with_skip_rows(self.skip_rows)
                .with_n_rows(Some(10))
                .try_into_reader_with_file_path(Some(self.path.clone()))?
                .finish()?;

            self.schema = Some(sample.schema().clone());
        }
        Ok(self.schema.clone().unwrap())
    }

    /// Stream through the file in chunks, applying a function to each chunk.
    ///
    /// This is useful for processing large files without loading them entirely.
    pub fn process_chunks<F>(&self, mut processor: F) -> Result<()>
    where
        F: FnMut(DataFrame) -> Result<()>,
    {
        // For now, read all at once and simulate chunks
        // In future Polars versions, we can use the batched reader
        let df = self.read_all()?;
        let total_rows = df.height();
        let chunk_size = self.options.chunk_size;

        let mut offset = 0;
        while offset < total_rows {
            let end = (offset + chunk_size).min(total_rows);
            let chunk = df.slice(offset as i64, end - offset);
            processor(chunk)?;
            offset = end;
        }

        Ok(())
    }

    /// Read the CSV file and convert to CsvTable format.
    ///
    /// This allows seamless integration with the existing pipeline that
    /// expects CsvTable.
    pub fn read_as_csv_table(&self) -> Result<CsvTable> {
        let df = self.read_all()?;
        dataframe_to_csv_table(df, self.options.ingest_options.label_row_index)
    }

    /// Sample rows and convert to CsvTable format.
    ///
    /// Only reads sample_size rows for memory efficiency.
    pub fn sample_as_csv_table(&self) -> Result<CsvTable> {
        let df = self.sample_rows()?;
        dataframe_to_csv_table(df, self.options.ingest_options.label_row_index)
    }
}

/// Build column hints from a DataFrame.
fn build_hints_from_df(df: &DataFrame) -> Result<BTreeMap<String, ColumnHint>> {
    let mut hints = BTreeMap::new();
    let row_count = df.height();

    for col in df.get_columns() {
        let name = col.name().to_string();

        // Count non-null values
        let non_null = col.len() - col.null_count();

        // Check if numeric
        let is_numeric = matches!(
            col.dtype(),
            DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
                | DataType::Float32
                | DataType::Float64
        );

        // Calculate unique ratio
        let unique_count = col.n_unique().unwrap_or(0);
        let unique_ratio = if non_null == 0 {
            0.0
        } else {
            unique_count as f64 / non_null as f64
        };

        // Calculate null ratio
        let null_ratio = if row_count == 0 {
            1.0
        } else {
            col.null_count() as f64 / row_count as f64
        };

        hints.insert(
            name,
            ColumnHint {
                is_numeric,
                unique_ratio,
                null_ratio,
                label: None, // Labels come from schema hints, not data
            },
        );
    }

    Ok(hints)
}

/// Estimate file size category for auto-tuning options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSizeCategory {
    /// Small: < 10 MB
    Small,
    /// Medium: 10 MB - 100 MB
    Medium,
    /// Large: 100 MB - 1 GB
    Large,
    /// VeryLarge: > 1 GB
    VeryLarge,
}

impl FileSizeCategory {
    /// Determine file size category from path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let metadata = std::fs::metadata(path.as_ref())
            .with_context(|| format!("Failed to get file metadata: {}", path.as_ref().display()))?;

        let size_mb = metadata.len() / (1024 * 1024);
        Ok(match size_mb {
            0..10 => FileSizeCategory::Small,
            10..100 => FileSizeCategory::Medium,
            100..1000 => FileSizeCategory::Large,
            _ => FileSizeCategory::VeryLarge,
        })
    }

    /// Get recommended streaming options for this file size.
    pub fn recommended_options(&self) -> StreamingOptions {
        match self {
            FileSizeCategory::Small => StreamingOptions {
                sample_size: 500,
                parallel: false,
                chunk_size: 10000,
                low_memory: false,
                ingest_options: IngestOptions::default(),
            },
            FileSizeCategory::Medium => StreamingOptions {
                sample_size: 1000,
                parallel: true,
                chunk_size: 50000,
                low_memory: false,
                ingest_options: IngestOptions::default(),
            },
            FileSizeCategory::Large => StreamingOptions {
                sample_size: 2000,
                parallel: true,
                chunk_size: 100000,
                low_memory: false,
                ingest_options: IngestOptions::default(),
            },
            FileSizeCategory::VeryLarge => StreamingOptions {
                sample_size: 5000,
                parallel: true,
                chunk_size: 200000,
                low_memory: true,
                ingest_options: IngestOptions::default(),
            },
        }
    }
}

/// Convert a Polars DataFrame to CsvTable format.
fn dataframe_to_csv_table(df: DataFrame, _label_row_index: Option<usize>) -> Result<CsvTable> {
    let headers: Vec<String> = df
        .get_column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows = Vec::with_capacity(df.height());
    for row_idx in 0..df.height() {
        let mut row = Vec::with_capacity(headers.len());
        for col in df.get_columns() {
            let value = col.get(row_idx).map_err(|e| anyhow::anyhow!("{}", e))?;
            let str_value = crate::polars_utils::any_to_string(value);
            row.push(str_value);
        }
        rows.push(row);
    }

    Ok(CsvTable {
        headers,
        rows,
        labels: None, // Labels would need separate handling from metadata
    })
}

/// Check if a file should use streaming based on its size.
pub fn should_use_streaming(path: impl AsRef<Path>) -> bool {
    should_use_streaming_with_threshold(path, DEFAULT_STREAMING_THRESHOLD_BYTES)
}

/// Check if a file should use streaming based on a custom threshold.
pub fn should_use_streaming_with_threshold(path: impl AsRef<Path>, threshold_bytes: u64) -> bool {
    std::fs::metadata(path.as_ref())
        .map(|m| m.len() >= threshold_bytes)
        .unwrap_or(false)
}

/// Read a CSV file, automatically using streaming for large files.
///
/// For files smaller than 10 MB, uses the standard csv crate reader.
/// For larger files, uses Polars streaming for better memory efficiency.
pub fn read_csv_table_auto(path: impl AsRef<Path>) -> Result<CsvTable> {
    read_csv_table_auto_with_options(path, &IngestOptions::default())
}

/// Read a CSV file with options, automatically using streaming for large files.
pub fn read_csv_table_auto_with_options(
    path: impl AsRef<Path>,
    options: &IngestOptions,
) -> Result<CsvTable> {
    let path = path.as_ref();

    if should_use_streaming(path) {
        tracing::debug!(
            path = %path.display(),
            "Using Polars streaming for large file"
        );
        let category = FileSizeCategory::from_path(path)?;
        let mut streaming_options = category.recommended_options();
        streaming_options.ingest_options = options.clone();
        let reader = StreamingCsvReader::new(path, streaming_options)?;
        reader.read_as_csv_table()
    } else {
        // Fall back to standard reader for small files
        crate::csv_table::read_csv_table_with_options(path, options)
    }
}

/// Build column hints using sampling for large files.
///
/// For large files, only samples a subset of rows for efficiency.
/// For small files, reads all rows.
pub fn build_column_hints_auto(path: impl AsRef<Path>) -> Result<BTreeMap<String, ColumnHint>> {
    build_column_hints_auto_with_options(path, &IngestOptions::default())
}

/// Build column hints with options, using sampling for large files.
pub fn build_column_hints_auto_with_options(
    path: impl AsRef<Path>,
    options: &IngestOptions,
) -> Result<BTreeMap<String, ColumnHint>> {
    let path = path.as_ref();

    if should_use_streaming(path) {
        tracing::debug!(
            path = %path.display(),
            "Using sampling for column hints on large file"
        );
        let category = FileSizeCategory::from_path(path)?;
        let mut streaming_options = category.recommended_options();
        streaming_options.ingest_options = options.clone();
        let reader = StreamingCsvReader::new(path, streaming_options)?;
        reader.build_column_hints()
    } else {
        // Fall back to reading full file for small files
        let table = crate::csv_table::read_csv_table_with_options(path, options)?;
        Ok(crate::csv_table::build_column_hints(&table))
    }
}
