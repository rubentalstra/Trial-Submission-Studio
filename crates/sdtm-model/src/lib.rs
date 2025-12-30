//! SDTM data model types and structures.
//!
//! This crate provides the core data model for SDTM (Study Data Tabulation Model)
//! processing, including domain definitions, variables, controlled terminology,
//! validation, and processing request/response types.
//!
//! # SDTMIG Reference
//!
//! All types in this crate are designed to align with SDTMIG v3.4 specifications.
//!
//! # Modules
//!
//! - [`conformance`] - Validation severity levels and reports
//! - [`ct`] - Controlled Terminology model (codelists, terms, catalogs)
//! - [`domain`] - Domain and variable definitions
//! - [`error`] - Error types for SDTM processing
//! - [`lookup`] - Utility types for case-insensitive lookups
//! - [`mapping`] - Column mapping suggestions and configuration
//! - [`p21`] - Pinnacle 21 validation rule types
//! - [`processing`] - Request/response types for study processing

pub mod conformance;
pub mod ct;
pub mod domain;
pub mod error;
pub mod lookup;
pub mod mapping;
pub mod p21;
pub mod processing;

pub use conformance::{CheckType, Severity, ValidationIssue, ValidationReport};
pub use ct::{Codelist, ResolvedCodelist, Term, TerminologyCatalog, TerminologyRegistry};
pub use domain::{DatasetClass, Domain, Variable, VariableType};
pub use p21::{P21Category, P21Rule, P21RuleRegistry, P21Severity};
pub use error::{Result, SdtmError};
pub use lookup::CaseInsensitiveSet;
pub use mapping::{ColumnHint, MappingConfig, MappingSuggestion};
pub use processing::{
    DomainResult, OutputFormat, OutputPaths, ProcessStudyRequest, ProcessStudyResponse, StudyError,
};
