//! SDTM data transformation utilities.
//!
//! This crate provides a **data-driven transformation pipeline** for SDTM processing.
//!
//! # Architecture
//!
//! - **pipeline**: Core types for transformation rules and pipelines
//! - **context**: Runtime context for executing transformations
//! - **executors**: Individual transformation executor functions
//! - **normalization**: CT and datetime normalization utilities
//! - **frame**: Domain frame types for SDTM datasets
//! - **data_utils**: DataFrame value extraction and SDTM identifier sanitization
//! - **domain_sets**: Domain collection utilities for lookup and reporting
//!
//! # Key Principle
//!
//! **No hardcoded domain-specific rules.** All transformation types are inferred from:
//! - Variable name patterns (`STUDYID`, `DOMAIN`, `*SEQ`, `*DY`, `*DTC`)
//! - `described_value_domain` field (ISO 8601 formats)
//! - `codelist_code` field (CT normalization)
//! - `data_type` + `role` combination
//!
//! # Example
//!
//! ```ignore
//! use sdtm_transform::pipeline::{DomainPipeline, TransformType, TransformRule};
//! use sdtm_transform::context::TransformContext;
//!
//! // Build pipeline from domain metadata
//! let mut pipeline = DomainPipeline::new("AE");
//!
//! // Add transformation rules (typically auto-generated from Variable metadata)
//! pipeline.add_rule(TransformRule::derived("STUDYID", TransformType::Constant, 1));
//! pipeline.add_rule(TransformRule::derived("DOMAIN", TransformType::Constant, 2));
//! pipeline.add_rule(TransformRule::derived("USUBJID", TransformType::UsubjidPrefix, 3));
//! ```

pub mod context;
pub mod data_utils;
pub mod domain_sets;
pub mod executors;
pub mod frame;
pub mod inference;
pub mod normalization;
pub mod pipeline;

// Re-export datetime module at top level for backwards compatibility
pub use normalization::datetime;

// Re-export common types
pub use context::{CtResolutionMode, TransformContext, TransformResult};
pub use frame::{DomainFrame, DomainFrameMeta};
pub use inference::{TransformSummary, build_pipeline_from_domain, infer_transform_type};
pub use pipeline::{DomainPipeline, TransformOrigin, TransformRule, TransformType};

// Re-export common functions
pub use data_utils::{sanitize_qnam, sanitize_test_code, strip_all_quotes, strip_quotes};
pub use domain_sets::domain_map_by_code;
pub use executors::{
    apply_constant, apply_iso8601_date, apply_iso8601_datetime, apply_numeric_conversion,
    apply_usubjid_prefix, assign_sequence_numbers, build_preview_dataframe, build_simple_preview,
    calculate_study_day, copy_column, get_ct_columns, normalize_ct_column,
};
