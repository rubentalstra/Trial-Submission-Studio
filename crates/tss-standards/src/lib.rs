//! CDISC standards types, loaders, and utilities.
//!
//! This crate provides:
//!
//! - **Type definitions** for SDTM, ADaM, and SEND standards
//! - **Standards loaders** from offline CSV files
//! - **Controlled Terminology** (CT) support with validation
//! - **Polars utilities** for data manipulation
//!
//! # Module Organization
//!
//! - [`traits`]: Core traits and shared types (Standard, VariableType, CoreDesignation)
//! - [`sdtm`]: SDTM domains and variables per SDTMIG v3.4
//! - [`adam`]: ADaM datasets and variables per ADaMIG v1.3
//! - [`send`]: SEND domains and variables per SENDIG v3.1.1
//! - [`ct`]: Controlled Terminology types and loaders
//! - [`polars`]: Polars AnyValue utility functions
//! - [`registry`]: Unified standards registry
//!
//! # Standards Directory Structure
//!
//! ```text
//! standards/
//! ├── Terminology/             # Controlled Terminology by version
//! │   ├── 2024-03-29/          # CT version (default)
//! │   └── 2025-09-26/          # CT version (latest)
//! ├── sdtm/ig/v3.4/            # SDTM-IG v3.4
//! │   ├── Datasets.csv
//! │   └── Variables.csv
//! ├── adam/ig/v1.3/            # ADaM-IG v1.3
//! │   ├── DataStructures.csv
//! │   └── Variables.csv
//! └── send/ig/v3.1.1/          # SEND-IG v3.1.1
//!     ├── Datasets.csv
//!     └── Variables.csv
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use tss_standards::{StandardsRegistry, StandardsConfig, CdiscDomain};
//!
//! // Load all standards
//! let registry = StandardsRegistry::load_all()?;
//!
//! // Access domains
//! let ae = registry.find_sdtm_domain("AE").unwrap();
//! println!("AE has {} variables", ae.variables().len());
//!
//! // Validate against CT
//! if let Some(issue) = registry.ct.validate_submission_value("C66742", "INVALID") {
//!     println!("CT Error: {}", issue);
//! }
//! ```

use serde::{Deserialize, Serialize};

// Core modules - types
pub mod adam;
pub mod ct;
pub mod polars;
pub mod sdtm;
pub mod send;
pub mod traits;

// Loader modules
pub mod adam_ig;
pub mod error;
pub mod paths;
pub mod registry;
pub mod sdtm_ig;
pub mod send_ig;

// ============================================================================
// Re-exports for convenience
// ============================================================================

// Error types
pub use error::{Result, StandardsError};

// Path utilities
pub use paths::{STANDARDS_ENV_VAR, standards_root};

// Registry
pub use registry::{StandardsConfig, StandardsRegistry};

// CT types and loader
pub use ct::{
    Codelist, CtValidationIssue, CtVersion, ResolvedCodelist, Term, TerminologyCatalog,
    TerminologyRegistry,
};

// Convenience re-exports for loaders
pub use adam_ig::load as load_adam_ig;
pub use ct::load as load_ct;
pub use sdtm_ig::load as load_sdtm_ig;
pub use send_ig::load as load_send_ig;

// Core traits and types
pub use traits::{CdiscDomain, CdiscVariable, CoreDesignation, Standard, VariableType};

// SDTM types
pub use sdtm::{DatasetClass, SdtmDomain, SdtmVariable, VariableRole};

// ADaM types
pub use adam::{AdamDataset, AdamDatasetType, AdamVariable, AdamVariableSource};

// SEND types
pub use send::{SendDatasetClass, SendDomain, SendStudyType, SendVariable};

// Polars utilities
pub use polars::{
    any_to_f64, any_to_i64, any_to_string, any_to_string_non_empty, format_numeric, parse_f64,
    parse_i64,
};

// ============================================================================
// Additional types
// ============================================================================

/// Hints about a source column's characteristics.
///
/// Used to improve mapping/scoring accuracy based on column metadata.
/// This type is analyzed from source data during ingestion and used
/// by the mapping engine to make better suggestions.
///
/// # Example
///
/// ```
/// use tss_standards::ColumnHint;
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
