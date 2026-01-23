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
    // File menu - Project operations
    // =========================================================================

    /// Create a new project
    pub const NEW_PROJECT: &str = "new_project";

    /// Open a project file (.tss)
    pub const OPEN_PROJECT: &str = "open_project";

    /// Save the current project
    pub const SAVE_PROJECT: &str = "save_project";

    /// Save the current project to a new location
    pub const SAVE_PROJECT_AS: &str = "save_project_as";

    /// Close the current project
    pub const CLOSE_PROJECT: &str = "close_project";

    /// Clear recent projects list
    pub const CLEAR_RECENT: &str = "clear_recent";

    // -------------------------------------------------------------------------
    // Recent Projects submenu
    // -------------------------------------------------------------------------

    /// Prefix for recent project menu items (followed by UUID)
    pub const RECENT_PROJECT_PREFIX: &str = "recent_project:";

    /// Placeholder when no recent projects exist
    pub const NO_RECENT_PROJECTS: &str = "no_recent_projects";

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
