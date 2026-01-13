//! Main application module for Trial Submission Studio.
//!
//! This module implements the Iced 0.14.0 application using the builder pattern.
//! The architecture follows the Elm pattern: State → Message → Update → View.

use std::path::PathBuf;

use iced::keyboard;
use iced::widget::{column, container, text};
use iced::{Element, Subscription, Task, Theme};

use crate::message::{HomeMessage, Message};
use crate::state::AppState;
use crate::state::navigation::{EditorTab, View};
use crate::theme::clinical_light;
use crate::view::view_home;

// =============================================================================
// APPLICATION STATE
// =============================================================================

/// Main application state.
///
/// This is the root state container for Trial Submission Studio.
/// It wraps `AppState` which holds all domain and UI state.
pub struct App {
    /// All application state (domain data, UI state, settings)
    state: AppState,
}

// =============================================================================
// APPLICATION IMPLEMENTATION
// =============================================================================

impl App {
    /// Create a new application instance.
    ///
    /// This is called once at startup. Returns the initial state and any
    /// startup tasks (e.g., loading settings, checking for updates).
    pub fn new() -> (Self, Task<Message>) {
        let state = AppState::default();
        let app = Self { state };

        // TODO: Add startup tasks
        // - Load settings from disk
        // - Check for updates (if enabled)
        // - Load recent studies list
        let startup_tasks = Task::none();

        (app, startup_tasks)
    }

    /// Update application state in response to a message.
    ///
    /// This is the core of the Elm architecture - all state changes happen here.
    /// Returns any follow-up tasks to execute.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation
            Message::Navigate(view) => {
                self.state.view = view;
                Task::none()
            }

            // Workflow mode change
            Message::SetWorkflowMode(mode) => {
                self.state.set_workflow_mode(mode);
                Task::none()
            }

            // Home view messages
            Message::Home(home_msg) => self.handle_home_message(home_msg),

            // Domain editor messages
            Message::DomainEditor(editor_msg) => self.handle_domain_editor_message(editor_msg),

            // Export messages
            Message::Export(export_msg) => self.handle_export_message(export_msg),

            // Dialog messages
            Message::Dialog(dialog_msg) => self.handle_dialog_message(dialog_msg),

            // Menu messages
            Message::Menu(menu_msg) => self.handle_menu_message(menu_msg),

            // Background task results
            Message::StudyLoaded(result) => self.handle_study_loaded(result),

            Message::PreviewReady { domain, result } => self.handle_preview_ready(&domain, result),

            Message::ValidationComplete { domain, report } => {
                self.handle_validation_complete(&domain, report)
            }

            Message::UpdateCheckComplete(result) => self.handle_update_check_complete(result),

            // Global events
            Message::KeyPressed(key, modifiers) => self.handle_key_press(key, modifiers),

            Message::Tick => {
                // Periodic tick for polling export updates
                self.state.poll_export_updates();
                Task::none()
            }

            // File dialog result
            Message::FolderSelected(path) => {
                if let Some(folder) = path {
                    self.load_study(folder)
                } else {
                    Task::none()
                }
            }

            // No-op placeholder
            Message::Noop => Task::none(),
        }
    }

    /// Render the current view.
    ///
    /// This is a pure function that produces the UI based on current state.
    /// No side effects should happen here.
    pub fn view(&self) -> Element<'_, Message> {
        // Main content based on current view
        let content: Element<'_, Message> = match &self.state.view {
            View::Home => view_home(&self.state),
            View::DomainEditor { domain, tab } => self.view_domain_editor(domain, *tab),
            View::Export => self.view_export(),
        };

        // Wrap in main container
        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    /// Get the window title.
    ///
    /// This can change based on current state (e.g., show study name).
    pub fn title(&self) -> String {
        let study_name = self
            .state
            .study
            .as_ref()
            .map(|s| s.study_id.as_str())
            .unwrap_or("");

        match &self.state.view {
            View::Home if study_name.is_empty() => "Trial Submission Studio".to_string(),
            View::Home => format!("{} - Trial Submission Studio", study_name),
            View::DomainEditor { domain, .. } => {
                format!("{} ({}) - Trial Submission Studio", domain, study_name)
            }
            View::Export => format!("Export - {} - Trial Submission Studio", study_name),
        }
    }

    /// Get the current theme.
    ///
    /// Returns the Professional Clinical light theme.
    pub fn theme(&self) -> Theme {
        clinical_light()
    }

    /// Subscribe to runtime events.
    ///
    /// This sets up event listeners for keyboard shortcuts, timers, etc.
    pub fn subscription(&self) -> Subscription<Message> {
        // Keyboard event subscription using Iced 0.14.0 API
        keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Message::KeyPressed(key, modifiers)
            }
            _ => Message::Noop,
        })
    }
}

// =============================================================================
// VIEW IMPLEMENTATIONS (Placeholder for Phase 4 & 5)
// =============================================================================

impl App {
    /// Render the domain editor view.
    fn view_domain_editor(&self, domain: &str, tab: EditorTab) -> Element<'_, Message> {
        // Placeholder - will be implemented in Phase 4
        column![
            text(format!("Domain: {}", domain)).size(24),
            text(format!("Tab: {}", tab.name())).size(16),
        ]
        .spacing(16)
        .padding(32)
        .into()
    }

    /// Render the export view.
    fn view_export(&self) -> Element<'_, Message> {
        // Placeholder - will be implemented in Phase 5
        column![
            text("Export").size(24),
            text("Configure and export your domains.").size(16),
        ]
        .spacing(16)
        .padding(32)
        .into()
    }
}

// =============================================================================
// HOME MESSAGE HANDLERS
// =============================================================================

impl App {
    fn handle_home_message(&mut self, msg: HomeMessage) -> Task<Message> {
        match msg {
            HomeMessage::OpenStudyClicked => {
                // Open native file dialog
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Select Study Folder")
                            .pick_folder()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    Message::FolderSelected,
                )
            }

            HomeMessage::StudyFolderSelected(path) => self.load_study(path),

            HomeMessage::RecentStudyClicked(path) => self.load_study(path),

            HomeMessage::CloseStudyClicked => {
                self.state.ui.close_study_confirm = true;
                Task::none()
            }

            HomeMessage::CloseStudyConfirmed => {
                self.state.ui.close_study_confirm = false;
                self.state.study = None;
                self.state.ui.clear_domain_editors();
                self.state.ui.export.reset();
                self.state.view = View::Home;
                Task::none()
            }

            HomeMessage::CloseStudyCancelled => {
                self.state.ui.close_study_confirm = false;
                Task::none()
            }

            HomeMessage::DomainClicked(domain) => {
                self.state.view = View::DomainEditor {
                    domain,
                    tab: EditorTab::Mapping,
                };
                Task::none()
            }

            HomeMessage::GoToExportClicked => {
                self.state.view = View::Export;
                Task::none()
            }

            HomeMessage::RemoveFromRecent(_path) => {
                // TODO: Implement in Phase 6 (settings)
                Task::none()
            }

            HomeMessage::ClearRecentStudies => {
                // TODO: Implement in Phase 6 (settings)
                Task::none()
            }
        }
    }

    /// Load a study from a folder path.
    fn load_study(&mut self, path: PathBuf) -> Task<Message> {
        let settings_header_rows = self.state.settings.general.header_rows;

        Task::perform(
            async move { load_study_async(path, settings_header_rows).await },
            Message::StudyLoaded,
        )
    }
}

// =============================================================================
// STUDY LOADING (Async)
// =============================================================================

/// Load a study asynchronously.
///
/// This function runs in a background task and returns a `StudyState`
/// or an error message.
async fn load_study_async(
    folder: PathBuf,
    header_rows: usize,
) -> Result<crate::state::StudyState, String> {
    use crate::state::{DomainSource, DomainState, StudyState};

    // Create study state from folder
    let mut study = StudyState::from_folder(folder.clone());

    // Discover CSV files in the folder
    let csv_files: Vec<PathBuf> = std::fs::read_dir(&folder)
        .map_err(|e| format!("Failed to read folder: {}", e))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .map(|ext| ext.eq_ignore_ascii_case("csv"))
                .unwrap_or(false)
        })
        .collect();

    if csv_files.is_empty() {
        return Err("No CSV files found in the selected folder".to_string());
    }

    // Load metadata if available (Items.csv, CodeLists.csv)
    study.metadata = tss_ingest::load_study_metadata(&folder, header_rows).ok();

    // Load SDTM-IG for mapping
    let ig_domains =
        tss_standards::load_sdtm_ig().map_err(|e| format!("Failed to load SDTM-IG: {}", e))?;

    // Process each CSV file
    for csv_path in csv_files {
        // Extract domain code from filename (e.g., "DM.csv" -> "DM")
        let file_stem = csv_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_uppercase())
            .unwrap_or_default();

        // Skip non-domain files
        if file_stem.is_empty()
            || file_stem.starts_with('_')
            || file_stem.eq_ignore_ascii_case("items")
            || file_stem.eq_ignore_ascii_case("codelists")
        {
            continue;
        }

        // Load the CSV file
        let (df, _headers) = tss_ingest::read_csv_table(&csv_path, header_rows)
            .map_err(|e| format!("Failed to load {}: {}", file_stem, e))?;

        // Find domain in SDTM-IG
        let ig_domain = ig_domains
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(&file_stem));

        // Get domain label from IG
        let label = ig_domain.and_then(|d| d.label.clone());

        // Create domain source
        let source = DomainSource::new(csv_path, df.clone(), label);

        // Create mapping state
        let mapping = if let Some(domain) = ig_domain {
            // Build column hints for better matching
            let hints = tss_ingest::build_column_hints(&df);
            let source_columns: Vec<String> = df
                .get_column_names()
                .into_iter()
                .map(|s| s.to_string())
                .collect();

            tss_map::MappingState::new(
                domain.clone(),
                &study.study_id,
                &source_columns,
                hints,
                0.6, // minimum confidence for auto-suggestions
            )
        } else {
            // Unknown domain - skip
            tracing::warn!("Domain {} not found in SDTM-IG, skipping", file_stem);
            continue;
        };

        // Create domain state
        let domain_state = DomainState::new(source, mapping);
        study.add_domain(file_stem, domain_state);
    }

    if study.domains.is_empty() {
        return Err("No valid SDTM domains found in the selected folder".to_string());
    }

    Ok(study)
}

// =============================================================================
// OTHER MESSAGE HANDLERS (Placeholder)
// =============================================================================

impl App {
    fn handle_domain_editor_message(
        &mut self,
        _msg: crate::message::DomainEditorMessage,
    ) -> Task<Message> {
        // TODO: Implement in Phase 4
        Task::none()
    }

    fn handle_export_message(&mut self, _msg: crate::message::ExportMessage) -> Task<Message> {
        // TODO: Implement in Phase 5
        Task::none()
    }

    fn handle_dialog_message(&mut self, _msg: crate::message::DialogMessage) -> Task<Message> {
        // TODO: Implement in Phase 5
        Task::none()
    }

    fn handle_menu_message(&mut self, _msg: crate::message::MenuMessage) -> Task<Message> {
        // TODO: Implement in Phase 6
        Task::none()
    }

    fn handle_study_loaded(
        &mut self,
        result: Result<crate::state::StudyState, String>,
    ) -> Task<Message> {
        match result {
            Ok(study) => {
                tracing::info!(
                    "Study loaded: {} with {} domains",
                    study.study_id,
                    study.domains.len()
                );
                self.state.set_study(study);
            }
            Err(err) => {
                tracing::error!("Failed to load study: {}", err);
                // TODO: Show error dialog
            }
        }
        Task::none()
    }

    fn handle_preview_ready(
        &mut self,
        _domain: &str,
        _result: Result<polars::prelude::DataFrame, String>,
    ) -> Task<Message> {
        // TODO: Implement in Phase 4
        Task::none()
    }

    fn handle_validation_complete(
        &mut self,
        _domain: &str,
        _report: tss_validate::ValidationReport,
    ) -> Task<Message> {
        // TODO: Implement in Phase 4
        Task::none()
    }

    fn handle_update_check_complete(
        &mut self,
        _result: Result<Option<crate::message::UpdateInfo>, String>,
    ) -> Task<Message> {
        // TODO: Implement in Phase 5
        Task::none()
    }

    fn handle_key_press(
        &mut self,
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    ) -> Task<Message> {
        // Global keyboard shortcuts
        use keyboard::key::Named;

        match key.as_ref() {
            // Cmd/Ctrl+O: Open study
            keyboard::Key::Character("o") if modifiers.command() => {
                Task::done(Message::Menu(crate::message::MenuMessage::OpenStudy))
            }

            // Cmd/Ctrl+W: Close study
            keyboard::Key::Character("w") if modifiers.command() => {
                Task::done(Message::Menu(crate::message::MenuMessage::CloseStudy))
            }

            // Cmd/Ctrl+,: Settings
            keyboard::Key::Character(",") if modifiers.command() => {
                Task::done(Message::Menu(crate::message::MenuMessage::Settings))
            }

            // Cmd/Ctrl+E: Export
            keyboard::Key::Character("e") if modifiers.command() => {
                Task::done(Message::Navigate(View::Export))
            }

            // Escape: Go home or close dialog
            keyboard::Key::Named(Named::Escape) => {
                if self.state.ui.close_study_confirm {
                    self.state.ui.close_study_confirm = false;
                    Task::none()
                } else if self.state.view.is_domain_editor() || self.state.view.is_export() {
                    Task::done(Message::Navigate(View::Home))
                } else {
                    Task::none()
                }
            }

            _ => Task::none(),
        }
    }
}
