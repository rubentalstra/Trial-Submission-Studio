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
/// # Memory Optimization
///
/// Large variants (DomainEditor) are boxed to reduce the size of the enum.
/// This improves cache locality and reduces memory usage when the view
/// is not in that state.
///
/// # Example
///
/// ```ignore
/// // Navigate to domain editor
/// state.view = ViewState::domain_editor("DM", EditorTab::Mapping);
///
/// // Access domain editor state
/// if let ViewState::DomainEditor(editor) = &mut state.view {
///     editor.mapping_ui.selected_variable = Some(0);
/// }
/// ```
#[derive(Debug, Clone)]
pub enum ViewState {
    /// Home screen - study selection and overview.
    Home {
        /// Selected workflow mode (SDTM, ADaM, SEND).
        workflow_mode: WorkflowMode,
    },

    /// Source assignment screen - map source files to target domains.
    SourceAssignment {
        /// Selected workflow mode (SDTM, ADaM, SEND).
        workflow_mode: WorkflowMode,
        /// Assignment UI state.
        assignment_ui: SourceAssignmentUiState,
    },

    /// Domain editor with tabbed interface (boxed for memory efficiency).
    DomainEditor(Box<DomainEditorState>),

    /// Export screen.
    Export(ExportViewState),
}

/// State for the domain editor view.
///
/// Extracted as a separate struct to enable boxing in ViewState enum,
/// which reduces the overall enum size and improves memory efficiency.
#[derive(Debug, Clone)]
pub struct DomainEditorState {
    /// Domain code being edited (e.g., "DM", "AE").
    pub domain: String,
    /// Active tab.
    pub tab: EditorTab,
    /// Mapping tab UI state.
    pub mapping_ui: MappingUiState,
    /// Normalization tab UI state.
    pub normalization_ui: NormalizationUiState,
    /// Validation tab UI state.
    pub validation_ui: ValidationUiState,
    /// Preview tab UI state.
    pub preview_ui: PreviewUiState,
    /// SUPP tab UI state.
    pub supp_ui: SuppUiState,
    /// Cached preview DataFrame (computed on demand).
    pub preview_cache: Option<DataFrame>,
    /// Flag indicating validation needs refresh after mapping change.
    pub validation_dirty: bool,
    /// Flag indicating preview needs refresh after mapping change.
    pub preview_dirty: bool,
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

    /// Create home view state with a specific workflow mode.
    pub fn home_with_mode(workflow_mode: WorkflowMode) -> Self {
        Self::Home { workflow_mode }
    }

    /// Create source assignment view state.
    pub fn source_assignment(
        workflow_mode: WorkflowMode,
        assignment_ui: SourceAssignmentUiState,
    ) -> Self {
        Self::SourceAssignment {
            workflow_mode,
            assignment_ui,
        }
    }

    /// Create domain editor view state with custom rows per page.
    pub fn domain_editor_with_rows(
        domain: impl Into<String>,
        tab: EditorTab,
        rows_per_page: usize,
    ) -> Self {
        Self::DomainEditor(Box::new(DomainEditorState {
            domain: domain.into(),
            tab,
            mapping_ui: MappingUiState::default(),
            normalization_ui: NormalizationUiState::default(),
            validation_ui: ValidationUiState::default(),
            preview_ui: PreviewUiState::with_rows_per_page(rows_per_page),
            supp_ui: SuppUiState::default(),
            preview_cache: None,
            validation_dirty: false,
            preview_dirty: false,
        }))
    }

    /// Create export view state.
    pub fn export() -> Self {
        Self::Export(ExportViewState::default())
    }

    /// Get workflow mode.
    pub fn workflow_mode(&self) -> WorkflowMode {
        match self {
            Self::Home { workflow_mode, .. } => *workflow_mode,
            Self::SourceAssignment { workflow_mode, .. } => *workflow_mode,
            _ => WorkflowMode::default(),
        }
    }

    // =========================================================================
    // HELPER METHODS FOR COMMON PATTERNS
    // =========================================================================
    // VIEW STATE QUERY METHODS
    // =========================================================================
    // These methods are designed for use in the handler refactoring (Phase 4)
    // and are kept even if currently unused to support the planned architecture.

    /// Get the current domain code if in domain editor view.
    ///
    /// Returns `None` if not in domain editor view.
    #[inline]
    pub fn current_domain(&self) -> Option<&str> {
        match self {
            Self::DomainEditor(editor) => Some(&editor.domain),
            _ => None,
        }
    }

    /// Get the current editor tab if in domain editor view.
    ///
    /// Returns `None` if not in domain editor view.
    #[inline]
    pub fn current_tab(&self) -> Option<EditorTab> {
        match self {
            Self::DomainEditor(editor) => Some(editor.tab),
            _ => None,
        }
    }

    /// Check if currently in domain editor view.
    #[inline]
    pub fn is_domain_editor(&self) -> bool {
        matches!(self, Self::DomainEditor(_))
    }

    /// Check if currently in home view.
    #[inline]
    pub fn is_home(&self) -> bool {
        matches!(self, Self::Home { .. })
    }

    /// Check if currently in source assignment view.
    #[inline]
    pub fn is_source_assignment(&self) -> bool {
        matches!(self, Self::SourceAssignment { .. })
    }

    /// Check if currently in export view.
    #[inline]
    pub fn is_export(&self) -> bool {
        matches!(self, Self::Export(_))
    }

    // =========================================================================
    // HELPER METHODS FOR STATE ACCESS
    // =========================================================================
    // These methods provide convenient access to nested UI state.
    // They are designed for use in the handler refactoring (Phase 4) and are
    // kept even if currently unused to support the planned architecture.

    /// Get mutable reference to mapping UI state.
    ///
    /// Returns `None` if not in domain editor view.
    #[inline]
    pub fn mapping_ui_mut(&mut self) -> Option<&mut MappingUiState> {
        match self {
            Self::DomainEditor(editor) => Some(&mut editor.mapping_ui),
            _ => None,
        }
    }

    /// Get immutable reference to mapping UI state.
    ///
    /// Returns `None` if not in domain editor view.
    #[inline]
    pub fn mapping_ui(&self) -> Option<&MappingUiState> {
        match self {
            Self::DomainEditor(editor) => Some(&editor.mapping_ui),
            _ => None,
        }
    }

    /// Get mutable reference to validation UI state.
    ///
    /// Returns `None` if not in domain editor view.
    #[inline]
    pub fn validation_ui_mut(&mut self) -> Option<&mut ValidationUiState> {
        match self {
            Self::DomainEditor(editor) => Some(&mut editor.validation_ui),
            _ => None,
        }
    }

    /// Get mutable reference to preview UI state.
    ///
    /// Returns `None` if not in domain editor view.
    #[inline]
    pub fn preview_ui_mut(&mut self) -> Option<&mut PreviewUiState> {
        match self {
            Self::DomainEditor(editor) => Some(&mut editor.preview_ui),
            _ => None,
        }
    }

    /// Get mutable reference to normalization UI state.
    ///
    /// Returns `None` if not in domain editor view.
    #[inline]
    pub fn normalization_ui_mut(&mut self) -> Option<&mut NormalizationUiState> {
        match self {
            Self::DomainEditor(editor) => Some(&mut editor.normalization_ui),
            _ => None,
        }
    }

    /// Get mutable reference to SUPP UI state.
    ///
    /// Returns `None` if not in domain editor view.
    #[inline]
    pub fn supp_ui_mut(&mut self) -> Option<&mut SuppUiState> {
        match self {
            Self::DomainEditor(editor) => Some(&mut editor.supp_ui),
            _ => None,
        }
    }

    /// Invalidate (clear) the preview cache.
    ///
    /// Call this when data changes that would affect the preview.
    /// The preview will be rebuilt on next access.
    pub fn invalidate_preview(&mut self) {
        if let Self::DomainEditor(editor) = self {
            editor.preview_cache = None;
        }
    }

    /// Get the cached preview DataFrame if available.
    #[inline]
    pub fn preview_cache(&self) -> Option<&DataFrame> {
        match self {
            Self::DomainEditor(editor) => editor.preview_cache.as_ref(),
            _ => None,
        }
    }

    /// Set the preview cache.
    pub fn set_preview_cache(&mut self, df: DataFrame) {
        if let Self::DomainEditor(editor) = self {
            editor.preview_cache = Some(df);
        }
    }

    /// Get mutable reference to source assignment UI state.
    ///
    /// Returns `None` if not in source assignment view.
    #[inline]
    pub fn source_assignment_ui_mut(&mut self) -> Option<&mut SourceAssignmentUiState> {
        match self {
            Self::SourceAssignment { assignment_ui, .. } => Some(assignment_ui),
            _ => None,
        }
    }

    /// Get mutable reference to export view state.
    ///
    /// Returns `None` if not in export view.
    #[inline]
    pub fn export_state_mut(&mut self) -> Option<&mut ExportViewState> {
        match self {
            Self::Export(state) => Some(state),
            _ => None,
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

    /// All tabs in display order.
    pub const ALL: [EditorTab; 5] = [
        Self::Mapping,
        Self::Normalization,
        Self::Validation,
        Self::Preview,
        Self::Supp,
    ];
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
    /// Cached filtered variable indices (for performance).
    /// Contains indices into the domain's variables vec that pass the current filters.
    pub filtered_indices: Vec<usize>,
    /// Whether the filtered_indices cache is valid.
    /// Set to false when filters or underlying data changes.
    pub cache_valid: bool,
}

impl MappingUiState {
    /// Invalidate the filter cache. Call when filters or underlying data changes.
    #[inline]
    pub fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }
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
    /// Cached filtered issue indices (for performance).
    /// Contains indices into the validation report's issues vec that pass the current filter.
    pub filtered_indices: Vec<usize>,
    /// Whether the filtered_indices cache is valid.
    /// Set to false when filter or validation results change.
    pub cache_valid: bool,
}

impl ValidationUiState {
    /// Invalidate the filter cache. Call when filter or validation results change.
    #[inline]
    pub fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }
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

impl PreviewUiState {
    /// Create with a custom rows per page value.
    pub fn with_rows_per_page(rows: usize) -> Self {
        Self {
            rows_per_page: rows,
            ..Default::default()
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
    /// Cached filtered column names (for performance).
    /// Contains unmapped column names that pass the current filters.
    pub filtered_columns: Vec<String>,
    /// Whether the filtered_columns cache is valid.
    /// Set to false when filters or underlying data changes.
    pub cache_valid: bool,
}

impl SuppUiState {
    /// Invalidate the filter cache. Call when filters or underlying data changes.
    #[inline]
    pub fn invalidate_cache(&mut self) {
        self.cache_valid = false;
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
// SOURCE ASSIGNMENT STATE
// =============================================================================

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

/// Status of a source file in the assignment workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceFileStatus {
    /// File is unassigned (ready to be assigned to a domain).
    #[default]
    Unassigned,
    /// File is marked as metadata (e.g., Items.csv).
    Metadata,
    /// File is explicitly skipped.
    Skipped,
}

/// A source file entry in the assignment screen.
#[derive(Debug, Clone)]
pub struct SourceFileEntry {
    /// Full path to the CSV file.
    pub path: PathBuf,
    /// Filename without extension (for display).
    pub file_stem: String,
    /// Current status of this file.
    pub status: SourceFileStatus,
    /// Domain code this file is assigned to (if any).
    pub assigned_domain: Option<String>,
}

impl SourceFileEntry {
    /// Create a new source file entry from a path.
    pub fn new(path: PathBuf) -> Self {
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        Self {
            path,
            file_stem,
            status: SourceFileStatus::default(),
            assigned_domain: None,
        }
    }

    /// Check if this file is available for assignment (not metadata, not skipped, not assigned).
    pub fn is_available(&self) -> bool {
        self.status == SourceFileStatus::Unassigned && self.assigned_domain.is_none()
    }

    /// Check if this file needs action (not categorized yet).
    pub fn needs_action(&self) -> bool {
        self.status == SourceFileStatus::Unassigned && self.assigned_domain.is_none()
    }
}

/// A target domain entry for the assignment screen.
#[derive(Debug, Clone)]
pub struct TargetDomainEntry {
    /// Domain code (e.g., "DM", "AE").
    pub code: String,
    /// Human-readable label (e.g., "Demographics").
    pub label: Option<String>,
    /// Domain class (e.g., "Special-Purpose", "Events").
    pub class: Option<String>,
    /// Description for tooltip.
    pub description: Option<String>,
}

impl TargetDomainEntry {
    /// Create a new target domain entry.
    pub fn new(code: String, label: Option<String>, class: Option<String>) -> Self {
        Self {
            code,
            label,
            class,
            description: None,
        }
    }

    /// Create with description.
    pub fn with_description(mut self, desc: Option<String>) -> Self {
        self.description = desc;
        self
    }

    /// Get display name (label or code).
    pub fn display_name(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.code)
    }
}

/// UI state for the source assignment screen.
#[derive(Debug, Clone, Default)]
pub struct SourceAssignmentUiState {
    /// Study folder path.
    pub folder: PathBuf,
    /// List of source CSV files.
    pub source_files: Vec<SourceFileEntry>,
    /// List of target domains (from standards).
    pub target_domains: Vec<TargetDomainEntry>,
    /// Domains grouped by class for display.
    pub domains_by_class: BTreeMap<String, Vec<String>>,
    /// Search filter for source files.
    pub source_search: String,
    /// Search filter for domains.
    pub domain_search: String,
    /// Currently selected file index (for click-to-assign mode).
    pub selected_file: Option<usize>,
    /// Currently dragging file index (for drag-and-drop mode).
    pub dragging_file: Option<usize>,
    /// Domain being hovered over during drag.
    pub hover_domain: Option<String>,
    /// Whether study creation is in progress.
    pub is_creating_study: bool,
}

impl SourceAssignmentUiState {
    /// Create new source assignment UI state.
    pub fn new(
        folder: PathBuf,
        source_files: Vec<PathBuf>,
        target_domains: Vec<TargetDomainEntry>,
    ) -> Self {
        // Group domains by class
        let mut domains_by_class: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for domain in &target_domains {
            let class = domain.class.clone().unwrap_or_else(|| "Other".to_string());
            domains_by_class
                .entry(class)
                .or_default()
                .push(domain.code.clone());
        }

        Self {
            folder,
            source_files: source_files.into_iter().map(SourceFileEntry::new).collect(),
            target_domains,
            domains_by_class,
            source_search: String::new(),
            domain_search: String::new(),
            selected_file: None,
            dragging_file: None,
            hover_domain: None,
            is_creating_study: false,
        }
    }

    /// Get the number of assigned files.
    pub fn assigned_count(&self) -> usize {
        self.source_files
            .iter()
            .filter(|f| f.assigned_domain.is_some())
            .count()
    }

    /// Get the number of metadata files.
    pub fn metadata_count(&self) -> usize {
        self.source_files
            .iter()
            .filter(|f| f.status == SourceFileStatus::Metadata)
            .count()
    }

    /// Get the number of skipped files.
    pub fn skipped_count(&self) -> usize {
        self.source_files
            .iter()
            .filter(|f| f.status == SourceFileStatus::Skipped)
            .count()
    }

    /// Get the number of remaining files (need action).
    pub fn remaining_count(&self) -> usize {
        self.source_files
            .iter()
            .filter(|f| f.needs_action())
            .count()
    }

    /// Check if all files have been categorized.
    pub fn all_categorized(&self) -> bool {
        self.remaining_count() == 0
    }

    /// Get files assigned to a specific domain.
    pub fn files_for_domain(&self, domain_code: &str) -> Vec<&SourceFileEntry> {
        self.source_files
            .iter()
            .filter(|f| f.assigned_domain.as_deref() == Some(domain_code))
            .collect()
    }

    /// Get assignments as a map (domain_code -> file_path).
    pub fn get_assignments(&self) -> BTreeMap<String, PathBuf> {
        self.source_files
            .iter()
            .filter_map(|f| {
                f.assigned_domain
                    .as_ref()
                    .map(|d| (d.clone(), f.path.clone()))
            })
            .collect()
    }

    /// Get metadata file paths.
    pub fn get_metadata_files(&self) -> Vec<PathBuf> {
        self.source_files
            .iter()
            .filter(|f| f.status == SourceFileStatus::Metadata)
            .map(|f| f.path.clone())
            .collect()
    }

    /// Filter source files by search term.
    ///
    /// Returns only files that are available for assignment (not marked, not assigned)
    /// and match the search term.
    pub fn filtered_source_files(&self) -> Vec<(usize, &SourceFileEntry)> {
        let search = self.source_search.to_lowercase();
        self.source_files
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                // Only show files available for assignment that match search
                f.is_available()
                    && (search.is_empty() || f.file_stem.to_lowercase().contains(&search))
            })
            .collect()
    }

    /// Filter domains by search term.
    pub fn filtered_domains(&self) -> Vec<&TargetDomainEntry> {
        let search = self.domain_search.to_lowercase();
        self.target_domains
            .iter()
            .filter(|d| {
                search.is_empty()
                    || d.code.to_lowercase().contains(&search)
                    || d.label
                        .as_ref()
                        .is_some_and(|l| l.to_lowercase().contains(&search))
            })
            .collect()
    }

    /// Get marked (metadata/skipped) files.
    pub fn marked_files(&self) -> Vec<(usize, &SourceFileEntry)> {
        self.source_files
            .iter()
            .enumerate()
            .filter(|(_, f)| f.status != SourceFileStatus::Unassigned)
            .collect()
    }
}

// =============================================================================
// EXPORT VIEW STATE
// =============================================================================

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
    },
    /// Complete - export finished (result stored in dialog_registry).
    Complete,
}

impl ExportPhase {
    /// Check if export is in progress.
    pub fn is_exporting(&self) -> bool {
        matches!(self, Self::Exporting { .. })
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
