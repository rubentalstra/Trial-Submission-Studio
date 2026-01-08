//! CDISC data model types for SDTM, ADaM, and SEND standards.
//!
//! This crate provides type-safe representations of CDISC foundational standards:
//! - **SDTM**: Study Data Tabulation Model (clinical trial tabulation)
//! - **ADaM**: Analysis Data Model (analysis-ready datasets)
//! - **SEND**: Standard for Exchange of Nonclinical Data (animal studies)
//!
//! # Module Organization
//!
//! - [`traits`]: Common types shared across standards (Standard, DataType, CoreDesignation)
//! - [`sdtm`]: SDTM domains and variables per SDTMIG v3.4
//! - [`adam`]: ADaM datasets and variables per ADaMIG v1.3
//! - [`send`]: SEND domains and variables per SENDIG v3.1.1
//! - [`ct`]: Controlled Terminology types
//! - [`polars`]: Polars AnyValue utility functions

use serde::{Deserialize, Serialize};

pub mod adam;
pub mod ct;
pub mod polars;
pub mod sdtm;
pub mod send;
pub mod traits;

/// Hints about a source column's characteristics.
///
/// Used to improve mapping/scoring accuracy based on column metadata.
/// This type is analyzed from source data during ingestion and used
/// by the mapping engine to make better suggestions.
///
/// # Example
///
/// ```
/// use tss_model::ColumnHint;
///
/// let hint = ColumnHint {
///     is_numeric: true,
///     unique_ratio: 0.95,
///     null_ratio: 0.02,
///     label: Some("Patient Age".to_string()),
/// };
///
/// assert!(hint.is_numeric);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColumnHint {
    /// Whether the column contains numeric data.
    pub is_numeric: bool,
    /// Ratio of unique values (0.0 to 1.0).
    pub unique_ratio: f64,
    /// Ratio of null/missing values (0.0 to 1.0).
    pub null_ratio: f64,
    /// Optional column label from source metadata.
    pub label: Option<String>,
}

// Re-export CT types
pub use ct::{Codelist, ResolvedCodelist, Term, TerminologyCatalog, TerminologyRegistry};

// Re-export common traits and types
pub use traits::{CoreDesignation, DataType, Standard};

// Re-export SDTM types at root for backward compatibility
pub use sdtm::{DatasetClass, Domain, Variable, VariableRole, VariableType};

// Re-export ADaM types
pub use adam::{AdamDataset, AdamDatasetType, AdamVariable, AdamVariableSource};

// Re-export SEND types
pub use send::{SendDatasetClass, SendDomain, SendStudyType, SendVariable};

// Re-export Polars utility functions at crate root for convenience
pub use polars::{
    any_to_f64, any_to_i64, any_to_string, any_to_string_non_empty, format_numeric, parse_f64,
    parse_i64,
};
