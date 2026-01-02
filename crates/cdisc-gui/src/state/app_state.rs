//! Application-level state.
//!
//! This module contains `AppState` which is the root of all state.

use cdisc_model::TerminologyRegistry;

use super::{DomainState, StudyState, UiState};
use crate::settings::Settings;

/// Top-level application state.
///
/// This is the root of all state in the application.
pub struct AppState {
    /// Current view/screen
    pub view: View,
    /// Workflow mode (SDTM, ADaM, SEND)
    pub workflow_mode: WorkflowMode,
    /// Loaded study (None if no study loaded)
    pub study: Option<StudyState>,
    /// Application settings (persisted)
    pub settings: Settings,
    /// All UI state (separated from domain data)
    pub ui: UiState,
    /// Cached CT registry (loaded lazily)
    ct_registry: Option<TerminologyRegistry>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: View::default(),
            workflow_mode: WorkflowMode::default(),
            study: None,
            settings: Settings::default(),
            ui: UiState::default(),
            ct_registry: None,
        }
    }
}

impl AppState {
    /// Create new app state with loaded settings.
    pub fn new(settings: Settings) -> Self {
        Self {
            view: View::default(),
            workflow_mode: WorkflowMode::default(),
            study: None,
            settings,
            ui: UiState::default(),
            ct_registry: None,
        }
    }

    /// Set the workflow mode.
    pub fn set_workflow_mode(&mut self, mode: WorkflowMode) {
        self.workflow_mode = mode;
    }

    /// Get the cached CT registry.
    ///
    /// CT is loaded automatically when a study is loaded via `set_study()`.
    pub fn ct_registry(&self) -> Option<&TerminologyRegistry> {
        self.ct_registry.as_ref()
    }

    // ========================================================================
    // Domain Access
    // ========================================================================

    /// Get domain state.
    ///
    /// Returns `None` if no study is loaded or domain doesn't exist.
    pub fn domain(&self, code: &str) -> Option<&DomainState> {
        self.study.as_ref()?.get_domain(code)
    }

    /// Get mutable domain state.
    pub fn domain_mut(&mut self, code: &str) -> Option<&mut DomainState> {
        self.study.as_mut()?.get_domain_mut(code)
    }

    /// Check if a domain exists.
    pub fn is_domain_accessible(&self, code: &str) -> bool {
        self.study
            .as_ref()
            .map(|s| s.has_domain(code))
            .unwrap_or(false)
    }

    // ========================================================================
    // Navigation
    // ========================================================================

    /// Navigate to home screen.
    pub fn go_home(&mut self) {
        self.view = View::Home;
    }

    /// Navigate to domain editor.
    pub fn open_domain(&mut self, domain: String) {
        self.view = View::DomainEditor {
            domain,
            tab: EditorTab::Mapping,
        };
    }

    /// Navigate to export screen.
    pub fn go_export(&mut self) {
        self.view = View::Export;
    }

    /// Switch tab in domain editor.
    pub fn switch_tab(&mut self, tab: EditorTab) {
        if let View::DomainEditor {
            tab: current_tab, ..
        } = &mut self.view
        {
            *current_tab = tab;
        }
    }

    // ========================================================================
    // Convenience Accessors
    // ========================================================================

    /// Get study reference.
    pub fn study(&self) -> Option<&StudyState> {
        self.study.as_ref()
    }

    /// Get mutable study reference.
    pub fn study_mut(&mut self) -> Option<&mut StudyState> {
        self.study.as_mut()
    }

    // ========================================================================
    // Settings Management
    // ========================================================================

    /// Open the settings window.
    pub fn open_settings(&mut self) {
        self.ui.settings.open(&self.settings);
    }

    /// Close the settings window.
    ///
    /// If `apply` is true, the pending settings are applied.
    /// If `apply` is false, the pending settings are discarded.
    pub fn close_settings(&mut self, apply: bool) {
        if let Some(new_settings) = self.ui.settings.close(apply) {
            self.settings = new_settings;
        }
    }

    /// Check if settings window is open.
    pub fn is_settings_open(&self) -> bool {
        self.ui.settings.is_open()
    }

    // ========================================================================
    // Study Management
    // ========================================================================

    /// Set a new study, resetting UI state.
    pub fn set_study(&mut self, study: StudyState) {
        self.study = Some(study);
        self.ui.clear_domain_editors();
        self.ui.export.reset();
        self.view = View::Home;
        // Load CT registry for the study
        if self.ct_registry.is_none() {
            self.ct_registry = cdisc_standards::load_ct(cdisc_standards::CtVersion::default()).ok();
        }
    }

    /// Clear the current study.
    #[allow(dead_code)]
    pub fn clear_study(&mut self) {
        self.study = None;
        self.ui.clear_domain_editors();
        self.ui.export.reset();
        self.view = View::Home;
    }
}

// ============================================================================
// Workflow Mode Enum
// ============================================================================

/// CDISC standard workflow mode.
///
/// Determines which Implementation Guide is used for the current study.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowMode {
    /// SDTM - Study Data Tabulation Model (clinical trials)
    #[default]
    Sdtm,
    /// ADaM - Analysis Data Model (analysis-ready datasets)
    Adam,
    /// SEND - Standard for Exchange of Nonclinical Data (animal studies)
    Send,
}

impl WorkflowMode {
    /// Display name for the workflow mode.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sdtm => "SDTM",
            Self::Adam => "ADaM",
            Self::Send => "SEND",
        }
    }

    /// Full description of the workflow mode.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Sdtm => "Study Data Tabulation Model",
            Self::Adam => "Analysis Data Model",
            Self::Send => "Standard for Exchange of Nonclinical Data",
        }
    }

    /// Short tagline for UI cards.
    pub fn tagline(&self) -> &'static str {
        match self {
            Self::Sdtm => "Clinical Trial Tabulation",
            Self::Adam => "Analysis Datasets",
            Self::Send => "Nonclinical Studies",
        }
    }
}

// ============================================================================
// View Enum
// ============================================================================

/// Current view in the application.
#[derive(Default, Clone, PartialEq)]
pub enum View {
    /// Home screen - study selection
    #[default]
    Home,
    /// Domain editor with tabs
    DomainEditor {
        /// Selected domain code (e.g., "DM", "AE")
        domain: String,
        /// Active tab
        tab: EditorTab,
    },
    /// Export screen
    Export,
}

// ============================================================================
// Editor Tab Enum
// ============================================================================

/// Tabs in the domain editor.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum EditorTab {
    #[default]
    Mapping,
    Transform,
    Validation,
    Preview,
    Supp,
}

impl EditorTab {
    /// Get display name for the tab (with icon).
    pub fn label(&self) -> String {
        format!("{} {}", self.icon(), self.name())
    }

    /// Get just the tab name without icon.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mapping => "Mapping",
            Self::Transform => "Transform",
            Self::Validation => "Validation",
            Self::Preview => "Preview",
            Self::Supp => "SUPP",
        }
    }

    /// Get tab icon (phosphor icon).
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Mapping => egui_phosphor::regular::ARROWS_LEFT_RIGHT,
            Self::Transform => egui_phosphor::regular::SHUFFLE,
            Self::Validation => egui_phosphor::regular::CHECK_SQUARE,
            Self::Preview => egui_phosphor::regular::EYE,
            Self::Supp => egui_phosphor::regular::PLUS_SQUARE,
        }
    }

    /// Get all tabs in order.
    pub fn all() -> &'static [EditorTab] {
        &[
            Self::Mapping,
            Self::Transform,
            Self::Validation,
            Self::Preview,
            Self::Supp,
        ]
    }
}
