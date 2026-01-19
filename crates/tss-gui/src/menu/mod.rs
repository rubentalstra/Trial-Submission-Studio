//! Menu module for Trial Submission Studio.
//!
//! This module provides a unified menu abstraction across platforms:
//!
//! - **macOS**: Native NSMenu via `muda` crate with channel-based events
//! - **Windows/Linux**: Custom Iced-rendered menu bar
//!
//! The core abstraction is the [`MenuAction`] enum, which represents all
//! possible menu actions in a platform-agnostic way.

pub mod common;

// Platform-specific modules with single cfg gate at module level
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(not(target_os = "macos"))]
pub mod desktop;

// Re-exports for platform-specific implementations
#[cfg(target_os = "macos")]
pub use macos::{RecentStudyInfo, create_menu, menu_subscription, update_recent_studies_menu};

#[cfg(not(target_os = "macos"))]
pub use desktop::{DropdownId, MenuDropdownState, view_menu_bar};

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
    /// Open a study folder
    OpenStudy,

    /// Open a recent study by its UUID
    OpenRecentStudy(Uuid),

    /// Close the current study
    CloseStudy,

    /// Clear recent studies list
    ClearRecentStudies,

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

    /// Close all dropdown menus (desktop only)
    #[cfg(not(target_os = "macos"))]
    CloseDropdowns,
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
        assert_eq!(MenuAction::OpenStudy, MenuAction::OpenStudy);
        assert_ne!(MenuAction::OpenStudy, MenuAction::CloseStudy);

        let uuid = Uuid::new_v4();
        assert_eq!(
            MenuAction::OpenRecentStudy(uuid),
            MenuAction::OpenRecentStudy(uuid)
        );
    }

    #[test]
    fn test_menu_action_debug() {
        let action = MenuAction::Settings;
        assert!(format!("{:?}", action).contains("Settings"));
    }
}
