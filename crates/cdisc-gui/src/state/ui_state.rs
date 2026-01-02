//! UI state - completely separated from domain data.
//!
//! This module contains all UI-related state (selection indices,
//! pagination, search filters) that was previously scattered
//! throughout domain state.

use crate::settings::{ExportFormat, Settings};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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
}

impl Default for PreviewUiState {
    fn default() -> Self {
        Self {
            current_page: 0,
            rows_per_page: 50,
            error: None,
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

    /// Reset to first page.
    pub fn reset(&mut self) {
        self.current_page = 0;
        self.error = None;
    }

    /// Calculate total pages for a given row count.
    #[allow(dead_code)]
    pub fn total_pages(&self, total_rows: usize) -> usize {
        (total_rows + self.rows_per_page - 1).max(1) / self.rows_per_page.max(1)
    }

    /// Get the range of rows for the current page.
    #[allow(dead_code)]
    pub fn row_range(&self, total_rows: usize) -> (usize, usize) {
        let start = self.current_page * self.rows_per_page;
        let end = (start + self.rows_per_page).min(total_rows);
        (start, end)
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
}

impl SuppUiState {
    /// Set the selected column for detail view.
    pub fn select(&mut self, column: Option<String>) {
        self.selected_column = column;
        // Clear editing state when selection changes
        self.editing = None;
    }

    /// Start editing a column with initial values.
    pub fn start_editing(&mut self, column_name: &str, qnam: &str, qlabel: &str) {
        self.editing = Some(SuppEditingState {
            column_name: column_name.to_string(),
            qnam: qnam.to_string(),
            qlabel: qlabel.to_string(),
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

// ============================================================================
// Export UI State
// ============================================================================

/// UI state for the Export screen.
#[derive(Debug, Clone, Default)]
pub struct ExportUiState {
    /// Domains selected for export (domain codes)
    pub selected_domains: HashSet<String>,
    /// Override output directory (if different from default)
    pub output_dir_override: Option<PathBuf>,
    /// Override export format (if different from settings default)
    pub format_override: Option<ExportFormat>,
    /// Current export progress (None if not exporting)
    pub progress: Option<ExportProgress>,
}

impl ExportUiState {
    /// Toggle domain selection.
    pub fn toggle_domain(&mut self, code: &str) {
        if self.selected_domains.contains(code) {
            self.selected_domains.remove(code);
        } else {
            self.selected_domains.insert(code.to_string());
        }
    }

    /// Select all domains from a list.
    pub fn select_all(&mut self, codes: impl IntoIterator<Item = impl AsRef<str>>) {
        for code in codes {
            self.selected_domains.insert(code.as_ref().to_string());
        }
    }

    /// Deselect all domains.
    pub fn deselect_all(&mut self) {
        self.selected_domains.clear();
    }

    /// Get the number of selected domains.
    pub fn selection_count(&self) -> usize {
        self.selected_domains.len()
    }

    /// Reset export state (e.g., when loading a new study).
    pub fn reset(&mut self) {
        self.selected_domains.clear();
        self.output_dir_override = None;
        self.format_override = None;
        self.progress = None;
    }
}

// ============================================================================
// Export Progress
// ============================================================================

/// Export progress tracking.
#[derive(Debug, Clone)]
pub struct ExportProgress {
    /// Current step description
    pub current_step: String,
    /// Total domains to process
    pub total_domains: usize,
    /// Completed domains count
    pub completed_domains: usize,
    /// Individual step within current domain
    pub domain_step: ExportDomainStep,
    /// Any error that occurred
    pub error: Option<String>,
    /// Export completed successfully
    pub completed: bool,
}

impl ExportProgress {
    /// Get overall progress as a fraction (0.0 to 1.0).
    pub fn fraction(&self) -> f32 {
        if self.total_domains == 0 {
            return 1.0;
        }
        let domain_fraction = self.completed_domains as f32 / self.total_domains as f32;
        let step_fraction = match self.domain_step {
            ExportDomainStep::Pending => 0.0,
            ExportDomainStep::ApplyingMappings => 0.15,
            ExportDomainStep::NormalizingCT => 0.30,
            ExportDomainStep::GeneratingVariables => 0.45,
            ExportDomainStep::ValidatingOutput => 0.60,
            ExportDomainStep::WritingXpt => 0.80,
            ExportDomainStep::WritingDefineXml => 0.95,
            ExportDomainStep::Complete => 1.0,
        };
        // Each domain contributes equally to overall progress
        let per_domain = 1.0 / self.total_domains as f32;
        domain_fraction + (step_fraction * per_domain)
    }

    /// Check if export is in progress.
    pub fn is_in_progress(&self) -> bool {
        !self.completed && self.error.is_none()
    }
}

/// Steps within a domain export.
/// Note: Variants are used in fraction() calculation even if not constructed yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum ExportDomainStep {
    #[default]
    Pending,
    ApplyingMappings,
    NormalizingCT,
    GeneratingVariables,
    ValidatingOutput,
    WritingXpt,
    WritingDefineXml,
    Complete,
}
