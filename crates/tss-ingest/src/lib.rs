//! SDTM data ingestion utilities.
//!
//! This crate provides functionality for discovering, parsing, and loading
//! clinical trial source data (CSV files) into Polars DataFrames.
//!
//! # Features
//!
//! - **CSV Loading**: Read CSV files with explicit header row configuration
//! - **File Discovery**: List CSV files in a study folder
//! - **Metadata Loading**: Load Items.csv for column labels (explicit path)
//! - **Column Hints**: Extract column statistics for mapping suggestions
//!
//! # Example
//!
//! ```ignore
//! use std::path::Path;
//! use tss_ingest::{list_csv_files, load_items_metadata, read_csv_table};
//!
//! let study_dir = Path::new("mockdata/STUDY001");
//!
//! // Discover CSV files
//! let csv_files = list_csv_files(study_dir)?;
//!
//! // Load metadata from explicit Items.csv path
//! let items_path = study_dir.join("Items.csv");
//! let metadata = load_items_metadata(&items_path, 2)?;
//!
//! // Read a domain CSV (1 = single header, 2 = double header)
//! let (df, headers) = read_csv_table(study_dir.join("DM.csv").as_path(), 1)?;
//! ```

mod csv;
mod discovery;
mod error;
mod hints;
mod metadata;

// === Error Types ===
pub use error::{IngestError, Result};

// === CSV Reading ===
pub use csv::{CsvHeaders, read_csv_schema, read_csv_table};

// === File Discovery ===
pub use discovery::list_csv_files;

// === Metadata Types ===
pub use metadata::{
    AppliedStudyMetadata, SourceColumn, StudyCodelist, StudyMetadata, apply_study_metadata,
    load_items_metadata,
};

// === Column Hints ===
pub use hints::{build_column_hints, get_sample_values};
