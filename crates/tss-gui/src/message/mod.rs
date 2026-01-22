//! Message module for Trial Submission Studio.
//!
//! This module defines the message hierarchy for the Elm-style architecture.
//! All user interactions and events flow through these message types.
//!
//! Note: Some variants may appear unused on certain platforms (e.g., menu variants on macOS)
//! or are part of features still being wired up. The message system defines the application's
//! complete capability surface.

#![allow(dead_code)]

pub mod dialog;
pub mod domain_editor;
pub mod export;
pub mod home;
pub mod menu;
pub mod source_assignment;

use std::path::PathBuf;
use std::sync::Arc;

use iced::keyboard;
use iced::window;
use tss_standards::TerminologyRegistry;

// Use new state types
use crate::menu::MenuAction;
use crate::state::{DialogType, Study, ViewState, WorkflowMode};

pub use dialog::{
    AboutMessage, DeveloperSettingsMessage, DialogMessage, DisplaySettingsMessage,
    ExportSettingsMessage, GeneralSettingsMessage, SettingsCategory, SettingsMessage,
    ThirdPartyMessage, UpdateMessage, UpdateSettingsMessage, ValidationSettingsMessage,
    VerifyOutcome,
};
pub use domain_editor::DomainEditorMessage;
pub use export::ExportMessage;
pub use home::HomeMessage;
pub use menu::MenuMessage;
pub use source_assignment::SourceAssignmentMessage;

// Toast message
pub use crate::component::toast::ToastMessage;

/// Re-export ValidationReport from tss_validate for convenience.
pub type ValidationReport = tss_submit::ValidationReport;

/// Result type for study loading (includes terminology registry).
pub type StudyLoadResult = Result<(Study, TerminologyRegistry), String>;

/// Menu bar menu identifier for in-app menu (Windows/Linux).
///
/// This is a separate type to avoid circular dependencies with the menu module.
/// Only used on Windows/Linux where in-app menu bar is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarMenuId {
    File,
    Edit,
    Help,
}

/// Root message enum for the application.
///
/// All user interactions and system events are represented as variants
/// of this enum. The `update` function processes these messages to
/// modify application state.
///
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
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

    /// Source assignment view messages.
    SourceAssignment(SourceAssignmentMessage),

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
    /// Unified menu action (from native or in-app menu).
    MenuAction(MenuAction),

    /// Legacy menu action messages (being phased out).
    Menu(MenuMessage),

    /// Initialize native menu (startup task on macOS).
    InitNativeMenu,

    // =========================================================================
    // Multi-window dialog management
    // =========================================================================
    /// A dialog window was opened.
    DialogWindowOpened(DialogType, window::Id),

    /// A dialog window was closed.
    DialogWindowClosed(window::Id),

    /// Request to close a specific window.
    CloseWindow(window::Id),

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
    UpdateCheckComplete(Result<Option<tss_updater::UpdateInfo>, String>),

    /// Update is ready to install (verification passed or unavailable).
    /// Uses `Arc<Vec<u8>>` for cheap cloning of large binary data.
    UpdateReadyToInstall {
        info: tss_updater::UpdateInfo,
        data: Arc<Vec<u8>>,
        verified: bool,
    },

    // =========================================================================
    // Global events
    // =========================================================================
    /// System theme mode changed (light/dark).
    SystemThemeChanged(iced::theme::Mode),

    /// Keyboard event.
    KeyPressed(keyboard::Key, keyboard::Modifiers),

    /// File dialog returned a folder selection.
    FolderSelected(Option<PathBuf>),

    /// Dismiss error message.
    DismissError,

    // =========================================================================
    // External actions
    // =========================================================================
    /// Open a URL in the system browser.
    OpenUrl(String),

    // =========================================================================
    // Toast notifications
    // =========================================================================
    /// Toast notification messages.
    Toast(ToastMessage),

    /// No operation - used for placeholder actions.
    Noop,
}
