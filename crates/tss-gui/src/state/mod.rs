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

mod dialog;
mod domain_state;
mod settings;
mod study;
mod view_state;

// Re-exports - Dialog types from the unified registry
pub use dialog::{DialogRegistry, DialogState, ExportProgressState, PendingAction};
// Re-export DialogType from dialog module (remove local definition)
pub use dialog::DialogType;
pub use domain_state::{
    DomainSource, DomainState, SourceDomainState, SuppAction, SuppColumnConfig, SuppOrigin,
};
pub use settings::{
    AssignmentMode, ExportFormat, RecentProject, SdtmIgVersion, Settings, WorkflowType, XptVersion,
};
pub use study::Study;
pub use view_state::{
    EditorTab, ExportPhase, ExportResult, ExportViewState, MappingUiState, NormalizationUiState,
    NotCollectedEdit, PreviewUiState, SeverityFilter, SourceAssignmentUiState, SourceFileEntry,
    SourceFileStatus, SuppEditDraft, SuppFilterMode, SuppUiState, TargetDomainEntry,
    ValidationUiState, ViewState, WorkflowMode,
};

use std::path::PathBuf;

use crate::component::feedback::toast::ToastState;
use crate::error::GuiError;
#[cfg(not(target_os = "macos"))]
use crate::menu::MenuDropdownState;
use crate::theme::ThemeConfig;
use iced::window;
use tss_persistence::{AutoSaveConfig, DirtyTracker};
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

    /// Theme configuration (derived from settings, cached for quick access).
    pub theme_config: ThemeConfig,

    /// Whether the system is in dark mode (for ThemeMode::System).
    /// Updated via system::theme_changes() subscription.
    pub system_is_dark: bool,

    /// CDISC Controlled Terminology registry.
    ///
    /// Loaded once when first study is opened. Used for validation
    /// and controlled term lookups.
    pub terminology: Option<TerminologyRegistry>,

    /// Current error to display (transient).
    ///
    /// Set when an operation fails, cleared on user acknowledgment.
    /// Use `GuiError` for structured error handling with categories and suggestions.
    pub error: Option<GuiError>,

    /// Whether a background task is running (for UI feedback).
    pub is_loading: bool,

    /// Menu dropdown state (for in-app menu on Windows/Linux).
    /// Only present on desktop platforms; macOS uses native menus.
    #[cfg(not(target_os = "macos"))]
    pub menu_dropdown: MenuDropdownState,

    /// Unified registry for managing dialog windows (multi-window mode).
    pub dialog_registry: DialogRegistry,

    /// Main window ID (for identifying the main window in multi-window mode).
    pub main_window_id: Option<window::Id>,

    /// Active toast notification (if any).
    pub toast: Option<ToastState>,

    // =========================================================================
    // Project persistence
    // =========================================================================
    /// Path to the current .tss project file.
    ///
    /// `None` if the project hasn't been saved yet (new/unsaved project).
    pub project_path: Option<PathBuf>,

    /// Tracks unsaved changes for auto-save and dirty indicator.
    ///
    /// Used to show "*" in title bar and trigger auto-save after debounce.
    pub dirty_tracker: DirtyTracker,

    /// Auto-save configuration.
    ///
    /// Controls whether auto-save is enabled and its timing parameters.
    pub auto_save_config: AutoSaveConfig,

    /// Pending action to perform after save completes.
    ///
    /// Used by the unsaved changes dialog to remember what action
    /// to perform after the user clicks "Save".
    pub pending_action_after_save: Option<PendingAction>,

    /// Pending project to restore after study loading completes.
    ///
    /// When opening a .tss project, we first load the CSVs, then apply
    /// the saved mapping decisions from this project data.
    pub pending_project_restore: Option<(std::path::PathBuf, tss_persistence::ProjectFile)>,
}

// ExportProgressState and DialogType are now defined in dialog.rs and re-exported above

impl AppState {
    /// Create app state with loaded settings.
    pub fn with_settings(settings: Settings) -> Self {
        let theme_config = ThemeConfig::new(
            settings.display.theme_mode,
            settings.display.accessibility_mode,
        );

        // Create auto-save config from settings
        let auto_save_config = AutoSaveConfig {
            enabled: settings.general.auto_save_enabled,
            debounce_ms: settings.general.auto_save_debounce_ms,
            ..AutoSaveConfig::default()
        };

        Self {
            view: ViewState::default(),
            study: None,
            settings,
            theme_config,
            system_is_dark: false, // Will be updated by system::theme_changes() subscription
            terminology: None,
            error: None,
            is_loading: false,
            #[cfg(not(target_os = "macos"))]
            menu_dropdown: MenuDropdownState::default(),
            dialog_registry: DialogRegistry::new(),
            main_window_id: None,
            toast: None,
            project_path: None,
            dirty_tracker: DirtyTracker::new(),
            auto_save_config,
            pending_action_after_save: None,
            pending_project_restore: None,
        }
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
}
