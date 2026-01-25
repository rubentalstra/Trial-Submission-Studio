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
mod column_reader;
mod issue;
mod report;
pub mod rules;
mod util;

use polars::prelude::DataFrame;
use std::collections::{BTreeSet, HashSet};
use tss_standards::SdtmDomain;
use tss_standards::TerminologyRegistry;

// Re-export public types
pub use checks::dates::is_date_variable;
pub use column_reader::ColumnReader;
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

/// Validate cross-domain references across all domains.
///
/// Checks that:
/// - All USUBJIDs in non-DM domains exist in the DM domain
/// - RDOMAIN values in CO/RELREC reference valid domains
/// - RELSUB RSUBJID exists in DM and relationships are bidirectional
/// - RELSPEC PARENT references valid REFID within subject
/// - RELREC references point to existing records
///
/// # Arguments
/// * `domains` - List of (domain_name, DataFrame) pairs
///
/// # Returns
/// A vector of (domain_name, issues) pairs for domains with issues.
pub fn validate_cross_domain(domains: &[(&str, &DataFrame)]) -> Vec<(String, Vec<Issue>)> {
    // Find the DM domain
    let dm_df = domains
        .iter()
        .find(|(name, _)| *name == "DM")
        .map(|(_, df)| *df);

    let Some(dm_df) = dm_df else {
        // No DM domain - can't validate cross-domain references
        tracing::debug!("No DM domain found - skipping cross-domain validation");
        return vec![];
    };

    // Extract valid USUBJIDs from DM
    let dm_subjects = checks::cross_domain::extract_dm_subjects(dm_df);

    if dm_subjects.is_empty() {
        tracing::warn!(
            "DM domain has no USUBJIDs - cross-domain validation may report false positives"
        );
    }

    // Build set of valid domain codes for RDOMAIN validation
    let valid_domains: HashSet<String> = domains
        .iter()
        .map(|(name, _)| name.to_uppercase())
        .collect();

    let mut results = Vec::new();

    // Check each domain
    for (name, df) in domains {
        let name_upper = name.to_uppercase();
        let mut domain_issues = Vec::new();

        // Skip DM for USUBJID check (it's the reference)
        if name_upper != "DM" {
            domain_issues.extend(checks::cross_domain::check_usubjid_in_dm(
                name,
                df,
                &dm_subjects,
            ));
        }

        // RDOMAIN validation for CO and RELREC
        if name_upper == "CO" || name_upper == "RELREC" {
            domain_issues.extend(checks::cross_domain::check_rdomain_valid(
                name,
                df,
                &valid_domains,
            ));
        }

        // RELSUB-specific validation
        if name_upper == "RELSUB" {
            domain_issues.extend(checks::cross_domain::check_relsub(df, &dm_subjects));
        }

        // RELSPEC-specific validation
        if name_upper == "RELSPEC" {
            domain_issues.extend(checks::cross_domain::check_relspec(df));
        }

        // RELREC-specific validation (record references)
        if name_upper == "RELREC" {
            let context = checks::cross_domain::RelrecContext::new(domains);
            domain_issues.extend(checks::cross_domain::check_relrec(df, &context));
        }

        if !domain_issues.is_empty() {
            results.push((name.to_string(), domain_issues));
        }
    }

    results
}
