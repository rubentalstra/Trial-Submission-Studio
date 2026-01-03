//! SDTM data ingestion utilities.
//!
//! This crate provides functionality for discovering, parsing, and loading
//! clinical trial source data (CSV files) into Polars DataFrames.
//!
//! # Features
//!
//! - **CSV Loading**: Read CSV files with explicit header row configuration
//! - **Domain Discovery**: Find and classify CSV files by SDTM domain
//! - **Metadata Loading**: Load Items.csv and CodeLists.csv with dynamic schema detection
//! - **Column Hints**: Extract column statistics for mapping suggestions
//!
//! # Example
//!
//! ```ignore
//! use std::path::Path;
//! use tss_ingest::{list_csv_files, discover_domain_files, load_study_metadata, read_csv_table};
//!
//! let study_dir = Path::new("mockdata/STUDY001");
//!
//! // Discover CSV files
//! let csv_files = list_csv_files(study_dir)?;
//!
//! // Load study metadata
//! let metadata = load_study_metadata(study_dir)?;
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
pub use discovery::{DiscoveredFile, discover_domain_files, discover_files, list_csv_files};

// === Metadata Types ===
pub use metadata::{
    AppliedStudyMetadata, SourceColumn, StudyCodelist, StudyMetadata, apply_study_metadata,
    load_study_metadata,
};

// === Column Hints ===
pub use hints::{build_column_hints, get_sample_values};
