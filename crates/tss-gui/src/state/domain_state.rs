//! Domain state management.
//!
//! This module contains `DomainSource` (immutable source data) and
//! `DomainState` (mutable mapping with version tracking).

use super::DerivedState;
use polars::prelude::DataFrame;
use std::path::PathBuf;
use tss_map::MappingState as CoreMappingState;

// ============================================================================
// Domain Source (Immutable)
// ============================================================================

/// Immutable source data for a domain.
///
/// This contains the original CSV data and file path. Once loaded,
/// this data never changes during the session.
#[derive(Debug, Clone)]
pub struct DomainSource {
    /// Path to the source CSV file
    pub file_path: PathBuf,
    /// Source DataFrame (loaded once, never mutated)
    pub data: DataFrame,
    /// Human-readable domain label (e.g., "Demographics" for DM)
    pub label: Option<String>,
}

impl DomainSource {
    /// Create a new domain source.
    pub fn new(file_path: PathBuf, data: DataFrame, label: Option<String>) -> Self {
        Self {
            file_path,
            data,
            label,
        }
    }

    /// Get source column names.
    pub fn columns(&self) -> Vec<String> {
        self.data
            .get_column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get row count.
    pub fn row_count(&self) -> usize {
        self.data.height()
    }

    /// Get the file name without path.
    pub fn file_name(&self) -> Option<&str> {
        self.file_path.file_name().and_then(|n| n.to_str())
    }
}

// ============================================================================
// Domain State (Mutable with Version Tracking)
// ============================================================================

/// State for a single SDTM domain.
///
/// This contains the mapping state from `tss_map` crate, plus version
/// tracking for cache invalidation. The `derived` field contains cached
/// computed data (transform, preview, validation) that automatically
/// invalidates when `version` changes.
///
/// # Version Tracking
///
/// The `version` field is incremented whenever the mapping state is mutated.
/// Derived state stores the version it was computed from, enabling automatic
/// staleness detection.
///
/// # Usage
///
/// Always use `with_mapping()` to mutate the mapping state - this ensures
/// the version is incremented and derived state will be rebuilt.
///
/// ```ignore
/// domain.with_mapping(|m| {
///     m.accept_suggestion("USUBJID");
/// });
/// ```
#[derive(Clone)]
pub struct DomainState {
    /// Immutable source data (CSV)
    pub source: DomainSource,
    /// Core mapping state from tss_map crate
    pub mapping: CoreMappingState,
    /// Version counter - incremented on any mapping mutation
    pub version: u64,
    /// Cached derived state (transform, preview, validation, supp)
    pub derived: DerivedState,
    /// Whether the user has opened/interacted with this domain
    user_touched: bool,
}

impl DomainState {
    /// Create a new domain state.
    pub fn new(source: DomainSource, mapping: CoreMappingState) -> Self {
        Self {
            source,
            mapping,
            version: 0,
            derived: DerivedState::default(),
            user_touched: false,
        }
    }

    /// Check if the user has made mapping changes to this domain.
    pub fn is_touched(&self) -> bool {
        self.user_touched
    }

    /// Mutate the mapping state and auto-increment version.
    ///
    /// This is the ONLY way to mutate the mapping state. It ensures
    /// the version is always incremented, causing derived state to
    /// be considered stale and rebuilt on next access.
    ///
    /// Also marks the domain as touched (user has made changes).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = domain.with_mapping(|m| {
    ///     m.accept_manual("USUBJID", "SUBJECT_ID")
    /// });
    /// ```
    pub fn with_mapping<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut CoreMappingState) -> R,
    {
        let result = f(&mut self.mapping);
        self.version += 1;
        self.user_touched = true; // Mark as touched when any mapping change is made
        result
    }

    /// Get display name: "DM (Demographics)" or just "DM" if no label.
    pub fn display_name(&self, code: &str) -> String {
        match &self.source.label {
            Some(label) => format!("{} ({})", code, label),
            None => code.to_string(),
        }
    }

    /// Get row count from source data.
    pub fn row_count(&self) -> usize {
        self.source.row_count()
    }

    /// Get mapping summary.
    pub fn summary(&self) -> tss_map::MappingSummary {
        self.mapping.summary()
    }

    /// Check if mapping is complete (all Required/Expected resolved).
    pub fn is_mapping_complete(&self) -> bool {
        let summary = self.summary();
        summary.required_mapped == summary.required_total
    }
}

impl std::fmt::Debug for DomainState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomainState")
            .field("source", &self.source.file_path)
            .field("version", &self.version)
            .field("mapping_summary", &self.mapping.summary())
            .finish_non_exhaustive()
    }
}
