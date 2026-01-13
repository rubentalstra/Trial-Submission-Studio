//! Message module for Trial Submission Studio.
//!
//! This module defines the message hierarchy for the Elm-style architecture.
//! All user interactions and events flow through these message types.

pub mod dialog;
pub mod domain_editor;
pub mod export;
pub mod home;
pub mod menu;

use std::path::PathBuf;

use iced::keyboard;
use tss_model::TerminologyRegistry;

// Use new state types
use crate::state::{EditorTab, Study, ViewState, WorkflowMode};

pub use dialog::{
    AboutMessage, DeveloperSettingsMessage, DialogMessage, DisplaySettingsMessage,
    ExportSettingsMessage, GeneralSettingsMessage, SettingsCategory, SettingsMessage,
    ThirdPartyMessage, UpdateCheckFrequency, UpdateMessage, UpdateSettingsMessage,
    ValidationSettingsMessage,
};
pub use domain_editor::DomainEditorMessage;
pub use export::ExportMessage;
pub use home::HomeMessage;
pub use menu::MenuMessage;

/// Re-export ValidationReport from tss_validate for convenience.
pub type ValidationReport = tss_validate::ValidationReport;

/// Result type for study loading (includes terminology registry).
pub type StudyLoadResult = Result<(Study, TerminologyRegistry), String>;

/// Root message enum for the application.
///
/// All user interactions and system events are represented as variants
/// of this enum. The `update` function processes these messages to
/// modify application state.
#[derive(Debug, Clone)]
pub enum Message {
    // =========================================================================
    // Navigation
    // =========================================================================
    /// Navigate to a different view.
    Navigate(ViewState),

    /// Change the workflow mode (SDTM, ADaM, SEND).
    SetWorkflowMode(WorkflowMode),

    // =========================================================================
    // View-specific messages
    // =========================================================================
    /// Home view messages.
    Home(HomeMessage),

    /// Domain editor messages (includes all tabs).
    DomainEditor(DomainEditorMessage),

    /// Export view messages.
    Export(ExportMessage),

    // =========================================================================
    // Dialogs
    // =========================================================================
    /// Dialog messages (About, Settings, ThirdParty, Update).
    Dialog(DialogMessage),

    // =========================================================================
    // Menu
    // =========================================================================
    /// Menu action messages.
    Menu(MenuMessage),

    // =========================================================================
    // Background task results
    // =========================================================================
    /// Study loading completed (includes study and terminology registry).
    StudyLoaded(StudyLoadResult),

    /// Preview computation completed for a domain.
    PreviewReady {
        domain: String,
        result: Result<polars::prelude::DataFrame, String>,
    },

    /// Validation completed for a domain.
    ValidationComplete {
        domain: String,
        report: ValidationReport,
    },

    /// Update check completed.
    UpdateCheckComplete(Result<Option<UpdateInfo>, String>),

    // =========================================================================
    // Global events
    // =========================================================================
    /// Keyboard event.
    KeyPressed(keyboard::Key, keyboard::Modifiers),

    /// File dialog returned a folder selection.
    FolderSelected(Option<PathBuf>),

    /// Dismiss error message.
    DismissError,

    /// No operation - used for placeholder actions.
    Noop,
}

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub changelog: String,
    pub download_url: String,
}

impl Message {
    /// Creates a navigation message to go to the home view.
    pub fn go_home() -> Self {
        Self::Navigate(ViewState::home())
    }

    /// Creates a navigation message to go to the export view.
    pub fn go_export() -> Self {
        Self::Navigate(ViewState::export())
    }

    /// Creates a navigation message to go to a domain editor.
    pub fn go_domain(domain: impl Into<String>, tab: EditorTab) -> Self {
        Self::Navigate(ViewState::domain_editor(domain, tab))
    }
}
