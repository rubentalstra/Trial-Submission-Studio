//! SDTM data transformation crate.
//!
//! Provides a data-driven, variable-level transformation system for SDTM domains.
//! All transformation logic is derived from Variable metadata - no hardcoded domain rules.
//!
//! # Overview
//!
//! This crate provides:
//! - **Pipeline inference**: Automatically derive transformation rules from SDTM Variable metadata
//! - **Transformation execution**: Apply transformations to produce SDTM-compliant DataFrames
//! - **Normalization functions**: ISO 8601 dates, durations, controlled terminology, etc.
//!
//! # Example
//!
//! ```ignore
//! use tss_transform::{build_pipeline_from_domain, execute_pipeline, TransformContext};
//!
//! // Load domain definition
//! let domain = load_sdtm_domain("AE");
//!
//! // Build pipeline from metadata
//! let pipeline = build_pipeline_from_domain(&domain);
//!
//! // Create execution context
//! let context = TransformContext::new("CDISC01", "AE")
//!     .with_mappings(mappings);
//!
//! // Execute transformations
//! let result_df = execute_pipeline(&source_df, &pipeline, &context)?;
//! ```
//!
//! # Design Principles
//!
//! - **Metadata-driven**: All transformation types inferred from Variable metadata
//! - **SDTM-compliant**: Follows SDTMIG v3.4 rules for dates, CT, sequences, etc.
//! - **Stateless functions**: Pure functions for easy testing and composition
//! - **Error preservation**: On transformation failure, preserve original value + log

mod error;
mod executor;
mod inference;
mod preview;
mod types;

pub mod normalization;

// Core types
pub use types::{DomainPipeline, TransformContext, TransformRule, TransformType};

// Error type
pub use error::TransformError;

// Pipeline building
pub use inference::build_pipeline_from_domain;

// Execution
pub use executor::execute_pipeline;

// Preview for validation
pub use preview::{
    build_preview_dataframe, build_preview_dataframe_with_dm,
    build_preview_dataframe_with_dm_and_omitted, build_preview_dataframe_with_omitted,
};
