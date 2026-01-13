//! Domain editor messages.
//!
//! Messages for the domain editor view including all five tabs:
//! Mapping, Transform, Validation, Preview, and SUPP.

use crate::state::navigation::EditorTab;

/// Messages for the Domain Editor view.
#[derive(Debug, Clone)]
pub enum DomainEditorMessage {
    /// Switch to a different tab
    TabSelected(EditorTab),

    /// Go back to home view
    BackClicked,

    /// Mapping tab messages
    Mapping(MappingMessage),

    /// Transform/Normalization tab messages
    Transform(TransformMessage),

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
// TRANSFORM TAB
// =============================================================================

/// Messages for the Transform/Normalization tab.
#[derive(Debug, Clone)]
pub enum TransformMessage {
    /// Select a transform rule
    RuleSelected(usize),

    /// Toggle a transform rule on/off
    RuleToggled { index: usize, enabled: bool },

    /// Refresh transform preview
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

/// Messages for the SUPP configuration tab.
#[derive(Debug, Clone)]
pub enum SuppMessage {
    /// Select a column in the SUPP list
    ColumnSelected(String),

    /// Start editing a SUPP column
    StartEditing(String),

    /// Cancel editing
    CancelEditing,

    /// Save SUPP column changes
    SaveColumn {
        column: String,
        qnam: String,
        qlabel: String,
        qorig: QualifierOrigin,
        qeval: String,
    },

    /// Change the action for a SUPP column
    ActionChanged { column: String, action: SuppAction },

    /// QNAM field changed (during editing)
    QnamChanged(String),

    /// QLABEL field changed (during editing)
    QlabelChanged(String),

    /// QORIG changed (during editing)
    QorigChanged(QualifierOrigin),

    /// QEVAL field changed (during editing)
    QevalChanged(String),
}

/// Origin of a SUPP qualifier value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QualifierOrigin {
    /// Data from case report form
    #[default]
    Crf,
    /// Derived from other data
    Derived,
    /// Sponsor-assigned value
    Assigned,
}

impl QualifierOrigin {
    /// Returns the CDISC code for this origin.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Crf => "CRF",
            Self::Derived => "DERIVED",
            Self::Assigned => "ASSIGNED",
        }
    }

    /// Returns a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Crf => "Case Report Form",
            Self::Derived => "Derived",
            Self::Assigned => "Assigned",
        }
    }
}

/// Action to take for a SUPP column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SuppAction {
    /// Column is pending review
    #[default]
    Pending,
    /// Add to SUPP domain
    AddToSupp,
    /// Skip this column (don't include in output)
    Skip,
}

impl SuppAction {
    /// Returns a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::AddToSupp => "Add to SUPP",
            Self::Skip => "Skip",
        }
    }
}
