//! Application-level state

use super::StudyState;
use crate::settings::{ui::SettingsWindow, Settings};

/// Top-level application state
pub struct AppState {
    /// Current view/screen
    pub view: View,
    /// Loaded study (None if no study loaded)
    pub study: Option<StudyState>,
    /// Application settings (persisted)
    pub settings: Settings,
    /// Settings window visibility
    pub settings_open: bool,
    /// Pending settings (for cancel functionality)
    pub settings_pending: Option<Settings>,
    /// Settings window UI state
    pub settings_window: SettingsWindow,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: View::default(),
            study: None,
            settings: Settings::default(),
            settings_open: false,
            settings_pending: None,
            settings_window: SettingsWindow::default(),
        }
    }
}

impl AppState {
    /// Create new app state with loaded settings
    pub fn new(settings: Settings) -> Self {
        Self {
            view: View::default(),
            study: None,
            settings,
            settings_open: false,
            settings_pending: None,
            settings_window: SettingsWindow::default(),
        }
    }
}

/// Current view in the application
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

/// Tabs in the domain editor
#[derive(Default, Clone, Copy, PartialEq)]
pub enum EditorTab {
    #[default]
    Mapping,
    Transform,
    Validation,
    Preview,
    Supp,
}

impl EditorTab {
    /// Get display name for the tab (with icon)
    pub fn label(&self) -> String {
        format!("{} {}", self.icon(), self.name())
    }

    /// Get just the tab name without icon
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mapping => "Mapping",
            Self::Transform => "Transform",
            Self::Validation => "Validation",
            Self::Preview => "Preview",
            Self::Supp => "SUPP",
        }
    }

    /// Get tab icon (phosphor icon)
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Mapping => egui_phosphor::regular::ARROWS_LEFT_RIGHT,
            Self::Transform => egui_phosphor::regular::SHUFFLE,
            Self::Validation => egui_phosphor::regular::CHECK_SQUARE,
            Self::Preview => egui_phosphor::regular::EYE,
            Self::Supp => egui_phosphor::regular::PLUS_SQUARE,
        }
    }

    /// Get all tabs in order
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

impl AppState {
    /// Navigate to home screen
    pub fn go_home(&mut self) {
        self.view = View::Home;
    }

    /// Navigate to domain editor
    pub fn open_domain(&mut self, domain: String) {
        self.view = View::DomainEditor {
            domain,
            tab: EditorTab::Mapping,
        };
    }

    /// Navigate to export screen
    pub fn go_export(&mut self) {
        self.view = View::Export;
    }

    /// Switch tab in domain editor
    pub fn switch_tab(&mut self, tab: EditorTab) {
        if let View::DomainEditor {
            tab: current_tab, ..
        } = &mut self.view
        {
            *current_tab = tab;
        }
    }

    /// Open the settings window
    pub fn open_settings(&mut self) {
        self.settings_pending = Some(self.settings.clone());
        self.settings_open = true;
    }

    /// Close the settings window
    ///
    /// If `apply` is true, the pending settings are applied and saved.
    /// If `apply` is false, the pending settings are discarded.
    pub fn close_settings(&mut self, apply: bool) {
        if apply {
            if let Some(pending) = self.settings_pending.take() {
                self.settings = pending;
            }
        }
        self.settings_pending = None;
        self.settings_open = false;
    }
}
