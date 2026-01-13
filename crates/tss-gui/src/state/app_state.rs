//! Application-level state.
//!
//! This module contains `AppState` which is the root of all state.

use std::sync::mpsc::{Receiver, Sender};
use tss_model::TerminologyRegistry;

use super::navigation::{EditorTab, View, WorkflowMode};
use super::ui_state::{ExportError, ExportPhase, ExportResult};
use super::{DomainState, StudyState, UiState};

// TODO: These imports will be restored when services/export are ported to Iced
// use crate::export::{ExportPhase, ExportUpdate};
// use crate::services::PreviewResult;
// use crate::settings::Settings;

/// Placeholder for Settings until the module is ported
#[derive(Debug, Clone)]
pub struct Settings {
    pub general: GeneralSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
        }
    }
}

/// Placeholder for GeneralSettings
#[derive(Debug, Clone)]
pub struct GeneralSettings {
    pub header_rows: usize,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self { header_rows: 1 }
    }
}

/// Placeholder for PreviewResult
pub struct PreviewResult {
    pub domain_code: String,
    pub result: Result<polars::prelude::DataFrame, String>,
}

/// Placeholder for ExportUpdate
pub enum ExportUpdate {
    Progress {
        domain: String,
        step: String,
    },
    FileWritten {
        path: std::path::PathBuf,
    },
    Complete {
        files_written: Vec<std::path::PathBuf>,
        domains_exported: usize,
    },
    Error {
        message: String,
        domain: Option<String>,
    },
    Cancelled,
}

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
    /// Channel for receiving preview results from background threads
    pub preview_receiver: Receiver<PreviewResult>,
    /// Sender for spawning preview tasks (cloned to background threads)
    pub preview_sender: Sender<PreviewResult>,
    /// Channel for receiving export updates from background threads
    pub export_receiver: Receiver<ExportUpdate>,
    /// Sender for spawning export tasks (cloned to background threads)
    pub export_sender: Sender<ExportUpdate>,
}

impl Default for AppState {
    fn default() -> Self {
        let (preview_sender, preview_receiver) = std::sync::mpsc::channel();
        let (export_sender, export_receiver) = std::sync::mpsc::channel();
        Self {
            view: View::default(),
            workflow_mode: WorkflowMode::default(),
            study: None,
            settings: Settings::default(),
            ui: UiState::default(),
            ct_registry: None,
            preview_receiver,
            preview_sender,
            export_receiver,
            export_sender,
        }
    }
}

impl AppState {
    /// Create new app state with loaded settings.
    pub fn new(settings: Settings) -> Self {
        let (preview_sender, preview_receiver) = std::sync::mpsc::channel();
        let (export_sender, export_receiver) = std::sync::mpsc::channel();
        Self {
            view: View::default(),
            workflow_mode: WorkflowMode::default(),
            study: None,
            settings,
            ui: UiState::default(),
            ct_registry: None,
            preview_receiver,
            preview_sender,
            export_receiver,
            export_sender,
        }
    }

    /// Poll for export updates from background thread.
    ///
    /// Call this each frame when export is in progress.
    pub fn poll_export_updates(&mut self) {
        while let Ok(update) = self.export_receiver.try_recv() {
            match update {
                ExportUpdate::Progress { domain, step } => {
                    self.ui.export.current_domain = Some(domain);
                    self.ui.export.current_step = step;
                }
                ExportUpdate::FileWritten { path } => {
                    self.ui.export.written_files.push(path);
                }
                ExportUpdate::Complete {
                    files_written,
                    domains_exported,
                } => {
                    self.ui.export.result = Some(Ok(ExportResult {
                        files_written,
                        domains_exported,
                    }));
                    self.ui.export.phase = ExportPhase::Complete;
                }
                ExportUpdate::Error { message, domain } => {
                    self.ui.export.result = Some(Err(ExportError { message, domain }));
                    self.ui.export.phase = ExportPhase::Complete;
                }
                ExportUpdate::Cancelled => {
                    self.ui.export.reset();
                }
            }
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

    /// Invalidate preview and all dependent data.
    ///
    /// Called by mapping tab when user makes changes. This clears cached data
    /// immediately (instant, no computation). The actual rebuild happens lazily
    /// when user switches to Preview/Transform/Validation tabs.
    pub fn invalidate_preview(&mut self, domain_code: &str) {
        // Clear domain data
        if let Some(domain) = self.study_mut().and_then(|s| s.get_domain_mut(domain_code)) {
            domain.derived.preview = None;
            domain.derived.validation = None;
        }
        // Clear UI state (in case of rapid changes while rebuilding)
        self.ui.domain_editor(domain_code).preview.is_rebuilding = false;
        self.ui.domain_editor(domain_code).preview.error = None;
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
            self.ct_registry = tss_standards::load_ct(tss_standards::CtVersion::default()).ok();
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

// Note: WorkflowMode, View, and EditorTab are now defined in navigation.rs
// and imported at the top of this file.
