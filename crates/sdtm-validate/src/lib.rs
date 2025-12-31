//! SDTM validation and conformance checking.
//!
//! This crate provides comprehensive validation logic for SDTM datasets:
//!
//! - **Controlled Terminology (CT)**: Validates values against CT codelists
//! - **Required Variables**: Checks presence and population of Req variables
//! - **Expected Variables**: Warns about missing Exp variables
//! - **Data Type Validation**: Ensures Num columns contain numeric data
//! - **ISO 8601 Date Validation**: Validates date/datetime format compliance
//! - **Sequence Uniqueness**: Checks for duplicate --SEQ per subject
//! - **Text Length**: Validates character field lengths
//! - **Identifier Nulls**: Checks that ID variables have no nulls
//!
//! # Example
//!
//! ```ignore
//! use sdtm_validate::{validate_domain, load_default_rules, Issue, Severity};
//!
//! // Load rules once at startup
//! let rules = load_default_rules()?;
//!
//! // Validate a domain
//! let report = validate_domain(&domain, &df, ct_registry.as_ref());
//!
//! // Display issues with rule metadata
//! for issue in &report.issues {
//!     let severity = issue.severity(Some(&rules));
//!     let message = issue.message(Some(&rules));
//!     println!("[{:?}] {}: {}", severity, issue.rule_id(), message);
//! }
//! ```

mod checks;
mod issue;
mod report;
pub mod rules;
mod util;

use polars::prelude::DataFrame;
use sdtm_model::Domain;
use sdtm_model::ct::TerminologyRegistry;

// Re-export public types
pub use checks::dates::is_date_variable;
pub use issue::{Issue, Severity};
pub use report::ValidationReport;
pub use rules::{Category, LoadError, Rule, RuleRegistry, load_default_rules, load_rules};
pub use util::CaseInsensitiveSet;

/// Validate a single domain against SDTM conformance rules.
///
/// Runs all validation checks:
/// - Controlled terminology values
/// - Required variable presence and population
/// - Expected variable presence (warnings)
/// - Data type conformance
/// - ISO 8601 date format validation
/// - Unique sequence numbers per subject
/// - Text length limits
/// - Identifier null checks
pub fn validate_domain(
    domain: &Domain,
    df: &DataFrame,
    ct_registry: Option<&TerminologyRegistry>,
) -> ValidationReport {
    checks::run_all(domain, df, ct_registry)
}
