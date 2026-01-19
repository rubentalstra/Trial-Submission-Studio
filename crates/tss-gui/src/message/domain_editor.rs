//! Domain editor messages.
//!
//! Messages for the domain editor view including all five tabs:
//! Mapping, Normalization, Validation, Preview, and SUPP.

use crate::state::EditorTab;

/// Messages for the Domain Editor view.
#[derive(Debug, Clone)]
pub enum DomainEditorMessage {
    /// Switch to a different tab
    TabSelected(EditorTab),

    /// Go back to home view
    BackClicked,

    /// Mapping tab messages
    Mapping(MappingMessage),

    /// Normalization tab messages
    Normalization(NormalizationMessage),

    /// Validation tab messages
    Validation(ValidationMessage),

    /// Preview tab messages
    Preview(PreviewMessage),

    /// SUPP configuration tab messages
    Supp(SuppMessage),
}

// =============================================================================
// MAPPING TAB
// =============================================================================

/// Messages for the Mapping tab.
#[derive(Debug, Clone)]
pub enum MappingMessage {
    /// User selected a variable in the list
    VariableSelected(usize),

    /// Search text changed
    SearchChanged(String),

    /// Clear search
    SearchCleared,

    /// Accept the suggested mapping for a variable
    AcceptSuggestion(String),

    /// Clear the mapping for a variable
    ClearMapping(String),

    /// Manually map a variable to a column
    ManualMap { variable: String, column: String },

    /// Start marking a variable as "Not Collected" (shows inline reason input)
    MarkNotCollected { variable: String },

    /// Update the reason text while editing (inline)
    NotCollectedReasonChanged(String),

    /// Save the "Not Collected" marking with reason
    NotCollectedSave { variable: String, reason: String },

    /// Cancel the "Not Collected" inline editing
    NotCollectedCancel,

    /// Start editing an existing "Not Collected" reason
    EditNotCollectedReason {
        variable: String,
        current_reason: String,
    },

    /// Revert "Not Collected" back to unmapped (clear the status)
    ClearNotCollected(String),

    /// Mark a variable as "Omitted" (intentionally skipped, Perm only)
    MarkOmitted(String),

    /// Clear the "Omitted" status
    ClearOmitted(String),

    /// Show only unmapped variables
    FilterUnmappedToggled(bool),

    /// Show only required variables
    FilterRequiredToggled(bool),
}

// =============================================================================
// NORMALIZATION TAB
// =============================================================================

/// Messages for the Normalization tab.
#[derive(Debug, Clone)]
pub enum NormalizationMessage {
    /// Select a normalization rule
    RuleSelected(usize),
}

// =============================================================================
// VALIDATION TAB
// =============================================================================

/// Messages for the Validation tab.
#[derive(Debug, Clone)]
pub enum ValidationMessage {
    /// Select a validation issue
    IssueSelected(usize),

    /// Filter by severity
    SeverityFilterChanged(SeverityFilter),

    /// Refresh validation
    RefreshValidation,

    /// Navigate to the source of an issue
    GoToIssueSource { variable: String },
}

/// Filter for validation issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeverityFilter {
    /// Show all issues
    #[default]
    All,
    /// Show only errors
    Errors,
    /// Show only warnings
    Warnings,
    /// Show only info
    Info,
}

// =============================================================================
// PREVIEW TAB
// =============================================================================

/// Messages for the Preview tab.
#[derive(Debug, Clone)]
pub enum PreviewMessage {
    /// Go to a specific page
    GoToPage(usize),

    /// Go to the next page
    NextPage,

    /// Go to the previous page
    PreviousPage,

    /// Change rows per page
    RowsPerPageChanged(usize),

    /// Rebuild preview
    RebuildPreview,
}

// =============================================================================
// SUPP TAB (Redesigned - Clean Architecture)
// =============================================================================

use crate::state::SuppOrigin;

/// Messages for the SUPP configuration tab.
///
/// # Design
///
/// The SUPP tab uses a simple state-based flow:
/// - **Pending columns**: Editable inline, changes stored directly
/// - **Included columns**: Read-only view, click "Edit" to modify
/// - **Skipped columns**: Simple skip message, option to add instead
///
/// When editing an already-included column, changes go to a draft
/// and are only committed on "Save".
#[derive(Debug, Clone)]
pub enum SuppMessage {
    // =========================================================================
    // NAVIGATION & FILTERING
    // =========================================================================
    /// Select a column in the master list
    ColumnSelected(String),

    /// Search filter text changed
    SearchChanged(String),

    /// Filter mode changed (All/Pending/Included/Skipped)
    FilterModeChanged(crate::state::SuppFilterMode),

    // =========================================================================
    // FIELD EDITING
    // =========================================================================
    /// QNAM field changed (auto-uppercase, max 8 chars)
    QnamChanged(String),

    /// QLABEL field changed (max 40 chars)
    QlabelChanged(String),

    /// QORIG field changed
    QorigChanged(SuppOrigin),

    /// QEVAL field changed (optional)
    QevalChanged(String),

    // =========================================================================
    // ACTIONS
    // =========================================================================
    /// Add the current column to SUPP (Pending → Include)
    AddToSupp,

    /// Skip the current column (Pending → Skip)
    Skip,

    /// Remove from SUPP / Undo skip (Include/Skip → Pending)
    UndoAction,

    // =========================================================================
    // EDIT MODE (for already-included columns)
    // =========================================================================
    /// Enter edit mode for an included column
    StartEdit,

    /// Save changes and exit edit mode
    SaveEdit,

    /// Discard changes and exit edit mode
    CancelEdit,
}
