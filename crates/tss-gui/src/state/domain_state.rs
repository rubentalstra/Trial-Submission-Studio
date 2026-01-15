//! Domain state - source data and mapping.
//!
//! A domain represents a single SDTM dataset (e.g., DM, AE, LB).
//! This module contains:
//! - [`DomainSource`] - Immutable source data (CSV file + DataFrame)
//! - [`Domain`] - Source + mapping state + normalization pipeline
//! - [`SuppColumnConfig`] - SUPP qualifier configuration for unmapped columns

use polars::prelude::{DataFrame, PlSmallStr};
use std::collections::HashMap;
use std::path::PathBuf;
use tss_submit::MappingState;
use tss_submit::{NormalizationPipeline, Severity, ValidationReport, infer_normalization_rules};

// =============================================================================
// SUPP CONFIGURATION
// =============================================================================

/// Configuration for a column to be included in SUPP domain.
#[derive(Debug, Clone)]
pub struct SuppColumnConfig {
    /// Source column name.
    pub column: String,
    /// QNAM - Qualifier Variable Name (max 8 chars, uppercase).
    pub qnam: String,
    /// QLABEL - Qualifier Variable Label (max 40 chars).
    pub qlabel: String,
    /// QORIG - Origin of the data.
    pub qorig: SuppOrigin,
    /// QEVAL - Evaluator (optional).
    pub qeval: Option<String>,
    /// Action: whether to include in SUPP or skip.
    pub action: SuppAction,
}

impl SuppColumnConfig {
    /// Create a new SUPP config with default values derived from column name.
    pub fn from_column(column: &str) -> Self {
        // Auto-generate QNAM from column name (max 8 chars, uppercase)
        let qnam = column
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(8)
            .collect::<String>()
            .to_uppercase();

        Self {
            column: column.to_string(),
            qnam,
            qlabel: String::new(), // User must provide meaningful description
            qorig: SuppOrigin::Crf,
            qeval: None,
            action: SuppAction::Pending,
        }
    }

    /// Check if the configuration is valid for export.
    pub fn is_valid(&self) -> bool {
        !self.qnam.is_empty()
            && self.qnam.len() <= 8
            && !self.qlabel.is_empty()
            && self.qlabel.len() <= 40
    }

    /// Check if this column should be included in SUPP output.
    pub fn should_include(&self) -> bool {
        self.action == SuppAction::Include && self.is_valid()
    }
}

/// Origin of SUPP qualifier data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SuppOrigin {
    /// Data from Case Report Form.
    #[default]
    Crf,
    /// Derived from other data.
    Derived,
    /// Sponsor-assigned value.
    Assigned,
}

impl SuppOrigin {
    /// Get CDISC code for this origin.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Crf => "CRF",
            Self::Derived => "DERIVED",
            Self::Assigned => "ASSIGNED",
        }
    }

    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Crf => "Case Report Form",
            Self::Derived => "Derived",
            Self::Assigned => "Assigned",
        }
    }

    /// All values.
    pub const ALL: [SuppOrigin; 3] = [Self::Crf, Self::Derived, Self::Assigned];
}

/// Action for a SUPP column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SuppAction {
    /// Column is pending review.
    #[default]
    Pending,
    /// Include in SUPP domain.
    Include,
    /// Skip this column (don't include in output).
    Skip,
}

impl SuppAction {
    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Include => "Add to SUPP",
            Self::Skip => "Skip",
        }
    }

    /// All values.
    pub const ALL: [SuppAction; 3] = [Self::Pending, Self::Include, Self::Skip];
}

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
/// - **Validation cache persists across navigation** - Stored here so it survives
///   view changes. Use `validation_summary()` for quick stats.
///
/// - **Mapping state is from `tss_map`** - The core mapping logic lives in the
///   `tss_map` crate. This struct just holds the state.
///
/// # Example
///
/// ```ignore
/// let domain = DomainState::new(source, mapping);
///
/// // Check mapping status
/// let summary = domain.mapping.summary();
/// println!("Mapped: {}/{}", summary.mapped, summary.total_variables);
///
/// // Check validation (if run)
/// if let Some((warnings, errors)) = domain.validation_summary() {
///     println!("Issues: {} warnings, {} errors", warnings, errors);
/// }
/// ```
#[derive(Clone)]
pub struct DomainState {
    /// Immutable source data (CSV).
    pub source: DomainSource,

    /// Mapping state from `tss_map` crate.
    pub mapping: MappingState,

    /// Normalization pipeline (derived from SDTM domain metadata).
    /// Computed once when domain is created, defines transformations for export.
    pub normalization: NormalizationPipeline,

    /// SUPP configuration for unmapped columns.
    /// Key is the source column name.
    pub supp_config: HashMap<String, SuppColumnConfig>,

    /// Cached validation report.
    /// Stored at domain level so it persists across view navigation.
    /// None = validation not yet run, Some = cached results.
    pub validation_cache: Option<ValidationReport>,
}

impl DomainState {
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
            supp_config: HashMap::new(),
            validation_cache: None,
        }
    }

    /// Get validation summary as (warnings, errors) count.
    ///
    /// Returns `None` if validation hasn't been run yet.
    /// Returns `Some((warnings, errors))` from cached validation report.
    pub fn validation_summary(&self) -> Option<(usize, usize)> {
        self.validation_cache.as_ref().map(|report| {
            let warnings = report
                .issues
                .iter()
                .filter(|i| matches!(i.severity(), Severity::Warning))
                .count();
            let errors = report
                .issues
                .iter()
                .filter(|i| matches!(i.severity(), Severity::Error | Severity::Reject))
                .count();
            (warnings, errors)
        })
    }

    /// Clear validation cache (call when mapping/normalization changes).
    pub fn invalidate_validation(&mut self) {
        self.validation_cache = None;
    }

    /// Get display name: "Demographics" or fallback to code "DM".
    pub fn display_name(&self, code: &str) -> String {
        match &self.source.label {
            Some(label) => label.to_string(),
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
    pub fn summary(&self) -> tss_submit::MappingSummary {
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

impl std::fmt::Debug for DomainState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomainState")
            .field("source", &self.source.file_path)
            .field("rows", &self.source.row_count())
            .field("mapping_summary", &self.mapping.summary())
            .finish_non_exhaustive()
    }
}
