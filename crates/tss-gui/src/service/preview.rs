//! Preview service - computes transformed DataFrame for preview tab.
//!
//! Uses `Task::perform` pattern for background computation.

use std::collections::BTreeMap;
use std::sync::Arc;

use polars::prelude::DataFrame;
use tss_standards::TerminologyRegistry;
use tss_submit::MappingState;
use tss_submit::build_preview_dataframe_with_omitted;

/// Error from preview computation.
#[derive(Debug, Clone)]
pub struct PreviewError(pub String);

impl std::fmt::Display for PreviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Input for preview computation.
///
/// Uses `Arc<DataFrame>` for efficient sharing of source data (#271).
#[derive(Clone)]
pub struct PreviewInput {
    /// Source DataFrame (Arc for cheap cloning).
    pub source_df: Arc<DataFrame>,
    /// Mapping state for extracting accepted mappings.
    pub mapping: MappingState,
    /// Optional CT registry for normalization.
    pub ct_registry: Option<TerminologyRegistry>,
}

/// Compute preview DataFrame asynchronously.
///
/// This function is designed to be used with `Task::perform`:
///
/// ```ignore
/// Task::perform(
///     compute_preview(input),
///     Message::PreviewReady,
/// )
/// ```
pub async fn compute_preview(input: PreviewInput) -> Result<DataFrame, PreviewError> {
    // Run the blocking computation in a separate thread
    tokio::task::spawn_blocking(move || compute_preview_sync(input))
        .await
        .map_err(|e| PreviewError(format!("Task panicked: {}", e)))?
}

/// Synchronous preview computation (runs on blocking thread).
fn compute_preview_sync(input: PreviewInput) -> Result<DataFrame, PreviewError> {
    let PreviewInput {
        source_df,
        mapping,
        ct_registry,
    } = input;

    // Extract accepted mappings as variable_name -> source_column
    let mappings: BTreeMap<String, String> = mapping
        .all_accepted()
        .iter()
        .map(|(var, (col, _confidence))| (var.clone(), col.clone()))
        .collect();

    // Get omitted variables
    let omitted = mapping.all_omitted();

    // Get domain definition and study ID
    let domain = mapping.domain();
    let study_id = mapping.study_id();

    // Build preview using normalization crate
    // Dereference Arc to pass reference to underlying DataFrame
    build_preview_dataframe_with_omitted(
        &source_df,
        &mappings,
        omitted,
        domain,
        study_id,
        ct_registry.as_ref(),
    )
    .map_err(|e: tss_submit::NormalizationError| PreviewError(e.to_string()))
}
