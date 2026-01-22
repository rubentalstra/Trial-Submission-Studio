//! CDISC submission preparation library.
//!
//! This crate provides all functionality needed for preparing data for FDA submission:
//!
//! - **Mapping** (`map`): Fuzzy column-to-variable mapping with confidence scores
//! - **Normalization** (`normalize`): Data transformation and standardization
//! - **Validation** (`validate`): CDISC conformance checking
//! - **Export** (`export`): Output generation (XPT, Dataset-XML, Define-XML)
//!
//! # Error Handling
//!
//! This crate uses a unified [`SubmitError`] type for all operations, built with `thiserror`.
//! Each submodule also provides more specific error types that can be converted to `SubmitError`.
//!
//! # Example
//!
//! ```ignore
//! use tss_submit::{MappingState, validate_domain, write_xpt_outputs};
//!
//! // 1. Map source columns to SDTM variables
//! let mut mapping = MappingState::new(domain, "STUDY01", &columns, hints, 0.6);
//!
//! // 2. Validate the mapped data
//! let report = validate_domain(&domain, &df, ct.as_ref());
//!
//! // 3. Export to XPT
//! write_xpt_outputs(&domain_data, output_dir)?;
//! ```

pub mod error;
pub mod export;
pub mod map;
pub mod normalize;
pub mod validate;

// Re-export unified error type
pub use error::{Result, SubmitError};

// Re-export commonly used types
pub use map::{
    ColumnScore, Mapping, MappingConfig, MappingError, MappingState, MappingSummary,
    ScoreComponent, ScoringEngine, Suggestion, VariableStatus,
};

pub use normalize::{
    NormalizationContext, NormalizationError, NormalizationPipeline, NormalizationRule,
    NormalizationType, build_preview_dataframe, build_preview_dataframe_with_dm,
    build_preview_dataframe_with_dm_and_omitted, build_preview_dataframe_with_omitted,
    execute_normalization, infer_normalization_rules,
};

pub use validate::{
    Category, Issue, Severity, ValidationReport, validate_domain,
    validate_domain_with_not_collected,
};

pub use export::{
    DatasetXmlOptions, DefineXmlOptions, DomainFrame, build_xpt_dataset_with_name,
    write_dataset_xml, write_dataset_xml_outputs, write_define_xml, write_xpt_outputs,
};
