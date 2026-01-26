//! Home view messages.
//!
//! Messages for the home screen including project operations,
//! recent projects, and workflow mode selection.

use std::path::PathBuf;

/// Messages for the Home view.
#[derive(Debug, Clone)]
pub enum HomeMessage {
    // =========================================================================
    // Project operations
    // =========================================================================
    /// User selected a study folder from the file dialog (for new project)
    StudyFolderSelected(PathBuf),

    /// User wants to close the current project
    CloseProjectClicked,

    /// User confirmed closing the project
    CloseProjectConfirmed,

    /// User cancelled closing the project
    CloseProjectCancelled,

    // =========================================================================
    // Navigation
    // =========================================================================
    /// User clicked on a domain to open it
    DomainClicked(String),

    /// User clicked "Go to Export" button
    GoToExportClicked,

    // =========================================================================
    // Recent Projects
    // =========================================================================
    /// User clicked on a recent project
    RecentProjectClicked(PathBuf),

    /// Remove a project from recent list
    RemoveFromRecentProjects(PathBuf),

    /// Clear all recent projects
    ClearAllRecentProjects,

    /// Prune projects with missing paths
    PruneStaleProjects,
}
