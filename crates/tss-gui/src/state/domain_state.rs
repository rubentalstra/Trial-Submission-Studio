//! Domain state - source-mapped and generated domains.
//!
//! A domain represents a single SDTM dataset (e.g., DM, AE, LB, CO, RELREC).
//! This module contains:
//! - [`DomainState`] - Enum with Source and Generated variants
//! - [`SourceDomainState`] - Domain mapped from CSV source data
//! - [`GeneratedDomainState`] - Domain generated via UI (CO, RELREC, RELSPEC, RELSUB)
//! - [`DomainSource`] - Immutable source data (CSV file + DataFrame)
//! - [`SuppColumnConfig`] - SUPP qualifier configuration for unmapped columns

use polars::prelude::{DataFrame, PlSmallStr};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tss_standards::SdtmDomain;
use tss_submit::MappingState;
use tss_submit::{NormalizationPipeline, Severity, ValidationReport, infer_normalization_rules};

use super::relationship::{GeneratedDomainEntry, GeneratedDomainType};

// =============================================================================
// SUPP CONFIGURATION
// =============================================================================

/// Configuration for a column to be included in SUPP domain.
#[derive(Debug, Clone)]
pub struct SuppColumnConfig {
    /// Source column name.
    /// Note: Redundant with HashMap key, but kept for struct completeness.
    #[allow(dead_code)]
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

// =============================================================================
// DOMAIN SOURCE (Immutable)
// =============================================================================

/// Immutable source data for a domain.
///
/// Contains the original CSV file path and loaded DataFrame.
/// This data never changes once loaded.
///
/// Uses `Arc<DataFrame>` for efficient cloning - the underlying
/// DataFrame is shared across clones rather than deep-copied (#271).
#[derive(Debug, Clone)]
pub struct DomainSource {
    /// Path to the source CSV file.
    pub file_path: PathBuf,

    /// Source DataFrame (loaded once, never mutated).
    /// Wrapped in Arc for cheap cloning when sharing across async tasks.
    pub data: Arc<DataFrame>,

    /// Human-readable label (e.g., "Demographics" for DM).
    pub label: Option<String>,
}

impl DomainSource {
    /// Create a new domain source.
    pub fn new(file_path: PathBuf, data: DataFrame, label: Option<String>) -> Self {
        Self {
            file_path,
            data: Arc::new(data),
            label,
        }
    }

    /// Create a new domain source with pre-wrapped Arc data.
    pub fn with_arc(file_path: PathBuf, data: Arc<DataFrame>, label: Option<String>) -> Self {
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
}

// =============================================================================
// DOMAIN STATE - ENUM WITH SOURCE AND GENERATED VARIANTS
// =============================================================================

/// A single SDTM domain - either source-mapped or generated.
///
/// # Variants
///
/// - **Source**: Mapped from CSV source data with mapping/normalization pipeline
/// - **Generated**: Created via UI for CO, RELREC, RELSPEC, RELSUB domains
///
/// # Design Notes
///
/// This is an enum (sum type) to make illegal states unrepresentable:
/// - Source domains have mapping/normalization; generated don't
/// - Generated domains have entry lists; source domains don't
/// - No dead fields, no Option-based "is this a generated domain?" checks
///
/// # Example
///
/// ```ignore
/// match &domain {
///     DomainState::Source(src) => {
///         // Access mapping, normalization, supp_config
///         let summary = src.mapping.summary();
///     }
///     DomainState::Generated(gen) => {
///         // Access generated data and entries
///         let data = &gen.data;
///     }
/// }
/// ```
#[derive(Clone)]
pub enum DomainState {
    /// Domain mapped from CSV source data.
    Source(SourceDomainState),
    /// Domain generated via UI (CO, RELREC, RELSPEC, RELSUB).
    Generated(GeneratedDomainState),
}

impl DomainState {
    /// Create a new source-mapped domain.
    ///
    /// Automatically infers the normalization pipeline from the SDTM domain
    /// metadata. This pipeline defines what transformations will be applied
    /// to each variable during export.
    pub fn new_source(source: DomainSource, mapping: MappingState) -> Self {
        Self::Source(SourceDomainState::new(source, mapping))
    }

    /// Create a new generated domain.
    pub fn new_generated(
        domain_type: GeneratedDomainType,
        data: DataFrame,
        entries: Vec<GeneratedDomainEntry>,
        definition: SdtmDomain,
    ) -> Self {
        Self::Generated(GeneratedDomainState::new(
            domain_type,
            data,
            entries,
            definition,
        ))
    }

    /// Check if this is a source-mapped domain.
    #[inline]
    pub fn is_source(&self) -> bool {
        matches!(self, Self::Source(_))
    }

    /// Check if this is a generated domain.
    #[inline]
    pub fn is_generated(&self) -> bool {
        matches!(self, Self::Generated(_))
    }

    /// Get as source domain (if applicable).
    pub fn as_source(&self) -> Option<&SourceDomainState> {
        match self {
            Self::Source(s) => Some(s),
            Self::Generated(_) => None,
        }
    }

    /// Get as mutable source domain (if applicable).
    pub fn as_source_mut(&mut self) -> Option<&mut SourceDomainState> {
        match self {
            Self::Source(s) => Some(s),
            Self::Generated(_) => None,
        }
    }

    /// Get as generated domain (if applicable).
    pub fn as_generated(&self) -> Option<&GeneratedDomainState> {
        match self {
            Self::Source(_) => None,
            Self::Generated(g) => Some(g),
        }
    }

    /// Get as mutable generated domain (if applicable).
    pub fn as_generated_mut(&mut self) -> Option<&mut GeneratedDomainState> {
        match self {
            Self::Source(_) => None,
            Self::Generated(g) => Some(g),
        }
    }

    // =========================================================================
    // Common methods that work for both variants
    // =========================================================================

    /// Get validation summary as (warnings, errors) count.
    ///
    /// Returns `None` if validation hasn't been run yet.
    /// Returns `Some((warnings, errors))` from cached validation report.
    pub fn validation_summary(&self) -> Option<(usize, usize)> {
        let cache = match self {
            Self::Source(s) => s.validation_cache.as_ref(),
            Self::Generated(g) => g.validation_cache.as_ref(),
        };

        cache.map(|report| {
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

    /// Get the cached validation report.
    pub fn validation_cache(&self) -> Option<&ValidationReport> {
        match self {
            Self::Source(s) => s.validation_cache.as_ref(),
            Self::Generated(g) => g.validation_cache.as_ref(),
        }
    }

    /// Set the validation cache.
    pub fn set_validation_cache(&mut self, report: ValidationReport) {
        match self {
            Self::Source(s) => s.validation_cache = Some(report),
            Self::Generated(g) => g.validation_cache = Some(report),
        }
    }

    /// Clear validation cache (call when data changes).
    pub fn invalidate_validation(&mut self) {
        match self {
            Self::Source(s) => s.validation_cache = None,
            Self::Generated(g) => g.validation_cache = None,
        }
    }

    /// Get display name: label or fallback to code.
    pub fn display_name(&self, code: &str) -> String {
        match self {
            Self::Source(s) => s.display_name(code),
            Self::Generated(g) => g.display_name(code),
        }
    }

    /// Get row count from data.
    #[inline]
    pub fn row_count(&self) -> usize {
        match self {
            Self::Source(s) => s.row_count(),
            Self::Generated(g) => g.row_count(),
        }
    }

    /// Get the domain label (if set).
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Source(s) => s.source.label.as_deref(),
            Self::Generated(g) => Some(g.domain_type.label()),
        }
    }

    /// Get the DataFrame for this domain.
    ///
    /// For source domains, this is the source data.
    /// For generated domains, this is the generated output.
    pub fn data(&self) -> &Arc<DataFrame> {
        match self {
            Self::Source(s) => &s.source.data,
            Self::Generated(g) => &g.data,
        }
    }

    /// Get the CDISC domain definition (if available).
    ///
    /// For source domains, this comes from the mapping state.
    /// For generated domains, this is stored directly.
    pub fn definition(&self) -> Option<&SdtmDomain> {
        match self {
            Self::Source(s) => Some(s.mapping.domain()),
            Self::Generated(g) => Some(&g.definition),
        }
    }

    // =========================================================================
    // Source-only methods (provided for backwards compatibility, delegate to as_source)
    // =========================================================================

    /// Get mapping summary (source domains only).
    ///
    /// Returns a default summary for generated domains.
    pub fn summary(&self) -> tss_submit::MappingSummary {
        match self {
            Self::Source(s) => s.mapping.summary(),
            Self::Generated(_) => tss_submit::MappingSummary {
                total_variables: 0,
                mapped: 0,
                unmapped_required: 0,
                unmapped_expected: 0,
                unmapped_permissible: 0,
                auto_generated: 0,
                not_collected: 0,
                omitted: 0,
            },
        }
    }
}

impl std::fmt::Debug for DomainState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(s) => f
                .debug_struct("DomainState::Source")
                .field("file", &s.source.file_path)
                .field("rows", &s.source.row_count())
                .field("mapping_summary", &s.mapping.summary())
                .finish_non_exhaustive(),
            Self::Generated(g) => f
                .debug_struct("DomainState::Generated")
                .field("type", &g.domain_type)
                .field("rows", &g.row_count())
                .field("entries", &g.entries.len())
                .finish_non_exhaustive(),
        }
    }
}

// =============================================================================
// SOURCE DOMAIN STATE
// =============================================================================

/// State for domains mapped from source CSV files.
///
/// This contains all the data needed for the mapping/normalization workflow:
/// - Source data from CSV
/// - Mapping state (which source columns map to which CDISC variables)
/// - Normalization pipeline (transformations to apply during export)
/// - SUPP configuration for unmapped columns
#[derive(Clone)]
pub struct SourceDomainState {
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

impl SourceDomainState {
    /// Create a new source domain.
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

impl std::fmt::Debug for SourceDomainState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceDomainState")
            .field("source", &self.source.file_path)
            .field("rows", &self.source.row_count())
            .field("mapping_summary", &self.mapping.summary())
            .finish_non_exhaustive()
    }
}

// =============================================================================
// GENERATED DOMAIN STATE
// =============================================================================

/// State for generated domains (CO, RELREC, RELSPEC, RELSUB).
///
/// Generated domains are created via UI rather than mapped from source CSV:
/// - User enters relationship/comment data via builder UI
/// - Application generates the DataFrame with proper CDISC structure
/// - No mapping or normalization needed - data is already in final form
#[derive(Clone)]
pub struct GeneratedDomainState {
    /// Type of generated domain.
    pub domain_type: GeneratedDomainType,

    /// CDISC domain definition.
    pub definition: SdtmDomain,

    /// Generated DataFrame ready for export.
    /// Wrapped in Arc for cheap cloning.
    pub data: Arc<DataFrame>,

    /// Source entries that were used to generate this domain.
    /// Kept for editing/regeneration.
    pub entries: Vec<GeneratedDomainEntry>,

    /// Cached validation report.
    pub validation_cache: Option<ValidationReport>,
}

impl GeneratedDomainState {
    /// Create a new generated domain.
    pub fn new(
        domain_type: GeneratedDomainType,
        data: DataFrame,
        entries: Vec<GeneratedDomainEntry>,
        definition: SdtmDomain,
    ) -> Self {
        Self {
            domain_type,
            definition,
            data: Arc::new(data),
            entries,
            validation_cache: None,
        }
    }

    /// Get display name from domain type.
    pub fn display_name(&self, _code: &str) -> String {
        self.domain_type.label().to_string()
    }

    /// Get row count from generated data.
    #[inline]
    pub fn row_count(&self) -> usize {
        self.data.height()
    }

    /// Get the domain code.
    #[inline]
    pub fn code(&self) -> &'static str {
        self.domain_type.code()
    }
}

impl std::fmt::Debug for GeneratedDomainState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeneratedDomainState")
            .field("type", &self.domain_type)
            .field("rows", &self.row_count())
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}

// =============================================================================
// BACKWARDS COMPATIBILITY - DomainState::new
// =============================================================================

impl DomainState {
    /// Create a new domain (backwards-compatible alias for new_source).
    ///
    /// This exists for backwards compatibility with existing code that calls
    /// `DomainState::new(source, mapping)`. New code should prefer
    /// `DomainState::new_source()` for clarity.
    #[inline]
    pub fn new(source: DomainSource, mapping: MappingState) -> Self {
        Self::new_source(source, mapping)
    }
}
