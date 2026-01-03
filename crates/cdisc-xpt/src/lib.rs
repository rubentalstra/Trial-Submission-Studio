//! SAS Transport (XPT) file format reader and writer.
//!
//! This crate provides functionality to read and write SAS Transport V5 and V8 format files,
//! commonly used for SDTM datasets in regulatory submissions.
//!
//! # Features
//!
//! - Full SAS Transport V5 format support (default, for maximum compatibility)
//! - SAS Transport V8 format support (longer names, labels, and formats)
//! - IEEE ↔ IBM mainframe floating-point conversion
//! - Support for all 28 SAS missing value codes (`.`, `._`, `.A`-`.Z`)
//! - Variable metadata including formats and informats
//! - Optional Polars DataFrame integration (with `polars` feature)
//!
//! # Format Versions
//!
//! | Feature | V5 Limit | V8 Limit |
//! |---------|----------|----------|
//! | Variable name | 8 chars | 32 chars |
//! | Variable label | 40 chars | 256 chars |
//! | Format name | 8 chars | 32 chars |
//! | Dataset name | 8 chars | 32 chars |
//!
//! By default, files are written in V5 format. Use [`XptWriterOptions::with_version`]
//! to write V8 format files. When reading, the format is auto-detected.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use cdisc_xpt::{XptDataset, XptColumn, XptValue, read_xpt, write_xpt};
//!
//! // Read an XPT file
//! let dataset = read_xpt(Path::new("dm.xpt")).unwrap();
//! println!("Dataset: {} ({} rows)", dataset.name, dataset.num_rows());
//!
//! // Create a new dataset
//! let mut ds = XptDataset::with_columns(
//!     "DM",
//!     vec![
//!         XptColumn::character("USUBJID", 20).with_label("Unique Subject ID"),
//!         XptColumn::numeric("AGE").with_label("Age in Years"),
//!     ],
//! );
//! ds.add_row(vec![
//!     XptValue::character("STUDY-001"),
//!     XptValue::numeric(35.0),
//! ]);
//!
//! // Write to XPT file
//! write_xpt(Path::new("dm_out.xpt"), &ds).unwrap();
//! ```
//!
//! # Missing Values
//!
//! SAS supports 28 different missing value codes:
//!
//! ```
//! use cdisc_xpt::{MissingValue, NumericValue, XptValue};
//!
//! // Standard missing (.)
//! let missing = XptValue::numeric_missing();
//!
//! // Special missing (.A through .Z)
//! let missing_a = XptValue::numeric_missing_with(MissingValue::Special('A'));
//!
//! // Check for missing
//! assert!(missing.is_missing());
//! ```

mod error;
pub mod float;
pub mod header;
pub mod reader;
mod types;
pub mod validation;
mod version;
pub mod writer;

#[cfg(feature = "polars")]
pub mod polars;

// Re-export error types
pub use error::{
    ErrorLocation, Result, Severity, ValidationError, ValidationErrorCode, ValidationResult,
    XptError,
};

// Re-export version
pub use version::XptVersion;

// Re-export core types
pub use types::{
    FormatSpec, InformatSpec, Justification, MissingValue, NumericValue, Observation,
    RowLengthError, XptColumn, XptDataset, XptLibrary, XptReaderOptions, XptType, XptValue,
    XptWriterOptions,
};

// Re-export reader functionality
pub use reader::{
    DatasetMeta, ObservationIter, StreamingReader, XptReader, read_xpt, read_xpt_streaming,
    read_xpt_streaming_with_options, read_xpt_with_options,
};

// Re-export writer functionality
pub use writer::{
    DatasetInfo, StreamingWriter, ValidatedWriter, XptWriter, XptWriterBuilder, write_xpt,
    write_xpt_with_options,
};

// Re-export Polars integration
#[cfg(feature = "polars")]
pub use polars::{read_xpt_to_dataframe, write_dataframe_to_xpt};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
