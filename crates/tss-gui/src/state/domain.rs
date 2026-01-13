//! Domain state - source data and mapping.
//!
//! A domain represents a single SDTM dataset (e.g., DM, AE, LB).
//! This module contains:
//! - [`DomainSource`] - Immutable source data (CSV file + DataFrame)
//! - [`Domain`] - Source + mapping state + normalization pipeline

use polars::prelude::{DataFrame, PlSmallStr};
use std::path::PathBuf;
use tss_map::MappingState;
use tss_normalization::{NormalizationPipeline, infer_normalization_rules};

// =============================================================================
// DOMAIN SOURCE (Immutable)
// =============================================================================

/// Immutable source data for a domain.
///
/// Contains the original CSV file path and loaded DataFrame.
/// This data never changes once loaded.
#[derive(Debug, Clone)]
pub struct DomainSource {
    /// Path to the source CSV file.
    pub file_path: PathBuf,

    /// Source DataFrame (loaded once, never mutated).
    pub data: DataFrame,

    /// Human-readable label (e.g., "Demographics" for DM).
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

    /// Get column names from the source data.
    pub fn column_names(&self) -> Vec<String> {
        self.data
            .get_column_names()
            .into_iter()
            .map(PlSmallStr::to_string)
            .collect()
    }

    /// Get row count.
    #[inline]
    pub fn row_count(&self) -> usize {
        self.data.height()
    }

    /// Get column count.
    #[inline]
    pub fn column_count(&self) -> usize {
        self.data.width()
    }

    /// Get file name without path.
    pub fn file_name(&self) -> Option<&str> {
        self.file_path.file_name().and_then(|n| n.to_str())
    }
}

// =============================================================================
// DOMAIN (Source + Mapping)
// =============================================================================

/// A single SDTM domain with source data and mapping state.
///
/// # Design Notes
///
/// - **Normalization pipeline is computed once** - The pipeline is derived from
///   the SDTM domain metadata when the domain is created. It defines what
///   transformations will be applied to each variable during export.
///
/// - **Preview, validation, and transform results** are computed on demand,
///   not stored here. This keeps state simple and avoids cache invalidation.
///
/// - **Mapping state is from `tss_map`** - The core mapping logic lives in the
///   `tss_map` crate. This struct just holds the state.
///
/// # Example
///
/// ```ignore
/// let domain = Domain::new(source, mapping);
///
/// // Check mapping status
/// let summary = domain.mapping.summary();
/// println!("Mapped: {}/{}", summary.mapped, summary.total_variables);
///
/// // Access normalization rules
/// for rule in &domain.normalization.rules {
///     println!("{}: {:?}", rule.target_variable, rule.transform_type);
/// }
/// ```
#[derive(Clone)]
pub struct Domain {
    /// Immutable source data (CSV).
    pub source: DomainSource,

    /// Mapping state from `tss_map` crate.
    pub mapping: MappingState,

    /// Normalization pipeline (derived from SDTM domain metadata).
    /// Computed once when domain is created, defines transformations for export.
    pub normalization: NormalizationPipeline,
}

impl Domain {
    /// Create a new domain.
    ///
    /// Automatically infers the normalization pipeline from the SDTM domain
    /// metadata. This pipeline defines what transformations will be applied
    /// to each variable during export.
    pub fn new(source: DomainSource, mapping: MappingState) -> Self {
        // Infer normalization rules from the SDTM domain metadata
        let normalization = infer_normalization_rules(mapping.domain());

        Self {
            source,
            mapping,
            normalization,
        }
    }

    /// Get display name: "DM (Demographics)" or just "DM".
    pub fn display_name(&self, code: &str) -> String {
        match &self.source.label {
            Some(label) => format!("{} ({})", code, label),
            None => code.to_string(),
        }
    }

    /// Get row count from source data.
    #[inline]
    pub fn row_count(&self) -> usize {
        self.source.row_count()
    }

    /// Get column names from source data.
    #[inline]
    pub fn column_names(&self) -> Vec<String> {
        self.source.column_names()
    }

    /// Get mapping summary.
    #[inline]
    pub fn summary(&self) -> tss_map::MappingSummary {
        self.mapping.summary()
    }

    /// Check if all required/expected variables are mapped.
    pub fn is_mapping_complete(&self) -> bool {
        let summary = self.summary();
        summary.required_mapped == summary.required_total
    }

    /// Check if user has made any mapping changes.
    ///
    /// A domain is "touched" if any variable has been accepted,
    /// marked not collected, or marked omitted.
    pub fn is_touched(&self) -> bool {
        let summary = self.summary();
        summary.mapped > 0 || summary.not_collected > 0 || summary.omitted > 0
    }

    /// Get unmapped source columns (for SUPP configuration).
    ///
    /// Returns columns that are not mapped to any SDTM variable.
    pub fn unmapped_columns(&self) -> Vec<String> {
        let mapped_columns: std::collections::BTreeSet<_> = self
            .mapping
            .all_accepted()
            .values()
            .map(|(col, _)| col.as_str())
            .collect();

        self.source
            .column_names()
            .into_iter()
            .filter(|col| !mapped_columns.contains(col.as_str()))
            .collect()
    }
}

impl std::fmt::Debug for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Domain")
            .field("source", &self.source.file_path)
            .field("rows", &self.source.row_count())
            .field("mapping_summary", &self.mapping.summary())
            .finish_non_exhaustive()
    }
}
