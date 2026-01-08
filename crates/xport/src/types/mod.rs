//! Core types for XPT file handling.
//!
//! This module provides the fundamental data structures for representing
//! XPT datasets, columns, values, and file handling options.
//!
//! # Type Overview
//!
//! - [`XptValue`]: A cell value (numeric or character)
//! - [`NumericValue`]: A numeric value that may be missing
//! - [`MissingValue`]: One of 28 SAS missing value codes
//! - [`XptColumn`]: Column/variable definition (metadata)
//! - [`XptDataset`]: Dataset with columns and rows
//! - [`XptLibrary`]: Container for multiple datasets
//! - [`Observation`]: Single row for streaming operations
//! - [`FormatSpec`]: Output format specification

mod column;
mod dataset;
mod format;
mod missing;
mod observation;
mod options;
mod value;

// Column types
pub use column::{Justification, XptColumn, XptType};

// Dataset types
pub use dataset::{RowLengthError, XptDataset, XptLibrary};

// Format types
pub use format::{FormatSpec, InformatSpec};

// Missing value types
pub use missing::MissingValue;

// Observation type for streaming
pub use observation::Observation;

// Reader/Writer options
pub use options::{XptReaderOptions, XptWriterOptions};

// Re-export XptVersion from the version module for backward compatibility
pub use crate::version::XptVersion;

// Value types
pub use value::{NumericValue, XptValue};
