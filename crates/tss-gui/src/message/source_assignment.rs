//! Source assignment view messages.
//!
//! Messages for the source-to-domain assignment screen where users
//! manually map CSV files to CDISC domains.

/// Messages for the Source Assignment view.
#[derive(Debug, Clone)]
pub enum SourceAssignmentMessage {
    // =========================================================================
    // Drag and Drop (when AssignmentMode::DragAndDrop)
    // =========================================================================
    /// User started dragging a file.
    DragStarted {
        /// Index of the file being dragged.
        file_index: usize,
    },

    /// User is dragging over a domain (or moved away).
    DragOverDomain {
        /// Domain code being hovered, or None if not over a domain.
        domain_code: Option<String>,
    },

    /// User dropped a file on a domain.
    DroppedOnDomain {
        /// Index of the file being dropped.
        file_index: usize,
        /// Domain code to assign the file to.
        domain_code: String,
    },

    /// User cancelled the drag operation.
    DragCancelled,

    // =========================================================================
    // Click-to-Assign (when AssignmentMode::ClickToAssign)
    // =========================================================================
    /// User clicked on a file to select it for assignment.
    FileClicked {
        /// Index of the clicked file.
        file_index: usize,
    },

    /// User clicked on a domain to assign the selected file.
    DomainClicked {
        /// Domain code to assign to.
        domain_code: String,
    },

    // =========================================================================
    // Context Menu Actions
    // =========================================================================
    /// Mark a file as metadata.
    MarkAsMetadata {
        /// Index of the file to mark.
        file_index: usize,
    },

    /// Mark a file as skipped.
    MarkAsSkipped {
        /// Index of the file to mark.
        file_index: usize,
    },

    /// Unmark a file (restore to unassigned).
    UnmarkFile {
        /// Index of the file to unmark.
        file_index: usize,
    },

    /// Unassign a file from a domain.
    UnassignFile {
        /// Domain code the file is assigned to.
        domain_code: String,
        /// Index of the file to unassign.
        file_index: usize,
    },

    // =========================================================================
    // Search & Filter
    // =========================================================================
    /// Source file search text changed.
    SourceSearchChanged(String),

    /// Domain search text changed.
    DomainSearchChanged(String),

    // =========================================================================
    // Navigation
    // =========================================================================
    /// User clicked the back button.
    BackClicked,

    /// User confirmed going back (discarding progress).
    BackConfirmed,

    /// User cancelled going back.
    BackCancelled,

    /// User clicked the continue button.
    ContinueClicked,

    /// Study creation completed successfully.
    StudyCreated(super::StudyLoadResult),
}
