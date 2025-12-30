//! Application-level state

use super::StudyState;

/// Top-level application state
#[derive(Default)]
pub struct AppState {
    /// Current view/screen
    pub view: View,
    /// Loaded study (None if no study loaded)
    pub study: Option<StudyState>,
    /// User preferences
    pub preferences: Preferences,
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
    /// Get display name for the tab
    pub fn label(&self) -> &'static str {
        match self {
            Self::Mapping => "Mapping",
            Self::Transform => "Transform",
            Self::Validation => "Validation",
            Self::Preview => "Preview",
            Self::Supp => "SUPP",
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

/// User preferences (persisted to disk)
#[derive(Clone)]
pub struct Preferences {
    /// Dark mode enabled
    pub dark_mode: bool,
    /// Recent study folders
    pub recent_studies: Vec<std::path::PathBuf>,
    /// Maximum recent studies to remember
    pub max_recent: usize,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            dark_mode: false,
            recent_studies: Vec::new(),
            max_recent: 10,
        }
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
}
