//! Study-level runtime state

use super::TransformState;
use crate::services::MappingState;
use polars::prelude::DataFrame;
use sdtm_ingest::StudyMetadata;
use sdtm_validate::ValidationReport;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

// ============================================================================
// Preview Tab State
// ============================================================================

/// State for Preview tab display
#[derive(Debug, Clone)]
pub struct PreviewState {
    /// Current page number (0-indexed)
    pub current_page: usize,
    /// Rows per page
    pub rows_per_page: usize,
    /// Error message if preview generation failed
    pub error: Option<String>,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            current_page: 0,
            rows_per_page: 50,
            error: None,
        }
    }
}

// ============================================================================
// SUPP Tab State
// ============================================================================

/// State for SUPP (Supplemental Qualifiers) configuration
#[derive(Debug, Clone, Default)]
pub struct SuppState {
    /// Configuration for each unmapped source column
    pub columns: BTreeMap<String, SuppColumnConfig>,
    /// Currently selected column for detail view
    pub selected_column: Option<String>,
}

impl SuppState {
    /// Count columns by action
    pub fn count_by_action(&self) -> (usize, usize, usize) {
        let mut pending = 0;
        let mut added = 0;
        let mut skipped = 0;
        for config in self.columns.values() {
            match config.action {
                SuppAction::Pending => pending += 1,
                SuppAction::AddToSupp => added += 1,
                SuppAction::Skip => skipped += 1,
            }
        }
        (pending, added, skipped)
    }

    /// Get all columns configured as AddToSupp
    pub fn supp_columns(&self) -> Vec<&SuppColumnConfig> {
        self.columns
            .values()
            .filter(|c| c.action == SuppAction::AddToSupp)
            .collect()
    }
}

/// Configuration for a single source column in SUPP
#[derive(Debug, Clone)]
pub struct SuppColumnConfig {
    /// Action: Add to SUPP or Skip
    pub action: SuppAction,
    /// QNAM value (max 8 chars, uppercase, no leading numbers)
    pub qnam: String,
    /// QLABEL value (max 40 chars)
    pub qlabel: String,
    /// Original column name (for reference)
    pub source_column: String,
    /// Auto-suggested QNAM (for "Suggest" button)
    pub suggested_qnam: String,
}

impl SuppColumnConfig {
    /// Create a new config with auto-suggested QNAM
    pub fn new(source_column: String, domain_code: &str) -> Self {
        let suggested = suggest_qnam(&source_column, domain_code);
        Self {
            action: SuppAction::Pending,
            qnam: suggested.clone(),
            qlabel: String::new(),
            source_column,
            suggested_qnam: suggested,
        }
    }

    /// Validate QNAM according to SDTMIG rules
    pub fn validate_qnam(&self) -> Result<(), String> {
        validate_qnam(&self.qnam)
    }
}

/// Action for a SUPP column
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SuppAction {
    /// Not yet decided
    #[default]
    Pending,
    /// Include in SUPP-- dataset
    AddToSupp,
    /// Exclude from export
    Skip,
}

impl SuppAction {
    /// Get icon for this action
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pending => egui_phosphor::regular::CIRCLE_DASHED,
            Self::AddToSupp => egui_phosphor::regular::CHECK,
            Self::Skip => egui_phosphor::regular::MINUS,
        }
    }
}

/// Suggest a QNAM from a source column name
///
/// Rules:
/// - Uppercase only
/// - Max 8 characters
/// - No leading numbers
/// - Prefix with domain code
fn suggest_qnam(column_name: &str, domain_code: &str) -> String {
    // Clean up the column name
    let clean = column_name
        .to_uppercase()
        .replace('_', "")
        .replace('-', "")
        .replace(' ', "");

    // Strip common prefixes
    let base = clean
        .strip_prefix("EXTRA")
        .or_else(|| clean.strip_prefix("ADDITIONAL"))
        .or_else(|| clean.strip_prefix("OTHER"))
        .or_else(|| clean.strip_prefix("CUSTOM"))
        .unwrap_or(&clean);

    // If base is empty or starts with a number, use column name chars
    let base = if base.is_empty()
        || base
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        &clean
    } else {
        base
    };

    // Calculate how many chars we can use from base (max 8 - domain_code.len())
    let max_base_len = 8usize.saturating_sub(domain_code.len());
    let truncated_base: String = base.chars().take(max_base_len).collect();

    // Combine domain code and truncated base
    let suggested = format!("{}{}", domain_code.to_uppercase(), truncated_base);

    // Final truncation to ensure max 8 chars
    suggested.chars().take(8).collect()
}

/// Validate a QNAM according to SDTMIG rules
pub fn validate_qnam(qnam: &str) -> Result<(), String> {
    if qnam.is_empty() {
        return Err("QNAM cannot be empty".to_string());
    }
    if qnam.len() > 8 {
        return Err("QNAM must be 8 characters or less".to_string());
    }
    if qnam
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        return Err("QNAM cannot start with a number".to_string());
    }
    if !qnam.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("QNAM can only contain letters, numbers, and underscores".to_string());
    }
    Ok(())
}

/// Result of checking domain initialization state
pub enum DomainInitState {
    /// Domain has mapping state, ready to display
    Ready,
    /// Just set to Loading, need repaint
    StartLoading,
    /// In Loading state, need to run initialization
    DoInitialize,
    /// Failed to find domain or other error
    Error,
}

/// Runtime state for a loaded study
pub struct StudyState {
    /// Study identifier (derived from folder name)
    pub study_id: String,
    /// Path to study folder
    pub study_folder: PathBuf,
    /// State for each discovered domain
    pub domains: HashMap<String, DomainState>,
    /// Source metadata (Items.csv, CodeLists.csv)
    pub metadata: Option<StudyMetadata>,
}

impl StudyState {
    /// Create a new study state from a folder path
    pub fn new(study_folder: PathBuf) -> Self {
        let study_id = study_folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Study")
            .to_string();

        Self {
            study_id,
            study_folder,
            domains: HashMap::new(),
            metadata: None,
        }
    }

    /// Get the label for a source column from Items.csv metadata
    pub fn get_column_label(&self, column_id: &str) -> Option<&str> {
        self.metadata
            .as_ref()?
            .items
            .get(&column_id.to_uppercase())
            .map(|item| item.label.as_str())
    }

    /// Get a domain by code
    pub fn get_domain(&self, code: &str) -> Option<&DomainState> {
        self.domains.get(code)
    }

    /// Get a mutable domain by code
    pub fn get_domain_mut(&mut self, code: &str) -> Option<&mut DomainState> {
        self.domains.get_mut(code)
    }

    /// Get all domain codes sorted alphabetically
    pub fn domain_codes(&self) -> Vec<&str> {
        let mut codes: Vec<&str> = self.domains.keys().map(String::as_str).collect();
        codes.sort();
        codes
    }

    /// Check domain init state and transition if needed (mutable version)
    pub fn check_domain_init(&mut self, domain_code: &str) -> DomainInitState {
        let Some(domain) = self.domains.get_mut(domain_code) else {
            return DomainInitState::Error;
        };

        if domain.mapping_state.is_some() {
            return DomainInitState::Ready;
        }

        match domain.status {
            DomainStatus::NotStarted => {
                domain.status = DomainStatus::Loading;
                DomainInitState::StartLoading
            }
            DomainStatus::Loading => DomainInitState::DoInitialize,
            _ => DomainInitState::Error,
        }
    }
}

/// State for a single domain
pub struct DomainState {
    /// Path to source CSV file
    pub source_file: PathBuf,
    /// Source data as DataFrame
    pub source_data: DataFrame,
    /// Current status
    pub status: DomainStatus,
    /// Human-readable domain label (e.g., "Demographics" for DM)
    pub domain_label: Option<String>,
    /// Interactive mapping state (for GUI)
    pub mapping_state: Option<MappingState>,
    /// Transform display state (read-only, shows what will be applied)
    pub transform_state: Option<TransformState>,
    /// Validation results
    pub validation: Option<ValidationReport>,
    /// Selected validation issue index (for detail view)
    pub validation_selected_idx: Option<usize>,
    /// Preview of processed data
    pub preview_data: Option<DataFrame>,
    /// Preview tab UI state (pagination, etc.)
    pub preview_state: PreviewState,
    /// SUPP configuration state (unmapped columns)
    pub supp_state: Option<SuppState>,
}

impl DomainState {
    /// Create a new domain state with optional label
    pub fn new(source_file: PathBuf, source_data: DataFrame, label: Option<String>) -> Self {
        Self {
            source_file,
            source_data,
            status: DomainStatus::NotStarted,
            domain_label: label,
            mapping_state: None,
            transform_state: None,
            validation: None,
            validation_selected_idx: None,
            preview_data: None,
            preview_state: PreviewState::default(),
            supp_state: None,
        }
    }

    /// Get display name: "DM (Demographics)" or just "DM" if no label
    pub fn display_name(&self, code: &str) -> String {
        match &self.domain_label {
            Some(label) => format!("{} ({})", code, label),
            None => code.to_string(),
        }
    }

    /// Get source column names
    pub fn source_columns(&self) -> Vec<String> {
        self.source_data
            .get_column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get row count
    pub fn row_count(&self) -> usize {
        self.source_data.height()
    }
}

/// Status of domain processing
#[derive(Default, Clone, Copy, PartialEq)]
#[allow(dead_code)] // Variants used in pattern matching, constructed when features are implemented
pub enum DomainStatus {
    /// Not yet started
    #[default]
    NotStarted,
    /// Loading/initializing mapping
    Loading,
    /// Mapping in progress
    MappingInProgress,
    /// Mapping complete
    MappingComplete,
    /// Validation failed
    ValidationFailed,
    /// Ready for export
    ReadyForExport,
}

impl DomainStatus {
    /// Get status icon (phosphor icon)
    pub fn icon(&self) -> &'static str {
        match self {
            Self::NotStarted => egui_phosphor::regular::CIRCLE,
            Self::Loading => egui_phosphor::regular::SPINNER_GAP,
            Self::MappingInProgress => egui_phosphor::regular::CIRCLE_HALF,
            Self::MappingComplete => egui_phosphor::regular::CIRCLE_WAVY_CHECK,
            Self::ValidationFailed => egui_phosphor::regular::X_CIRCLE,
            Self::ReadyForExport => egui_phosphor::regular::CHECK_CIRCLE,
        }
    }
}
