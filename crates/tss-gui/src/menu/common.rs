//! Shared constants for menu identifiers.
//!
//! These IDs are used by macOS native menus (muda).
//! On desktop platforms, actions are handled directly without string IDs.

/// Menu item identifiers for macOS native menus.
///
/// Using string IDs allows muda to identify which menu item was clicked.
#[cfg(target_os = "macos")]
pub mod ids {
    // =========================================================================
    // File menu
    // =========================================================================

    /// Open a study folder
    pub const OPEN_STUDY: &str = "open_study";

    /// Close the current study
    pub const CLOSE_STUDY: &str = "close_study";

    /// Clear recent studies list
    pub const CLEAR_RECENT: &str = "clear_recent";

    // -------------------------------------------------------------------------
    // Recent Studies submenu
    // -------------------------------------------------------------------------

    /// Prefix for recent study menu items (followed by UUID)
    pub const RECENT_STUDY_PREFIX: &str = "recent_study:";

    /// Placeholder when no recent studies exist
    pub const NO_RECENT_STUDIES: &str = "no_recent_studies";

    // =========================================================================
    // App/Settings
    // =========================================================================

    /// Open settings dialog
    pub const SETTINGS: &str = "settings";

    /// Check for updates
    pub const CHECK_UPDATES: &str = "check_updates";

    /// Show about dialog
    pub const ABOUT: &str = "about";

    /// Exit/Quit the application
    pub const EXIT: &str = "exit";

    // =========================================================================
    // Edit menu (stubs - Iced handles natively)
    // =========================================================================

    pub const UNDO: &str = "undo";
    pub const REDO: &str = "redo";
    pub const CUT: &str = "cut";
    pub const COPY: &str = "copy";
    pub const PASTE: &str = "paste";
    pub const SELECT_ALL: &str = "select_all";

    // =========================================================================
    // Help menu
    // =========================================================================

    /// Open documentation in browser
    pub const DOCUMENTATION: &str = "documentation";

    /// Open release notes
    pub const RELEASE_NOTES: &str = "release_notes";

    /// View on GitHub
    pub const VIEW_ON_GITHUB: &str = "view_on_github";

    /// Report an issue
    pub const REPORT_ISSUE: &str = "report_issue";

    /// View license
    pub const VIEW_LICENSE: &str = "view_license";

    /// View third-party licenses
    pub const THIRD_PARTY_LICENSES: &str = "third_party_licenses";
}
