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
//! use tss_validate::{validate_domain, Issue, Severity};
//!
//! // Validate a domain
//! let report = validate_domain(&domain, &df, ct_registry.as_ref());
//!
//! // Display issues
//! for issue in &report.issues {
//!     println!("[{:?}] {}: {}", issue.severity(), issue.category(), issue.message());
//! }
//! ```

mod checks;
mod issue;
mod report;
pub mod rules;
mod util;

use polars::prelude::DataFrame;
use std::collections::BTreeSet;
use tss_standards::SdtmDomain;
use tss_standards::TerminologyRegistry;

// Re-export public types
pub use checks::dates::is_date_variable;
pub use issue::{Issue, Severity};
pub use report::ValidationReport;
pub use rules::Category;
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
    domain: &SdtmDomain,
    df: &DataFrame,
    ct_registry: Option<&TerminologyRegistry>,
) -> ValidationReport {
    validate_domain_with_not_collected(domain, df, ct_registry, &BTreeSet::new())
}

/// Validate a single domain with support for "not collected" variables.
///
/// Variables in the `not_collected` set are exempt from ExpectedMissing warnings
/// because the user has explicitly acknowledged they were not collected.
///
/// # Arguments
/// * `domain` - SDTM domain definition
/// * `df` - DataFrame to validate
/// * `ct_registry` - Optional CT registry for terminology validation
/// * `not_collected` - Variables explicitly marked as "not collected" by user
pub fn validate_domain_with_not_collected(
    domain: &SdtmDomain,
    df: &DataFrame,
    ct_registry: Option<&TerminologyRegistry>,
    not_collected: &BTreeSet<String>,
) -> ValidationReport {
    checks::run_all(domain, df, ct_registry, not_collected)
}
