//! Application-level state.
//!
//! This module contains `AppState` which is the root of all state.
//! Domain access is DM-enforced through the `domain()` method.

use cdisc_model::TerminologyRegistry;

use super::{DomainState, StudyState, UiState};
use crate::settings::Settings;

/// Top-level application state.
///
/// This is the root of all state in the application. Use the provided
/// accessor methods for domain access - they enforce DM dependency.
pub struct AppState {
    /// Current view/screen
    pub view: View,
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
            study: None,
            settings,
            ui: UiState::default(),
            ct_registry: None,
        }
    }

    /// Get the cached CT registry.
    ///
    /// CT is loaded automatically when a study is loaded via `set_study()`.
    pub fn ct_registry(&self) -> Option<&TerminologyRegistry> {
        self.ct_registry.as_ref()
    }

    // ========================================================================
    // Domain Access (DM-Enforced)
    // ========================================================================

    /// Get domain state with DM dependency check.
    ///
    /// Returns `None` if:
    /// - No study is loaded
    /// - Domain doesn't exist
    /// - Domain is locked (DM not ready)
    ///
    /// Use `study.get_domain()` to bypass DM check (not recommended).
    pub fn domain(&self, code: &str) -> Option<&DomainState> {
        let study = self.study.as_ref()?;

        // DM is always accessible
        if code.eq_ignore_ascii_case("DM") {
            return study.get_domain(code);
        }

        // Other domains require DM preview
        if study.dm_preview_version.is_none() && study.has_dm_domain() {
            return None;
        }

        study.get_domain(code)
    }

    /// Get mutable domain state with DM dependency check.
    pub fn domain_mut(&mut self, code: &str) -> Option<&mut DomainState> {
        // Check DM readiness first (immutable borrow)
        let is_accessible = {
            let study = self.study.as_ref()?;
            if code.eq_ignore_ascii_case("DM") {
                true
            } else if study.has_dm_domain() {
                study.dm_preview_version.is_some()
            } else {
                true
            }
        };

        if !is_accessible {
            return None;
        }

        self.study.as_mut()?.get_domain_mut(code)
    }

    /// Check if a domain is accessible (for UI to show lock icons).
    pub fn is_domain_accessible(&self, code: &str) -> bool {
        let Some(study) = &self.study else {
            return false;
        };

        // DM is always accessible if it exists
        if code.eq_ignore_ascii_case("DM") {
            return study.has_domain(code);
        }

        // If no DM domain exists, all domains are accessible
        if !study.has_dm_domain() {
            return study.has_domain(code);
        }

        // Other domains require DM to have preview
        study.dm_preview_version.is_some() && study.has_domain(code)
    }

    /// Get the reason a domain is locked, if any.
    pub fn domain_lock_reason(&self, code: &str) -> Option<&'static str> {
        let Some(study) = &self.study else {
            return None;
        };

        // DM is never locked
        if code.eq_ignore_ascii_case("DM") {
            return None;
        }

        // If no DM domain exists, nothing is locked
        if !study.has_dm_domain() {
            return None;
        }

        // Lock reason if DM is not ready
        if study.dm_preview_version.is_none() {
            return Some("Complete DM domain first");
        }

        None
    }

    // ========================================================================
    // Navigation
    // ========================================================================

    /// Navigate to home screen.
    pub fn go_home(&mut self) {
        self.view = View::Home;
    }

    /// Navigate to domain editor.
    ///
    /// Returns `true` if navigation succeeded, `false` if domain is locked.
    pub fn open_domain(&mut self, domain: String) -> bool {
        // Check if domain is accessible
        if !self.is_domain_accessible(&domain) {
            return false;
        }

        self.view = View::DomainEditor {
            domain,
            tab: EditorTab::Mapping,
        };
        true
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
