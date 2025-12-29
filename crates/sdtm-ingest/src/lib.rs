//! SDTM data ingestion utilities.
//!
//! This crate provides functionality for discovering, parsing, and loading
//! clinical trial source data (CSV files) into Polars DataFrames.

pub mod csv_table;
pub mod discovery;
pub mod study_metadata;

// Internal utilities for Polars AnyValue conversions.
// These are re-exported for workspace use but may be refactored in the future.
#[doc(hidden)]
pub mod polars_utils;

pub use csv_table::{CsvSchema, CsvTable, build_column_hints, read_csv_schema, read_csv_table};
pub use discovery::{discover_domain_files, list_csv_files};
pub use study_metadata::{
    AppliedStudyMetadata, CodeList, SourceColumn, StudyMetadata, apply_study_metadata,
    load_study_metadata,
};

// Re-export polars utilities for internal workspace use.
// Note: These are low-level utilities and may be restructured.
pub use polars_utils::{
    any_to_f64, any_to_i64, any_to_string, any_to_string_non_empty, format_numeric, parse_f64,
    parse_i64,
};
