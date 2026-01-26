//! Domain state - source-mapped domains.
//!
//! A domain represents a single SDTM dataset (e.g., DM, AE, LB, CO, RELREC).
//! All domains are mapped from CSV source data.
//!
//! Per CDISC guidelines, CO, RELREC, RELSPEC, and RELSUB domains contain
//! **collected data** and should be imported from source CSV files, not
//! manually generated. (SDTM-IG v3.4 Sections 5.1, 8.2, 8.7, 8.8)

use polars::prelude::{DataFrame, PlSmallStr};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tss_standards::SdtmDomain;
use tss_submit::MappingState;
use tss_submit::{NormalizationPipeline, Severity, ValidationReport, infer_normalization_rules};

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
// DOMAIN STATE
// =============================================================================

/// Type alias for backwards compatibility.
/// All domains are now source-mapped domains.
pub type DomainState = SourceDomainState;

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

    /// Get the cached validation report.
    pub fn validation_cache(&self) -> Option<&ValidationReport> {
        self.validation_cache.as_ref()
    }

    /// Set the validation cache.
    pub fn set_validation_cache(&mut self, report: ValidationReport) {
        self.validation_cache = Some(report);
    }

    /// Clear validation cache (call when data changes).
    pub fn invalidate_validation(&mut self) {
        self.validation_cache = None;
    }

    /// Get mapping summary.
    pub fn summary(&self) -> tss_submit::MappingSummary {
        self.mapping.summary()
    }

    /// Get the domain label (if set).
    pub fn label(&self) -> Option<&str> {
        self.source.label.as_deref()
    }

    /// Get the DataFrame for this domain.
    pub fn data(&self) -> &Arc<DataFrame> {
        &self.source.data
    }

    /// Get the CDISC domain definition.
    pub fn definition(&self) -> &SdtmDomain {
        self.mapping.domain()
    }

    /// Check if this is a source-mapped domain (always true now).
    #[inline]
    pub fn is_source(&self) -> bool {
        true
    }

    /// Get as source domain (returns self for backwards compatibility).
    pub fn as_source(&self) -> Option<&SourceDomainState> {
        Some(self)
    }

    /// Get as mutable source domain (returns self for backwards compatibility).
    pub fn as_source_mut(&mut self) -> Option<&mut SourceDomainState> {
        Some(self)
    }
}

impl std::fmt::Debug for SourceDomainState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomainState")
            .field("source", &self.source.file_path)
            .field("rows", &self.source.row_count())
            .field("mapping_summary", &self.mapping.summary())
            .finish_non_exhaustive()
    }
}
