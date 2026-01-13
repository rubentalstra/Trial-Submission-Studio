//! Navigation state types.
//!
//! This module contains enums for application navigation:
//! - `View`: Current screen/route
//! - `EditorTab`: Tabs within the domain editor
//! - `WorkflowMode`: CDISC standard workflow (SDTM, ADaM, SEND)

// =============================================================================
// VIEW ENUM
// =============================================================================

/// Current view/screen in the application.
///
/// This determines what is rendered in the main content area.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum View {
    /// Home screen - study selection and overview
    #[default]
    Home,

    /// Domain editor with tabbed interface
    DomainEditor {
        /// Selected domain code (e.g., "DM", "AE", "LB")
        domain: String,
        /// Active tab within the editor
        tab: EditorTab,
    },

    /// Export screen - configure and execute export
    Export,
}

impl View {
    /// Create a new DomainEditor view for the given domain.
    pub fn domain_editor(domain: impl Into<String>) -> Self {
        Self::DomainEditor {
            domain: domain.into(),
            tab: EditorTab::default(),
        }
    }

    /// Create a DomainEditor view with a specific tab.
    pub fn domain_editor_with_tab(domain: impl Into<String>, tab: EditorTab) -> Self {
        Self::DomainEditor {
            domain: domain.into(),
            tab,
        }
    }

    /// Get the current domain code if in DomainEditor view.
    pub fn current_domain(&self) -> Option<&str> {
        match self {
            Self::DomainEditor { domain, .. } => Some(domain),
            _ => None,
        }
    }

    /// Get the current tab if in DomainEditor view.
    pub fn current_tab(&self) -> Option<EditorTab> {
        match self {
            Self::DomainEditor { tab, .. } => Some(*tab),
            _ => None,
        }
    }

    /// Check if this is the Home view.
    pub fn is_home(&self) -> bool {
        matches!(self, Self::Home)
    }

    /// Check if this is the Export view.
    pub fn is_export(&self) -> bool {
        matches!(self, Self::Export)
    }

    /// Check if this is a DomainEditor view.
    pub fn is_domain_editor(&self) -> bool {
        matches!(self, Self::DomainEditor { .. })
    }
}

// =============================================================================
// EDITOR TAB ENUM
// =============================================================================

/// Tabs in the domain editor.
///
/// Each tab represents a different aspect of domain configuration:
/// - Mapping: Variable-to-column mapping
/// - Transform: Data normalization rules
/// - Validation: CDISC conformance checking
/// - Preview: View transformed data
/// - Supp: SUPP qualifier configuration
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorTab {
    /// Variable mapping tab (default)
    #[default]
    Mapping,

    /// Data transformation/normalization tab
    Transform,

    /// Validation results tab
    Validation,

    /// Data preview tab
    Preview,

    /// SUPP qualifier configuration tab
    Supp,
}

impl EditorTab {
    /// Get the display name for this tab.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mapping => "Mapping",
            Self::Transform => "Transform",
            Self::Validation => "Validation",
            Self::Preview => "Preview",
            Self::Supp => "SUPP",
        }
    }

    /// Get a short description of this tab.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Mapping => "Map source columns to CDISC variables",
            Self::Transform => "Configure data normalization rules",
            Self::Validation => "Review CDISC conformance issues",
            Self::Preview => "Preview transformed output data",
            Self::Supp => "Configure supplemental qualifiers",
        }
    }

    /// Get all tabs in display order.
    pub const fn all() -> &'static [EditorTab] {
        &[
            Self::Mapping,
            Self::Transform,
            Self::Validation,
            Self::Preview,
            Self::Supp,
        ]
    }

    /// Get the index of this tab (0-based).
    pub fn index(&self) -> usize {
        match self {
            Self::Mapping => 0,
            Self::Transform => 1,
            Self::Validation => 2,
            Self::Preview => 3,
            Self::Supp => 4,
        }
    }

    /// Create a tab from its index.
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Mapping),
            1 => Some(Self::Transform),
            2 => Some(Self::Validation),
            3 => Some(Self::Preview),
            4 => Some(Self::Supp),
            _ => None,
        }
    }
}

// =============================================================================
// WORKFLOW MODE ENUM
// =============================================================================

/// CDISC standard workflow mode.
///
/// Determines which Implementation Guide is used for the study.
/// Each mode has different domains, validation rules, and terminology.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkflowMode {
    /// SDTM - Study Data Tabulation Model
    ///
    /// Used for clinical trial tabulation data submitted to FDA.
    #[default]
    Sdtm,

    /// ADaM - Analysis Data Model
    ///
    /// Used for analysis-ready datasets derived from SDTM.
    Adam,

    /// SEND - Standard for Exchange of Nonclinical Data
    ///
    /// Used for nonclinical/animal study data.
    Send,
}

impl WorkflowMode {
    /// Get the short display name (e.g., "SDTM").
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sdtm => "SDTM",
            Self::Adam => "ADaM",
            Self::Send => "SEND",
        }
    }

    /// Get the full description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Sdtm => "Study Data Tabulation Model",
            Self::Adam => "Analysis Data Model",
            Self::Send => "Standard for Exchange of Nonclinical Data",
        }
    }

    /// Get a short tagline for UI cards.
    pub fn tagline(&self) -> &'static str {
        match self {
            Self::Sdtm => "Clinical Trial Tabulation",
            Self::Adam => "Analysis Datasets",
            Self::Send => "Nonclinical Studies",
        }
    }

    /// Get all workflow modes.
    pub const fn all() -> &'static [WorkflowMode] {
        &[Self::Sdtm, Self::Adam, Self::Send]
    }
}
