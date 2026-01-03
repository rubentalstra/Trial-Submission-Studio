//! UI state - completely separated from domain data.
//!
//! This module contains all UI-related state (selection indices,
//! pagination, search filters) that was previously scattered
//! throughout domain state.

use crate::export::ExportUiState;
use crate::settings::Settings;
use crate::state::derived_state::QualifierOrigin;
use std::collections::HashMap;

// ============================================================================
// Top-Level UI State
// ============================================================================

/// All UI state in one place - never mixed with domain data.
#[derive(Default)]
pub struct UiState {
    /// Settings window UI state
    pub settings: SettingsUiState,
    /// Export screen UI state
    pub export: ExportUiState,
    /// Per-domain editor UI state
    pub domain_editors: HashMap<String, DomainEditorUiState>,
    /// Close study confirmation modal
    pub close_study_confirm: bool,
    /// Update dialog UI state
    pub update: UpdateDialogState,
    /// About dialog UI state
    pub about: AboutUiState,
}

impl UiState {
    /// Get or create UI state for a domain editor.
    pub fn domain_editor(&mut self, code: &str) -> &mut DomainEditorUiState {
        self.domain_editors
            .entry(code.to_string())
            .or_insert_with(DomainEditorUiState::default)
    }

    /// Get existing UI state for a domain (immutable).
    pub fn get_domain_editor(&self, code: &str) -> Option<&DomainEditorUiState> {
        self.domain_editors.get(code)
    }

    /// Clear all domain editor UI state (e.g., when loading a new study).
    pub fn clear_domain_editors(&mut self) {
        self.domain_editors.clear();
    }
}

// ============================================================================
// Domain Editor UI State
// ============================================================================

/// UI state for the domain editor (per domain).
#[derive(Debug, Clone, Default)]
pub struct DomainEditorUiState {
    /// Mapping tab UI state
    pub mapping: MappingUiState,
    /// Transform tab UI state
    pub transform: TransformUiState,
    /// Validation tab UI state
    pub validation: ValidationUiState,
    /// Preview tab UI state
    pub preview: PreviewUiState,
    /// SUPP tab UI state
    pub supp: SuppUiState,
}

// ============================================================================
// Mapping Tab UI State
// ============================================================================

/// UI state for the Mapping tab.
#[derive(Debug, Clone, Default)]
pub struct MappingUiState {
    /// Selected variable index in the list
    pub selected_idx: Option<usize>,
    /// Search filter text
    pub search_filter: String,
    /// "Not collected" reason being edited (variable name -> text)
    pub not_collected_editing: HashMap<String, String>,
}

impl MappingUiState {
    /// Set the selected variable index.
    pub fn select(&mut self, idx: Option<usize>) {
        self.selected_idx = idx;
    }

    /// Set the reason being edited for a variable.
    pub fn set_editing_reason(&mut self, var_name: &str, reason: &str) {
        self.not_collected_editing
            .insert(var_name.to_string(), reason.to_string());
    }

    /// Remove editing reason (e.g., after save).
    pub fn clear_editing_reason(&mut self, var_name: &str) {
        self.not_collected_editing.remove(var_name);
    }
}

// ============================================================================
// Transform Tab UI State
// ============================================================================

/// UI state for the Transform tab.
#[derive(Debug, Clone, Default)]
pub struct TransformUiState {
    /// Selected transform rule index
    pub selected_idx: Option<usize>,
}

impl TransformUiState {
    /// Set the selected transform rule index.
    pub fn select(&mut self, idx: Option<usize>) {
        self.selected_idx = idx;
    }
}

// ============================================================================
// Validation Tab UI State
// ============================================================================

/// UI state for the Validation tab.
#[derive(Debug, Clone, Default)]
pub struct ValidationUiState {
    /// Selected issue index
    pub selected_idx: Option<usize>,
}

impl ValidationUiState {
    /// Set the selected issue index.
    pub fn select(&mut self, idx: Option<usize>) {
        self.selected_idx = idx;
    }
}

// ============================================================================
// Preview Tab UI State
// ============================================================================

/// UI state for the Preview tab.
#[derive(Debug, Clone)]
pub struct PreviewUiState {
    /// Current page (0-indexed)
    pub current_page: usize,
    /// Rows per page
    pub rows_per_page: usize,
    /// Error message if preview generation failed
    pub error: Option<String>,
    /// Whether preview is currently being rebuilt in background
    pub is_rebuilding: bool,
}

impl Default for PreviewUiState {
    fn default() -> Self {
        Self {
            current_page: 0,
            rows_per_page: 50,
            error: None,
            is_rebuilding: false,
        }
    }
}

impl PreviewUiState {
    /// Go to the next page.
    pub fn next_page(&mut self) {
        self.current_page += 1;
    }

    /// Go to the previous page.
    pub fn prev_page(&mut self) {
        self.current_page = self.current_page.saturating_sub(1);
    }
}

// ============================================================================
// SUPP Tab UI State
// ============================================================================

/// UI state for the SUPP tab.
#[derive(Debug, Clone, Default)]
pub struct SuppUiState {
    /// Selected column for detail view
    pub selected_column: Option<String>,
    /// Editing state for the selected column
    pub editing: Option<SuppEditingState>,
}

/// Editing state for a SUPP column configuration.
#[derive(Debug, Clone, Default)]
pub struct SuppEditingState {
    /// The column being edited
    pub column_name: String,
    /// Pending QNAM value
    pub qnam: String,
    /// Pending QLABEL value
    pub qlabel: String,
    /// Pending QORIG value
    pub qorig: QualifierOrigin,
    /// Pending QEVAL value
    pub qeval: String,
}

impl SuppUiState {
    /// Set the selected column for detail view.
    pub fn select(&mut self, column: Option<String>) {
        self.selected_column = column;
        // Clear editing state when selection changes
        self.editing = None;
    }

    /// Start editing a column with initial values.
    pub fn start_editing(
        &mut self,
        column_name: &str,
        qnam: &str,
        qlabel: &str,
        qorig: QualifierOrigin,
        qeval: &str,
    ) {
        self.editing = Some(SuppEditingState {
            column_name: column_name.to_string(),
            qnam: qnam.to_string(),
            qlabel: qlabel.to_string(),
            qorig,
            qeval: qeval.to_string(),
        });
    }

    /// Cancel editing.
    pub fn cancel_editing(&mut self) {
        self.editing = None;
    }

    /// Get editing state if it matches the given column.
    pub fn editing_for(&self, column_name: &str) -> Option<&SuppEditingState> {
        self.editing
            .as_ref()
            .filter(|e| e.column_name == column_name)
    }

    /// Get mutable editing state if it matches the given column.
    pub fn editing_for_mut(&mut self, column_name: &str) -> Option<&mut SuppEditingState> {
        self.editing
            .as_mut()
            .filter(|e| e.column_name == column_name)
    }
}

// ============================================================================
// Settings UI State
// ============================================================================

/// UI state for the Settings window.
#[derive(Debug, Clone, Default)]
pub struct SettingsUiState {
    /// Is settings window open
    pub open: bool,
    /// Pending settings (working copy for editing)
    pub pending: Option<Settings>,
}

impl SettingsUiState {
    /// Open the settings window with a copy of current settings.
    pub fn open(&mut self, current: &Settings) {
        self.pending = Some(current.clone());
        self.open = true;
    }

    /// Close the settings window, optionally returning the pending settings.
    pub fn close(&mut self, apply: bool) -> Option<Settings> {
        self.open = false;
        if apply {
            self.pending.take()
        } else {
            self.pending = None;
            None
        }
    }

    /// Check if settings window is open.
    pub fn is_open(&self) -> bool {
        self.open
    }
}

// Note: ExportUiState is now in crate::export::types

// ============================================================================
// Update Dialog State
// ============================================================================

/// Single source of truth for update dialog state.
///
/// This enum replaces the previous `UpdateUiState` struct with phase + separate fields.
/// Each variant contains exactly the data needed for that state.
#[derive(Debug, Clone, Default)]
pub enum UpdateDialogState {
    /// Dialog is closed.
    #[default]
    Closed,
    /// Checking for updates (shows spinner).
    Checking,
    /// No update available (current version is latest).
    NoUpdate,
    /// Update is available.
    UpdateAvailable {
        /// The new version string.
        version: String,
        /// Changelog/release notes in markdown.
        changelog: String,
    },
    /// Installing update (download + extract + replace).
    Installing,
    /// An error occurred.
    Error(String),
}

impl UpdateDialogState {
    /// Check if the dialog should be displayed.
    #[must_use]
    pub fn is_open(&self) -> bool {
        !matches!(self, Self::Closed)
    }

    /// Close the dialog (reset to Closed state).
    pub fn close(&mut self) {
        *self = Self::Closed;
    }
}

// ============================================================================
// About Dialog UI State
// ============================================================================

/// UI state for the About dialog.
#[derive(Debug, Clone, Default)]
pub struct AboutUiState {
    /// Is the About dialog open.
    pub open: bool,
}

impl AboutUiState {
    /// Open the About dialog.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the About dialog.
    pub fn close(&mut self) {
        self.open = false;
    }
}
