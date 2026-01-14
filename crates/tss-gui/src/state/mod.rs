//! State management for Trial Submission Studio (Iced Architecture).
//!
//! # Architecture Overview
//!
//! This module follows Iced's Elm architecture principles:
//!
//! - **State is the single source of truth**
//! - **All state changes happen through messages in `update()`**
//! - **Views are pure functions of state**
//! - **Async operations use `Task`, not channels**
//!
//! ## Module Structure
//!
//! - [`AppState`] - Root state container (flat, no nesting complexity)
//! - [`Study`] - Loaded study with domains (domain data)
//! - [`DomainState`] - Single SDTM domain with source data and mapping
//! - [`ViewState`] - Current view and its UI state
//! - [`Settings`] - Persisted user preferences
//!
//! ## Key Design Decisions
//!
//! 1. **No channels/polling** - Iced's `Task` system handles async
//! 2. **No mutable accessors** - State changes only in `update()`
//! 3. **Derived data computed on demand** - Not cached in state
//! 4. **UI state lives with views** - Not mixed with domain data

mod domain_state;
mod settings;
mod study;
mod view_state;

// Re-exports
pub use domain_state::{DomainSource, DomainState, SuppAction, SuppColumnConfig, SuppOrigin};
pub use settings::{
    DeveloperSettings, ExportFormat, ExportSettings, GeneralSettings, Settings, XptVersion,
};
pub use study::Study;
pub use view_state::{
    EditorTab, ExportPhase, ExportResult, ExportViewState, MappingUiState, NormalizationUiState,
    NotCollectedEdit, PreviewUiState, SeverityFilter, SuppEditDraft, SuppFilterMode, SuppUiState,
    ValidationUiState, ViewState, WorkflowMode,
};

use crate::menu::MenuBarState;
use iced::window;
use tss_standards::TerminologyRegistry;

// =============================================================================
// ROOT APPLICATION STATE
// =============================================================================

/// Root application state.
///
/// This is intentionally flat and simple. All state changes happen through
/// message handling in `App::update()`.
///
/// # Example
///
/// ```ignore
/// // In App::update()
/// Message::StudyLoaded(result) => {
///     match result {
///         Ok(study) => {
///             self.state.study = Some(study);
///             self.state.view = ViewState::home();
///         }
///         Err(e) => {
///             self.state.error = Some(e);
///         }
///     }
///     Task::none()
/// }
/// ```
#[derive(Default)]
pub struct AppState {
    /// Current view and its associated UI state.
    ///
    /// This determines what's rendered and holds view-specific state
    /// (selections, pagination, filters, etc.)
    pub view: ViewState,

    /// Loaded study data.
    ///
    /// `None` when no study is open. Contains all domain data and mappings.
    pub study: Option<Study>,

    /// User settings (persisted to disk).
    pub settings: Settings,

    /// CDISC Controlled Terminology registry.
    ///
    /// Loaded once when first study is opened. Used for validation
    /// and controlled term lookups.
    pub terminology: Option<TerminologyRegistry>,

    /// Current error message to display (transient).
    ///
    /// Set when an operation fails, cleared on user acknowledgment.
    pub error: Option<String>,

    /// Whether a background task is running (for UI feedback).
    pub is_loading: bool,

    /// Menu bar state (for in-app menu on Windows/Linux).
    pub menu_bar: MenuBarState,

    /// Tracks open dialog windows (multi-window mode).
    pub dialog_windows: DialogWindows,

    /// Main window ID (for identifying the main window in multi-window mode).
    pub main_window_id: Option<window::Id>,
}

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

/// Tracks open dialog windows.
#[derive(Debug, Clone, Default)]
pub struct DialogWindows {
    /// About dialog window ID.
    pub about: Option<window::Id>,
    /// Settings dialog window ID.
    pub settings: Option<(window::Id, crate::message::SettingsCategory)>,
    /// Third-party licenses dialog window ID.
    pub third_party: Option<window::Id>,
    /// Update dialog window ID and state.
    pub update: Option<(window::Id, crate::view::dialog::update::UpdateState)>,
    /// Close study confirmation dialog window ID.
    pub close_study_confirm: Option<window::Id>,
    /// Export progress dialog window ID and state.
    pub export_progress: Option<(window::Id, ExportProgressState)>,
    /// Export completion dialog window ID and result.
    pub export_complete: Option<(window::Id, ExportResult)>,
}

impl DialogWindows {
    /// Check if a window ID belongs to any dialog.
    pub fn is_dialog_window(&self, id: window::Id) -> bool {
        self.about == Some(id)
            || self.settings.as_ref().map(|(i, _)| *i) == Some(id)
            || self.third_party == Some(id)
            || self.update.as_ref().map(|(i, _)| *i) == Some(id)
            || self.close_study_confirm == Some(id)
            || self.export_progress.as_ref().map(|(i, _)| *i) == Some(id)
            || self.export_complete.as_ref().map(|(i, _)| *i) == Some(id)
    }

    /// Get the dialog type for a window ID.
    pub fn dialog_type(&self, id: window::Id) -> Option<DialogType> {
        if self.about == Some(id) {
            Some(DialogType::About)
        } else if self.settings.as_ref().map(|(i, _)| *i) == Some(id) {
            Some(DialogType::Settings)
        } else if self.third_party == Some(id) {
            Some(DialogType::ThirdParty)
        } else if self.update.as_ref().map(|(i, _)| *i) == Some(id) {
            Some(DialogType::Update)
        } else if self.close_study_confirm == Some(id) {
            Some(DialogType::CloseStudyConfirm)
        } else if self.export_progress.as_ref().map(|(i, _)| *i) == Some(id) {
            Some(DialogType::ExportProgress)
        } else if self.export_complete.as_ref().map(|(i, _)| *i) == Some(id) {
            Some(DialogType::ExportComplete)
        } else {
            None
        }
    }

    /// Close a dialog window by ID.
    pub fn close(&mut self, id: window::Id) {
        if self.about == Some(id) {
            self.about = None;
        } else if self.settings.as_ref().map(|(i, _)| *i) == Some(id) {
            self.settings = None;
        } else if self.third_party == Some(id) {
            self.third_party = None;
        } else if self.update.as_ref().map(|(i, _)| *i) == Some(id) {
            self.update = None;
        } else if self.close_study_confirm == Some(id) {
            self.close_study_confirm = None;
        } else if self.export_progress.as_ref().map(|(i, _)| *i) == Some(id) {
            self.export_progress = None;
        } else if self.export_complete.as_ref().map(|(i, _)| *i) == Some(id) {
            self.export_complete = None;
        }
    }
}

/// Dialog type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    About,
    Settings,
    ThirdParty,
    Update,
    CloseStudyConfirm,
    ExportProgress,
    ExportComplete,
}

impl AppState {
    /// Create app state with loaded settings.
    pub fn with_settings(settings: Settings) -> Self {
        Self {
            view: ViewState::default(),
            study: None,
            settings,
            terminology: None,
            error: None,
            is_loading: false,
            menu_bar: MenuBarState::default(),
            dialog_windows: DialogWindows::default(),
            main_window_id: None,
        }
    }

    // =========================================================================
    // READ-ONLY ACCESSORS (no mutation - that happens in update())
    // =========================================================================

    /// Get study reference.
    #[inline]
    pub fn study(&self) -> Option<&Study> {
        self.study.as_ref()
    }

    /// Get domain by code.
    #[inline]
    pub fn domain(&self, code: &str) -> Option<&DomainState> {
        self.study.as_ref()?.domain(code)
    }

    /// Check if a study is loaded.
    #[inline]
    pub fn has_study(&self) -> bool {
        self.study.is_some()
    }

    /// Get domain codes (for iteration).
    pub fn domain_codes(&self) -> Vec<&str> {
        self.study
            .as_ref()
            .map(|s| s.domain_codes())
            .unwrap_or_default()
    }

    /// Get current workflow mode from view state.
    #[inline]
    pub fn workflow_mode(&self) -> WorkflowMode {
        self.view.workflow_mode()
    }

    /// Get current domain code if in domain editor.
    #[inline]
    pub fn current_domain_code(&self) -> Option<&str> {
        self.view.current_domain()
    }

    /// Get current editor tab if in domain editor.
    #[inline]
    pub fn current_tab(&self) -> Option<EditorTab> {
        self.view.current_tab()
    }
}
