//! Application-level state

use super::StudyState;
use crate::settings::{ExportFormat, Settings, ui::SettingsWindow};
use std::collections::HashSet;
use std::path::PathBuf;

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
    /// Export state
    pub export_state: ExportState,
}

/// Export operation state
#[derive(Default, Clone)]
pub struct ExportState {
    /// Domains selected for export (domain codes)
    pub selected_domains: HashSet<String>,
    /// Override output directory (if different from default)
    pub output_dir_override: Option<PathBuf>,
    /// Override export format (if different from settings default)
    pub format_override: Option<ExportFormat>,
    /// Current export progress (None if not exporting)
    pub progress: Option<ExportProgress>,
}

/// Export progress tracking
#[derive(Clone)]
pub struct ExportProgress {
    /// Current step description
    pub current_step: String,
    /// Current domain being processed
    pub current_domain: Option<String>,
    /// Total domains to process
    pub total_domains: usize,
    /// Completed domains count
    pub completed_domains: usize,
    /// Individual step within current domain
    pub domain_step: ExportDomainStep,
    /// Any error that occurred
    pub error: Option<String>,
    /// Export completed successfully
    pub completed: bool,
    /// Output files created
    pub output_files: Vec<PathBuf>,
}

/// Steps within a domain export
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportDomainStep {
    #[default]
    Pending,
    ApplyingMappings,
    NormalizingCT,
    GeneratingVariables,
    ValidatingOutput,
    WritingXpt,
    WritingDefineXml,
    Complete,
}

impl ExportDomainStep {
    /// Get display label for this step
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::ApplyingMappings => "Applying mappings",
            Self::NormalizingCT => "Normalizing terminology",
            Self::GeneratingVariables => "Generating derived variables",
            Self::ValidatingOutput => "Validating output",
            Self::WritingXpt => "Writing XPT file",
            Self::WritingDefineXml => "Generating Define-XML",
            Self::Complete => "Complete",
        }
    }
}

impl ExportProgress {
    /// Create a new export progress tracker
    pub fn new(total_domains: usize) -> Self {
        Self {
            current_step: "Preparing export...".to_string(),
            current_domain: None,
            total_domains,
            completed_domains: 0,
            domain_step: ExportDomainStep::Pending,
            error: None,
            completed: false,
            output_files: Vec::new(),
        }
    }

    /// Get overall progress as a fraction (0.0 to 1.0)
    pub fn fraction(&self) -> f32 {
        if self.total_domains == 0 {
            return 1.0;
        }
        let domain_fraction = self.completed_domains as f32 / self.total_domains as f32;
        let step_fraction = match self.domain_step {
            ExportDomainStep::Pending => 0.0,
            ExportDomainStep::ApplyingMappings => 0.15,
            ExportDomainStep::NormalizingCT => 0.30,
            ExportDomainStep::GeneratingVariables => 0.45,
            ExportDomainStep::ValidatingOutput => 0.60,
            ExportDomainStep::WritingXpt => 0.80,
            ExportDomainStep::WritingDefineXml => 0.95,
            ExportDomainStep::Complete => 1.0,
        };
        // Each domain contributes equally to overall progress
        let per_domain = 1.0 / self.total_domains as f32;
        domain_fraction + (step_fraction * per_domain)
    }
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
            export_state: ExportState::default(),
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
            export_state: ExportState::default(),
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
