//! Menu module for Trial Submission Studio.
//!
//! This module provides a unified menu abstraction across platforms:
//!
//! - **macOS**: Native NSMenu via `muda` crate with channel-based events
//! - **Windows/Linux**: Custom Iced-rendered menu bar
//!
//! The core abstraction is the [`MenuAction`] enum, which represents all
//! possible menu actions in a platform-agnostic way.

#[cfg(target_os = "macos")]
pub mod common;

// Platform-specific modules with single cfg gate at module level
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(not(target_os = "macos"))]
pub mod desktop;

// Re-exports for platform-specific implementations
#[cfg(target_os = "macos")]
pub use macos::{
    RecentProjectInfo, create_menu, init_menu_channel, menu_subscription,
    update_recent_projects_menu,
};

#[cfg(not(target_os = "macos"))]
pub use desktop::{DropdownId, MenuDropdownState, view_menu_bar};

#[cfg(target_os = "macos")]
use uuid::Uuid;

// =============================================================================
// MENU ACTION
// =============================================================================

/// Unified menu action enum.
///
/// This is the core abstraction that represents all menu actions in a
/// platform-agnostic way. Both macOS native menus and desktop in-app menus
/// convert their events to this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    // =========================================================================
    // File menu
    // =========================================================================
    /// Create a new project
    NewProject,

    /// Open a project file (.tss)
    OpenProject,

    /// Open a recent project by its UUID (macOS only - desktop uses path-based approach)
    #[cfg(target_os = "macos")]
    OpenRecentProject(Uuid),

    /// Save the current project
    SaveProject,

    /// Save the current project to a new location
    SaveProjectAs,

    /// Close the current project
    CloseProject,

    /// Clear recent projects list
    ClearRecentProjects,

    /// Open settings dialog
    Settings,

    /// Quit the application
    Quit,

    // =========================================================================
    // Edit menu (stubs - Iced handles natively)
    // =========================================================================
    /// Undo action
    Undo,

    /// Redo action
    Redo,

    /// Cut selection
    Cut,

    /// Copy selection
    Copy,

    /// Paste from clipboard
    Paste,

    /// Select all
    SelectAll,

    // =========================================================================
    // Help menu
    // =========================================================================
    /// Open documentation in browser
    Documentation,

    /// Open release notes
    ReleaseNotes,

    /// Open GitHub repository
    ViewOnGitHub,

    /// Report an issue on GitHub
    ReportIssue,

    /// View license information
    ViewLicense,

    /// View third-party licenses
    ThirdPartyLicenses,

    /// Check for updates
    CheckUpdates,

    /// Open About dialog
    About,

    // =========================================================================
    // Desktop-only: Menu bar toggle (not used on macOS)
    // =========================================================================
    /// Toggle a dropdown menu (desktop only)
    #[cfg(not(target_os = "macos"))]
    ToggleDropdown(DropdownId),
}

// Desktop-only: Allow converting DropdownId to MenuAction for toggle
#[cfg(not(target_os = "macos"))]
impl From<DropdownId> for MenuAction {
    fn from(id: DropdownId) -> Self {
        MenuAction::ToggleDropdown(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_action_equality() {
        assert_eq!(MenuAction::NewProject, MenuAction::NewProject);
        assert_ne!(MenuAction::NewProject, MenuAction::CloseProject);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_menu_action_recent_project() {
        let uuid = Uuid::new_v4();
        assert_eq!(
            MenuAction::OpenRecentProject(uuid),
            MenuAction::OpenRecentProject(uuid)
        );
    }

    #[test]
    fn test_menu_action_debug() {
        let action = MenuAction::Settings;
        assert!(format!("{:?}", action).contains("Settings"));
    }
}
