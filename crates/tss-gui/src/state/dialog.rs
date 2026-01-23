//! Dialog state management with a unified registry.
//!
//! This module provides a cleaner approach to managing dialog windows in Iced's
//! multi-window mode. The `DialogRegistry` replaces the previous pattern of
//! individual optional fields with a unified HashMap-based approach.

use std::collections::HashMap;

use iced::window;

use crate::message::SettingsCategory;
use crate::view::dialog::third_party::ThirdPartyState;
use crate::view::dialog::update::UpdateState;

// Re-use ExportResult from view_state
use super::view_state::ExportResult;

/// State for export progress dialog.
#[derive(Debug, Clone)]
pub struct ExportProgressState {
    /// Current domain being processed.
    pub current_domain: Option<String>,
    /// Current step label.
    pub current_step: String,
    /// Progress 0.0 to 1.0.
    pub progress: f32,
    /// Number of files written.
    pub files_written: usize,
}

impl Default for ExportProgressState {
    fn default() -> Self {
        Self {
            current_domain: None,
            current_step: "Preparing...".to_string(),
            progress: 0.0,
            files_written: 0,
        }
    }
}

/// Dialog type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogType {
    /// About dialog showing application info.
    About,
    /// Settings configuration dialog.
    Settings,
    /// Third-party licenses dialog.
    ThirdParty,
    /// Update availability dialog.
    Update,
    /// Close project confirmation dialog.
    CloseProjectConfirm,
    /// Export progress indicator dialog.
    ExportProgress,
    /// Export completion dialog.
    ExportComplete,
}

impl DialogType {
    /// Get the title for this dialog type.
    pub fn title(&self) -> &'static str {
        match self {
            Self::About => "About Trial Submission Studio",
            Self::Settings => "Settings",
            Self::ThirdParty => "Third-Party Licenses",
            Self::Update => "Software Update",
            Self::CloseProjectConfirm => "Close Project",
            Self::ExportProgress => "Exporting...",
            Self::ExportComplete => "Export Complete",
        }
    }

    /// Get the default window size for this dialog type.
    pub fn default_size(&self) -> (f32, f32) {
        match self {
            Self::About => (400.0, 320.0),
            Self::Settings => (700.0, 500.0),
            Self::ThirdParty => (700.0, 550.0),
            Self::Update => (420.0, 300.0),
            Self::CloseProjectConfirm => (400.0, 180.0),
            Self::ExportProgress => (380.0, 180.0),
            Self::ExportComplete => (400.0, 240.0),
        }
    }

    /// Check if this dialog should be resizable.
    pub fn is_resizable(&self) -> bool {
        matches!(self, Self::Settings | Self::ThirdParty)
    }
}

/// State stored for each dialog type.
#[derive(Debug, Clone)]
pub enum DialogState {
    /// About dialog (no additional state).
    About,
    /// Settings dialog with selected category.
    Settings(SettingsCategory),
    /// Third-party licenses dialog with scroll state.
    ThirdParty(ThirdPartyState),
    /// Update dialog with version info.
    Update(UpdateState),
    /// Close project confirmation (no additional state).
    CloseProjectConfirm,
    /// Export progress with progress info.
    ExportProgress(ExportProgressState),
    /// Export complete with results.
    ExportComplete(ExportResult),
}

impl DialogState {
    /// Get the dialog type for this state.
    pub fn dialog_type(&self) -> DialogType {
        match self {
            Self::About => DialogType::About,
            Self::Settings(_) => DialogType::Settings,
            Self::ThirdParty(_) => DialogType::ThirdParty,
            Self::Update(_) => DialogType::Update,
            Self::CloseProjectConfirm => DialogType::CloseProjectConfirm,
            Self::ExportProgress(_) => DialogType::ExportProgress,
            Self::ExportComplete(_) => DialogType::ExportComplete,
        }
    }
}

/// Unified registry for managing dialog windows.
///
/// This replaces the previous pattern of individual optional fields with a
/// HashMap-based approach that provides:
/// - Single source of truth for dialog state
/// - Type-safe state access via `get_*` methods
/// - Centralized open/close logic
#[derive(Debug, Clone, Default)]
pub struct DialogRegistry {
    /// Map from window ID to dialog state.
    dialogs: HashMap<window::Id, DialogState>,
    /// Reverse map from dialog type to window ID (for singleton dialogs).
    type_to_id: HashMap<DialogType, window::Id>,
}

impl DialogRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new dialog window.
    ///
    /// If a dialog of this type already exists, it will be replaced.
    pub fn register(&mut self, id: window::Id, state: DialogState) {
        let dialog_type = state.dialog_type();

        // Close existing dialog of same type if any
        if let Some(old_id) = self.type_to_id.remove(&dialog_type) {
            self.dialogs.remove(&old_id);
        }

        self.dialogs.insert(id, state);
        self.type_to_id.insert(dialog_type, id);
    }

    /// Close a dialog by window ID.
    ///
    /// Returns the dialog state if it was found.
    pub fn close(&mut self, id: window::Id) -> Option<DialogState> {
        if let Some(state) = self.dialogs.remove(&id) {
            self.type_to_id.remove(&state.dialog_type());
            Some(state)
        } else {
            None
        }
    }

    /// Close a dialog by type.
    ///
    /// Returns the window ID and state if it was found.
    pub fn close_by_type(&mut self, dtype: DialogType) -> Option<(window::Id, DialogState)> {
        if let Some(id) = self.type_to_id.remove(&dtype) {
            let state = self.dialogs.remove(&id)?;
            Some((id, state))
        } else {
            None
        }
    }

    /// Check if a dialog of the given type is open.
    pub fn is_open(&self, dtype: DialogType) -> bool {
        self.type_to_id.contains_key(&dtype)
    }

    /// Get the window ID for a dialog type.
    pub fn window_id(&self, dtype: DialogType) -> Option<window::Id> {
        self.type_to_id.get(&dtype).copied()
    }

    /// Check if a window ID belongs to any dialog.
    pub fn is_dialog_window(&self, id: window::Id) -> bool {
        self.dialogs.contains_key(&id)
    }

    /// Get the dialog type for a window ID.
    pub fn dialog_type(&self, id: window::Id) -> Option<DialogType> {
        self.dialogs.get(&id).map(DialogState::dialog_type)
    }

    /// Get the dialog state for a window ID.
    pub fn get(&self, id: window::Id) -> Option<&DialogState> {
        self.dialogs.get(&id)
    }

    /// Get mutable dialog state for a window ID.
    pub fn get_mut(&mut self, id: window::Id) -> Option<&mut DialogState> {
        self.dialogs.get_mut(&id)
    }

    /// Get all open dialog window IDs.
    pub fn window_ids(&self) -> impl Iterator<Item = window::Id> + '_ {
        self.dialogs.keys().copied()
    }

    // =========================================================================
    // TYPE-SAFE ACCESSORS
    // =========================================================================

    /// Get settings dialog state.
    pub fn settings(&self) -> Option<(window::Id, &SettingsCategory)> {
        let id = *self.type_to_id.get(&DialogType::Settings)?;
        if let Some(DialogState::Settings(cat)) = self.dialogs.get(&id) {
            Some((id, cat))
        } else {
            None
        }
    }

    /// Get mutable settings dialog state.
    pub fn settings_mut(&mut self) -> Option<(window::Id, &mut SettingsCategory)> {
        let id = *self.type_to_id.get(&DialogType::Settings)?;
        if let Some(DialogState::Settings(cat)) = self.dialogs.get_mut(&id) {
            Some((id, cat))
        } else {
            None
        }
    }

    /// Get third-party dialog state.
    pub fn third_party(&self) -> Option<(window::Id, &ThirdPartyState)> {
        let id = *self.type_to_id.get(&DialogType::ThirdParty)?;
        if let Some(DialogState::ThirdParty(state)) = self.dialogs.get(&id) {
            Some((id, state))
        } else {
            None
        }
    }

    /// Get mutable third-party dialog state.
    pub fn third_party_mut(&mut self) -> Option<(window::Id, &mut ThirdPartyState)> {
        let id = *self.type_to_id.get(&DialogType::ThirdParty)?;
        if let Some(DialogState::ThirdParty(state)) = self.dialogs.get_mut(&id) {
            Some((id, state))
        } else {
            None
        }
    }

    /// Get update dialog state.
    pub fn update(&self) -> Option<(window::Id, &UpdateState)> {
        let id = *self.type_to_id.get(&DialogType::Update)?;
        if let Some(DialogState::Update(state)) = self.dialogs.get(&id) {
            Some((id, state))
        } else {
            None
        }
    }

    /// Get mutable update dialog state.
    pub fn update_mut(&mut self) -> Option<(window::Id, &mut UpdateState)> {
        let id = *self.type_to_id.get(&DialogType::Update)?;
        if let Some(DialogState::Update(state)) = self.dialogs.get_mut(&id) {
            Some((id, state))
        } else {
            None
        }
    }

    /// Get export progress dialog state.
    pub fn export_progress(&self) -> Option<(window::Id, &ExportProgressState)> {
        let id = *self.type_to_id.get(&DialogType::ExportProgress)?;
        if let Some(DialogState::ExportProgress(state)) = self.dialogs.get(&id) {
            Some((id, state))
        } else {
            None
        }
    }

    /// Get mutable export progress dialog state.
    pub fn export_progress_mut(&mut self) -> Option<(window::Id, &mut ExportProgressState)> {
        let id = *self.type_to_id.get(&DialogType::ExportProgress)?;
        if let Some(DialogState::ExportProgress(state)) = self.dialogs.get_mut(&id) {
            Some((id, state))
        } else {
            None
        }
    }

    /// Get export complete dialog state.
    pub fn export_complete(&self) -> Option<(window::Id, &ExportResult)> {
        let id = *self.type_to_id.get(&DialogType::ExportComplete)?;
        if let Some(DialogState::ExportComplete(result)) = self.dialogs.get(&id) {
            Some((id, result))
        } else {
            None
        }
    }
}
