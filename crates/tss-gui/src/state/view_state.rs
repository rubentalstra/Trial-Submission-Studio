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
use std::collections::BTreeMap;
use tss_validate::ValidationReport;

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
pub enum ViewState {
    /// Home screen - study selection and overview.
    Home {
        /// Selected workflow mode (SDTM, ADaM, SEND).
        workflow_mode: WorkflowMode,
        /// Whether close study confirmation is shown.
        close_confirm: bool,
    },

    /// Domain editor with tabbed interface.
    DomainEditor {
        /// Domain code being edited (e.g., "DM", "AE").
        domain: String,
        /// Active tab.
        tab: EditorTab,
        /// Mapping tab UI state.
        mapping_ui: MappingUiState,
        /// Transform tab UI state.
        transform_ui: TransformUiState,
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
            close_confirm: false,
        }
    }

    /// Create domain editor view state.
    pub fn domain_editor(domain: impl Into<String>, tab: EditorTab) -> Self {
        Self::DomainEditor {
            domain: domain.into(),
            tab,
            mapping_ui: MappingUiState::default(),
            transform_ui: TransformUiState::default(),
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
    /// Data transformation/normalization.
    Transform,
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
            Self::Transform => "Transform",
            Self::Validation => "Validation",
            Self::Preview => "Preview",
            Self::Supp => "SUPP",
        }
    }

    /// Get description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Mapping => "Map source columns to CDISC variables",
            Self::Transform => "Configure data normalization rules",
            Self::Validation => "Review CDISC conformance issues",
            Self::Preview => "Preview transformed output data",
            Self::Supp => "Configure supplemental qualifiers",
        }
    }

    /// All tabs in display order.
    pub const ALL: [EditorTab; 5] = [
        Self::Mapping,
        Self::Transform,
        Self::Validation,
        Self::Preview,
        Self::Supp,
    ];

    /// Get tab index (0-based).
    pub fn index(&self) -> usize {
        match self {
            Self::Mapping => 0,
            Self::Transform => 1,
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
    /// "Not collected" dialog state.
    pub not_collected_dialog: Option<NotCollectedDialog>,
}

/// State for the "not collected" reason dialog.
#[derive(Debug, Clone, Default)]
pub struct NotCollectedDialog {
    /// Variable being marked.
    pub variable: String,
    /// Reason text being edited.
    pub reason: String,
}

// =============================================================================
// TRANSFORM TAB UI STATE
// =============================================================================

/// UI state for the transform tab.
#[derive(Debug, Clone, Default)]
pub struct TransformUiState {
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
#[derive(Debug, Clone, Default)]
pub struct SuppUiState {
    /// Selected column for detail view.
    pub selected_column: Option<String>,
    /// Editing state for the selected column.
    pub editing: Option<SuppEditingState>,
}

/// Editing state for a SUPP column.
#[derive(Debug, Clone, Default)]
pub struct SuppEditingState {
    /// Column name being edited.
    pub column: String,
    /// QNAM value.
    pub qnam: String,
    /// QLABEL value.
    pub qlabel: String,
    /// QORIG value.
    pub qorig: QualifierOrigin,
    /// QEVAL value.
    pub qeval: String,
}

/// SUPP qualifier origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QualifierOrigin {
    /// Data from case report form.
    #[default]
    Crf,
    /// Derived from other data.
    Derived,
    /// Sponsor-assigned value.
    Assigned,
}

impl QualifierOrigin {
    /// Get CDISC code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Crf => "CRF",
            Self::Derived => "DERIVED",
            Self::Assigned => "ASSIGNED",
        }
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Crf => "Case Report Form",
            Self::Derived => "Derived",
            Self::Assigned => "Assigned",
        }
    }

    /// All values.
    pub const ALL: [QualifierOrigin; 3] = [Self::Crf, Self::Derived, Self::Assigned];
}

/// SUPP column action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SuppAction {
    /// Pending review.
    #[default]
    Pending,
    /// Add to SUPP domain.
    AddToSupp,
    /// Skip (don't include).
    Skip,
}

impl SuppAction {
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::AddToSupp => "Add to SUPP",
            Self::Skip => "Skip",
        }
    }
}

/// SUPP column configuration (domain data, not UI state).
#[derive(Debug, Clone)]
pub struct SuppColumnConfig {
    /// Action for this column.
    pub action: SuppAction,
    /// QNAM value (max 8 chars).
    pub qnam: String,
    /// QLABEL value (max 40 chars).
    pub qlabel: String,
    /// QORIG value.
    pub qorig: QualifierOrigin,
    /// QEVAL value.
    pub qeval: String,
}

// =============================================================================
// EXPORT VIEW STATE
// =============================================================================

/// UI state for the export view.
#[derive(Debug, Clone, Default)]
pub struct ExportViewState {
    /// Selected domains for export.
    pub selected_domains: BTreeMap<String, bool>,
    /// Export in progress.
    pub is_exporting: bool,
    /// Current export step description.
    pub current_step: Option<String>,
    /// Export progress (0.0 to 1.0).
    pub progress: f32,
    /// Files that have been written.
    pub written_files: Vec<std::path::PathBuf>,
    /// Export result.
    pub result: Option<ExportResult>,
}

/// Export result.
#[derive(Debug, Clone)]
pub enum ExportResult {
    /// Export succeeded.
    Success {
        files: Vec<std::path::PathBuf>,
        domains_exported: usize,
    },
    /// Export failed.
    Error { message: String },
    /// Export was cancelled.
    Cancelled,
}
