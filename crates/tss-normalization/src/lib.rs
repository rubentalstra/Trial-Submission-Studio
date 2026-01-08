//! SDTM data normalization crate.
//!
//! Provides a data-driven, variable-level normalization system for SDTM domains.
//! All normalization logic is derived from Variable metadata - no hardcoded domain rules.
//!
//! # Overview
//!
//! This crate provides:
//! - **Pipeline inference**: Automatically derive normalization rules from SDTM Variable metadata
//! - **Normalization execution**: Apply normalizations to produce SDTM-compliant DataFrames
//! - **Normalization functions**: ISO 8601 dates, durations, controlled terminology, etc.
//!
//! # Example
//!
//! ```ignore
//! use tss_normalization::{infer_normalization_rules, execute_normalization, NormalizationContext};
//!
//! // Load domain definition
//! let domain = load_sdtm_domain("AE");
//!
//! // Build pipeline from metadata
//! let pipeline = infer_normalization_rules(&amp;domain);
//!
//! // Create execution context
//! let context = NormalizationContext::new("CDISC01", "AE")
//!     .with_mappings(mappings);
//!
//! // Execute normalizations
//! let result_df = execute_normalization(&amp;source_df, &amp;pipeline, &amp;context)?;
//! ```
//!
//! # Design Principles
//!
//! - **Metadata-driven**: All normalization types inferred from Variable metadata
//! - **SDTM-compliant**: Follows SDTMIG v3.4 rules for dates, CT, sequences, etc.
//! - **Stateless functions**: Pure functions for easy testing and composition
//! - **Error preservation**: On normalization failure, preserve original value + log

mod error;
mod executor;
mod inference;
mod preview;
mod types;

pub mod normalization;

// Core types
pub use types::{NormalizationContext, NormalizationPipeline, NormalizationRule, NormalizationType};

// Error type
pub use error::NormalizationError;

// Pipeline building
pub use inference::infer_normalization_rules;

// Execution
pub use executor::execute_normalization;

// Preview for validation
pub use preview::{
    build_preview_dataframe, build_preview_dataframe_with_dm,
    build_preview_dataframe_with_dm_and_omitted, build_preview_dataframe_with_omitted,
};
