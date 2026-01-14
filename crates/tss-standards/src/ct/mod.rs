//! Controlled Terminology (CT) types and loaders.
//!
//! This module provides:
//! - CT types: `Codelist`, `Term`, `TerminologyCatalog`, `TerminologyRegistry`
//! - CT loaders: `load()`, `load_from()`, `load_catalog()`, `load_sdtm_only()`
//! - Version management: `CtVersion`
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_standards::ct::{self, CtVersion};
//!
//! // Load all CT for a version
//! let registry = ct::load(CtVersion::default())?;
//!
//! // Validate a value against a codelist
//! if let Some(issue) = registry.validate_submission_value("C66742", "INVALID") {
//!     println!("Validation error: {}", issue);
//! }
//!
//! // Find submission value for a synonym (for normalization)
//! if let Some(value) = registry.find_submission_value("C66742", "YES") {
//!     println!("Normalized: {}", value); // "Y"
//! }
//! ```

pub mod loader;
pub mod types;

// Re-export types
pub use types::{
    Codelist, CtValidationIssue, ResolvedCodelist, Term, TerminologyCatalog, TerminologyRegistry,
};

// Re-export loader
pub use loader::{CtVersion, load, load_catalog, load_from, load_sdtm_only};
