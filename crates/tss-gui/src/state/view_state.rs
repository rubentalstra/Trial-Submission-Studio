//! View state - current view and associated UI state.
//!
//! # Architecture
//!
//! Instead of a flat `View` enum + separate `UiState` struct, we combine them:
//! - Each view variant holds its own UI state
//! - Navigation changes the entire `ViewState`
//! - UI state is scoped to where it's used
//!
//! This eliminates the need to synchronize separate state containers
//! and makes it clear what state belongs to which view.

use polars::prelude::DataFrame;
use tss_submit::ValidationReport;

use super::domain_state::{SuppColumnConfig, SuppOrigin};

// =============================================================================
// VIEW STATE (Current view + its UI state)
// =============================================================================

/// Current view and its associated UI state.
///
/// # Design
///
/// Each view variant contains all the UI state needed for that view.
/// When navigating, the entire view state is replaced, which automatically
/// clears any transient UI state.
///
/// # Example
///
/// ```ignore
/// // Navigate to domain editor
/// state.view = ViewState::domain_editor("DM", EditorTab::Mapping);
///
/// // Access domain editor state
/// if let ViewState::DomainEditor { mapping_ui, .. } = &mut state.view {
///     mapping_ui.selected_variable = Some(0);
/// }
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ViewState {
    /// Home screen - study selection and overview.
    Home {
        /// Selected workflow mode (SDTM, ADaM, SEND).
        workflow_mode: WorkflowMode,
    },

    /// Domain editor with tabbed interface.
    DomainEditor {
        /// Domain code being edited (e.g., "DM", "AE").
        domain: String,
        /// Active tab.
        tab: EditorTab,
        /// Mapping tab UI state.
        mapping_ui: MappingUiState,
        /// Normalization tab UI state.
        normalization_ui: NormalizationUiState,
        /// Validation tab UI state.
        validation_ui: ValidationUiState,
        /// Preview tab UI state.
        preview_ui: PreviewUiState,
        /// SUPP tab UI state.
        supp_ui: SuppUiState,
        /// Cached preview DataFrame (computed on demand).
        preview_cache: Option<DataFrame>,
        /// Cached validation report (computed on demand).
        validation_cache: Option<ValidationReport>,
    },

    /// Export screen.
    Export(ExportViewState),
}

impl Default for ViewState {
    fn default() -> Self {
        Self::home()
    }
}

impl ViewState {
    /// Create home view state.
    pub fn home() -> Self {
        Self::Home {
            workflow_mode: WorkflowMode::default(),
        }
    }

    /// Create domain editor view state.
    pub fn domain_editor(domain: impl Into<String>, tab: EditorTab) -> Self {
        Self::DomainEditor {
            domain: domain.into(),
            tab,
            mapping_ui: MappingUiState::default(),
            normalization_ui: NormalizationUiState::default(),
            validation_ui: ValidationUiState::default(),
            preview_ui: PreviewUiState::default(),
            supp_ui: SuppUiState::default(),
            preview_cache: None,
            validation_cache: None,
        }
    }

    /// Create export view state.
    pub fn export() -> Self {
        Self::Export(ExportViewState::default())
    }

    // =========================================================================
    // QUERIES
    // =========================================================================

    /// Check if this is the home view.
    pub fn is_home(&self) -> bool {
        matches!(self, Self::Home { .. })
    }

    /// Check if this is a domain editor view.
    pub fn is_domain_editor(&self) -> bool {
        matches!(self, Self::DomainEditor { .. })
    }

    /// Check if this is the export view.
    pub fn is_export(&self) -> bool {
        matches!(self, Self::Export(_))
    }

    /// Get the current domain code if in domain editor.
    pub fn current_domain(&self) -> Option<&str> {
        match self {
            Self::DomainEditor { domain, .. } => Some(domain),
            _ => None,
        }
    }

    /// Get the current tab if in domain editor.
    pub fn current_tab(&self) -> Option<EditorTab> {
        match self {
            Self::DomainEditor { tab, .. } => Some(*tab),
            _ => None,
        }
    }

    /// Get workflow mode.
    pub fn workflow_mode(&self) -> WorkflowMode {
        match self {
            Self::Home { workflow_mode, .. } => *workflow_mode,
            _ => WorkflowMode::default(),
        }
    }
}

// =============================================================================
// WORKFLOW MODE
// =============================================================================

/// CDISC standard workflow mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkflowMode {
    /// SDTM - Study Data Tabulation Model.
    #[default]
    Sdtm,
    /// ADaM - Analysis Data Model.
    Adam,
    /// SEND - Standard for Exchange of Nonclinical Data.
    Send,
}

impl WorkflowMode {
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sdtm => "SDTM",
            Self::Adam => "ADaM",
            Self::Send => "SEND",
        }
    }

    /// Get description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Sdtm => "Study Data Tabulation Model - Clinical trial tabulation data",
            Self::Adam => "Analysis Data Model - Analysis-ready datasets",
            Self::Send => "Standard for Exchange of Nonclinical Data - Animal studies",
        }
    }

    /// All workflow modes.
    pub const ALL: [WorkflowMode; 3] = [Self::Sdtm, Self::Adam, Self::Send];
}

// =============================================================================
// EDITOR TAB
// =============================================================================

/// Tabs in the domain editor.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorTab {
    /// Variable mapping.
    #[default]
    Mapping,
    /// Data normalization rules.
    Normalization,
    /// Validation results.
    Validation,
    /// Data preview.
    Preview,
    /// SUPP qualifier configuration.
    Supp,
}

impl EditorTab {
    /// Get display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mapping => "Mapping",
            Self::Normalization => "Normalization",
            Self::Validation => "Validation",
            Self::Preview => "Preview",
            Self::Supp => "SUPP",
        }
    }

    /// Get description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Mapping => "Map source columns to CDISC variables",
            Self::Normalization => "Configure data normalization rules",
            Self::Validation => "Review CDISC conformance issues",
            Self::Preview => "Preview transformed output data",
            Self::Supp => "Configure supplemental qualifiers",
        }
    }

    /// All tabs in display order.
    pub const ALL: [EditorTab; 5] = [
        Self::Mapping,
        Self::Normalization,
        Self::Validation,
        Self::Preview,
        Self::Supp,
    ];

    /// Get tab index (0-based).
    pub fn index(&self) -> usize {
        match self {
            Self::Mapping => 0,
            Self::Normalization => 1,
            Self::Validation => 2,
            Self::Preview => 3,
            Self::Supp => 4,
        }
    }

    /// Create tab from index.
    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

// =============================================================================
// MAPPING TAB UI STATE
// =============================================================================

/// UI state for the mapping tab.
#[derive(Debug, Clone, Default)]
pub struct MappingUiState {
    /// Selected variable index.
    pub selected_variable: Option<usize>,
    /// Search filter text.
    pub search_filter: String,
    /// Show only unmapped variables.
    pub filter_unmapped: bool,
    /// Show only required variables.
    pub filter_required: bool,
    /// Inline "Not Collected" editing state.
    /// Set when user is entering/editing a reason for marking a variable as not collected.
    pub not_collected_edit: Option<NotCollectedEdit>,
}

/// State for inline "Not Collected" reason editing.
#[derive(Debug, Clone, Default)]
pub struct NotCollectedEdit {
    /// Variable being marked/edited.
    pub variable: String,
    /// Reason text being entered.
    pub reason: String,
}

// =============================================================================
// TRANSFORM TAB UI STATE
// =============================================================================

/// UI state for the transform tab.
#[derive(Debug, Clone, Default)]
pub struct NormalizationUiState {
    /// Selected rule index.
    pub selected_rule: Option<usize>,
}

// =============================================================================
// VALIDATION TAB UI STATE
// =============================================================================

/// UI state for the validation tab.
#[derive(Debug, Clone, Default)]
pub struct ValidationUiState {
    /// Selected issue index.
    pub selected_issue: Option<usize>,
    /// Severity filter.
    pub severity_filter: SeverityFilter,
}

/// Filter for validation issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeverityFilter {
    /// Show all issues.
    #[default]
    All,
    /// Show only errors.
    Errors,
    /// Show only warnings.
    Warnings,
    /// Show only info.
    Info,
}

impl SeverityFilter {
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Errors => "Errors",
            Self::Warnings => "Warnings",
            Self::Info => "Info",
        }
    }
}

// =============================================================================
// PREVIEW TAB UI STATE
// =============================================================================

/// UI state for the preview tab.
#[derive(Debug, Clone)]
pub struct PreviewUiState {
    /// Current page (0-indexed).
    pub current_page: usize,
    /// Rows per page.
    pub rows_per_page: usize,
    /// Whether preview is being rebuilt.
    pub is_rebuilding: bool,
    /// Error message if rebuild failed.
    pub error: Option<String>,
}

impl Default for PreviewUiState {
    fn default() -> Self {
        Self {
            current_page: 0,
            rows_per_page: 50,
            is_rebuilding: false,
            error: None,
        }
    }
}

// =============================================================================
// SUPP TAB UI STATE
// =============================================================================

/// UI state for the SUPP tab.
///
/// # Edit Draft Pattern
///
/// When editing an already-included column, changes are stored in `edit_draft`
/// and only committed to `supp_config` when the user clicks "Save".
/// This allows cancellation without losing the original values.
///
/// For pending columns, edits go directly to `supp_config` (no draft needed).
#[derive(Debug, Clone, Default)]
pub struct SuppUiState {
    /// Selected column for detail view.
    pub selected_column: Option<String>,
    /// Search filter for column names.
    pub search_filter: String,
    /// Filter mode for columns.
    pub filter_mode: SuppFilterMode,
    /// Edit draft for already-included columns.
    /// When Some, user is editing an included column.
    /// When None, showing read-only view or editing a pending column.
    pub edit_draft: Option<SuppEditDraft>,
}

impl SuppUiState {
    /// Check if we're in edit mode for an included column.
    pub fn is_editing(&self) -> bool {
        self.edit_draft.is_some()
    }

    /// Start editing with a draft from the current config.
    pub fn start_editing(&mut self, draft: SuppEditDraft) {
        self.edit_draft = Some(draft);
    }

    /// Cancel editing, discarding the draft.
    pub fn cancel_editing(&mut self) {
        self.edit_draft = None;
    }
}

/// Draft state for editing an included SUPP column.
///
/// This holds temporary values while editing, allowing the user
/// to cancel without losing the original configuration.
#[derive(Debug, Clone)]
pub struct SuppEditDraft {
    /// QNAM value being edited.
    pub qnam: String,
    /// QLABEL value being edited.
    pub qlabel: String,
    /// QORIG value being edited.
    pub qorig: SuppOrigin,
    /// QEVAL value being edited.
    pub qeval: String,
}

impl SuppEditDraft {
    /// Create a draft from an existing config.
    pub fn from_config(config: &SuppColumnConfig) -> Self {
        Self {
            qnam: config.qnam.clone(),
            qlabel: config.qlabel.clone(),
            qorig: config.qorig,
            qeval: config.qeval.clone().unwrap_or_default(),
        }
    }
}

/// Filter mode for SUPP columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SuppFilterMode {
    /// Show all unmapped columns.
    #[default]
    All,
    /// Show only pending columns.
    Pending,
    /// Show only columns added to SUPP.
    Included,
    /// Show only skipped columns.
    Skipped,
}

// =============================================================================
// EXPORT VIEW STATE
// =============================================================================

use std::collections::HashSet;
use std::path::PathBuf;

/// Export phase - tracks the current state of the export workflow.
#[derive(Debug, Clone, Default)]
pub enum ExportPhase {
    /// Idle - configuring export options.
    #[default]
    Idle,
    /// Exporting - background task in progress.
    Exporting {
        /// Current domain being processed.
        current_domain: Option<String>,
        /// Current step label.
        current_step: String,
        /// Progress 0.0 to 1.0.
        progress: f32,
        /// Files written so far.
        files_written: Vec<PathBuf>,
    },
    /// Complete - export finished (success or error).
    Complete(ExportResult),
}

impl ExportPhase {
    /// Check if export is in progress.
    pub fn is_exporting(&self) -> bool {
        matches!(self, Self::Exporting { .. })
    }

    /// Check if export is complete.
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete(_))
    }
}

/// Export result.
#[derive(Debug, Clone)]
pub enum ExportResult {
    /// Export succeeded.
    Success {
        /// Output directory.
        output_dir: PathBuf,
        /// Files that were written.
        files: Vec<PathBuf>,
        /// Number of domains exported.
        domains_exported: usize,
        /// Elapsed time in milliseconds.
        elapsed_ms: u64,
        /// Any warnings generated.
        warnings: Vec<String>,
    },
    /// Export failed.
    Error {
        /// Error message.
        message: String,
        /// Domain that caused the error (if applicable).
        domain: Option<String>,
    },
    /// Export was cancelled by user.
    Cancelled,
}

impl ExportResult {
    /// Check if result is successful.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if result is an error.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}

/// UI state for the export view.
///
/// # Design
///
/// The export view follows a master-detail pattern:
/// - **Left panel**: Domain selection list with status indicators
/// - **Right panel**: Export configuration (format, output dir, options)
///
/// State is organized into:
/// - Selection state (which domains to export)
/// - Configuration state (format, output dir) - uses types from settings module
/// - Phase state (idle/exporting/complete)
#[derive(Debug, Clone, Default)]
pub struct ExportViewState {
    /// Selected domain codes for export.
    pub selected_domains: HashSet<String>,
    /// Custom output directory (None = use default: study_folder/export).
    pub output_dir: Option<PathBuf>,
    /// Current export phase.
    pub phase: ExportPhase,
}

impl ExportViewState {
    /// Check if a domain is selected.
    pub fn is_selected(&self, domain: &str) -> bool {
        self.selected_domains.contains(domain)
    }

    /// Toggle domain selection.
    pub fn toggle_domain(&mut self, domain: &str) {
        if self.selected_domains.contains(domain) {
            self.selected_domains.remove(domain);
        } else {
            self.selected_domains.insert(domain.to_string());
        }
    }

    /// Select all domains from a list.
    pub fn select_all(&mut self, domains: impl IntoIterator<Item = String>) {
        self.selected_domains.extend(domains);
    }

    /// Deselect all domains.
    pub fn deselect_all(&mut self) {
        self.selected_domains.clear();
    }

    /// Number of selected domains.
    pub fn selection_count(&self) -> usize {
        self.selected_domains.len()
    }

    /// Check if export can start.
    pub fn can_export(&self) -> bool {
        !self.selected_domains.is_empty() && !self.phase.is_exporting()
    }

    /// Reset to idle state (after completion modal dismissed).
    pub fn reset_phase(&mut self) {
        self.phase = ExportPhase::Idle;
    }

    /// Get the effective output directory.
    pub fn effective_output_dir(&self, study_folder: &std::path::Path) -> PathBuf {
        self.output_dir
            .clone()
            .unwrap_or_else(|| study_folder.join("export"))
    }
}
