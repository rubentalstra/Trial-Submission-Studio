//! Validation service - runs CDISC conformance validation for a domain.
//!
//! Uses `Task::perform` pattern for background computation.

use std::collections::BTreeSet;

use polars::prelude::DataFrame;
use tss_model::Domain;
use tss_model::TerminologyRegistry;
use tss_validate::ValidationReport;

/// Input for validation computation.
#[derive(Clone)]
pub struct ValidationInput {
    /// SDTM domain definition.
    pub domain: Domain,
    /// Transformed DataFrame to validate.
    pub df: DataFrame,
    /// Optional CT registry for terminology validation.
    pub ct_registry: Option<TerminologyRegistry>,
    /// Variables marked as "not collected".
    pub not_collected: BTreeSet<String>,
}

/// Compute validation asynchronously.
///
/// This function is designed to be used with `Task::perform`:
///
/// ```ignore
/// Task::perform(
///     compute_validation(input),
///     Message::ValidationComplete,
/// )
/// ```
pub async fn compute_validation(input: ValidationInput) -> ValidationReport {
    // Run the blocking computation in a separate thread
    tokio::task::spawn_blocking(move || compute_validation_sync(input))
        .await
        .unwrap_or_else(|e| {
            // Create an empty report on panic
            tracing::error!("Validation task panicked: {}", e);
            ValidationReport::new("UNKNOWN")
        })
}

/// Synchronous validation computation (runs on blocking thread).
fn compute_validation_sync(input: ValidationInput) -> ValidationReport {
    let ValidationInput {
        domain,
        df,
        ct_registry,
        not_collected,
    } = input;

    tss_validate::validate_domain_with_not_collected(
        &domain,
        &df,
        ct_registry.as_ref(),
        &not_collected,
    )
}
