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

    /// Mark a variable as "Not Collected"
    MarkNotCollected { variable: String },

    /// Confirm the "Not Collected" marking
    NotCollectedConfirmed { variable: String, reason: String },

    /// Cancel the "Not Collected" dialog
    NotCollectedCancelled,

    /// Mark a variable as "Omitted" (intentionally skipped)
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

    /// Toggle a normalization rule on/off
    RuleToggled { index: usize, enabled: bool },

    /// Refresh normalization preview
    RefreshPreview,
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
// SUPP TAB
// =============================================================================

use crate::state::{SuppAction, SuppOrigin};

/// Messages for the SUPP configuration tab.
#[derive(Debug, Clone)]
pub enum SuppMessage {
    /// Select a column in the SUPP list (master list selection)
    ColumnSelected(String),

    /// Search filter text changed
    SearchChanged(String),

    /// Filter mode changed
    FilterModeChanged(crate::state::SuppFilterMode),

    /// QNAM field changed for a column (inline editing, auto-uppercase)
    QnamChanged { column: String, value: String },

    /// QLABEL field changed for a column (inline editing)
    QlabelChanged { column: String, value: String },

    /// QORIG changed for a column (inline editing)
    QorigChanged { column: String, value: SuppOrigin },

    /// QEVAL field changed for a column (inline editing)
    QevalChanged { column: String, value: String },

    /// Action changed for a column (Pending/Include/Skip)
    ActionChanged { column: String, action: SuppAction },
}
