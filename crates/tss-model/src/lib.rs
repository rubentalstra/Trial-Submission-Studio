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

pub mod adam;
pub mod ct;
pub mod polars;
pub mod sdtm;
pub mod send;
pub mod traits;

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
