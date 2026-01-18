//! Menu messages.
//!
//! Messages triggered by menu actions (both native and in-app menus).

use std::path::PathBuf;

/// Messages for menu actions.
#[derive(Debug, Clone)]
pub enum MenuMessage {
    // =========================================================================
    // File menu
    // =========================================================================
    /// Open a study folder
    OpenStudy,

    /// Open a specific recent study by path
    OpenRecentStudy(PathBuf),

    /// Close the current study
    CloseStudy,

    /// Clear recent studies list
    ClearRecentStudies,

    /// Open settings dialog
    Settings,

    /// Quit the application
    Quit,

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
    // Edit menu (platform standard)
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
}
