//! Home view messages.
//!
//! Messages for the home screen including study selection,
//! recent studies, and workflow mode selection.

use std::path::PathBuf;

/// Messages for the Home view.
#[derive(Debug, Clone)]
pub enum HomeMessage {
    // =========================================================================
    // Study selection
    // =========================================================================
    /// User clicked "Open Study" button
    OpenStudyClicked,

    /// User selected a folder from the file dialog
    StudyFolderSelected(PathBuf),

    /// User clicked on a recent study
    RecentStudyClicked(PathBuf),

    /// User wants to close the current study
    CloseStudyClicked,

    /// User confirmed closing the study
    CloseStudyConfirmed,

    /// User cancelled closing the study
    CloseStudyCancelled,

    // =========================================================================
    // Navigation
    // =========================================================================
    /// User clicked on a domain to open it
    DomainClicked(String),

    /// User clicked "Go to Export" button
    GoToExportClicked,

    // =========================================================================
    // Study info
    // =========================================================================
    /// Remove a study from recent list
    RemoveFromRecent(PathBuf),

    /// Clear all recent studies
    ClearRecentStudies,
}
