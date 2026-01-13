//! Main application module for Trial Submission Studio.
//!
//! This module implements the Iced 0.14.0 application using the builder pattern.
//! The architecture follows the Elm pattern: State → Message → Update → View.
//!
//! # Key Design Principles
//!
//! - **All state changes happen in `update()`** - Views are pure functions
//! - **No channels/polling** - Use `Task::perform` for async operations
//! - **View state is part of ViewState enum** - Not separate UiState struct

use std::path::PathBuf;

use iced::keyboard;
use iced::widget::{column, container, text};
use iced::{Element, Subscription, Task, Theme};

use crate::message::{DomainEditorMessage, HomeMessage, Message};
use crate::state::{
    AppState, Domain, DomainSource, EditorTab, NotCollectedDialog, Settings, Study,
    SuppColumnConfig, SuppEditDraft, ViewState,
};
use crate::theme::clinical_light;
use crate::view::view_home;

// =============================================================================
// APPLICATION
// =============================================================================

/// Main application struct.
///
/// This is the root of the Iced application. It holds the application state
/// and implements the Elm architecture methods.
pub struct App {
    /// All application state.
    pub state: AppState,
}

impl App {
    /// Create a new application instance.
    ///
    /// Called once at startup. Returns the initial state and any startup tasks.
    pub fn new() -> (Self, Task<Message>) {
        // Load settings from disk
        let settings = Settings::load();

        let app = Self {
            state: AppState::with_settings(settings),
        };

        // TODO: Add startup tasks (auto-update check, etc.)
        (app, Task::none())
    }

    /// Update application state in response to a message.
    ///
    /// This is the core of the Elm architecture - all state changes happen here.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // =================================================================
            // Navigation
            // =================================================================
            Message::Navigate(view_state) => {
                self.state.view = view_state;
                Task::none()
            }

            Message::SetWorkflowMode(mode) => {
                if let ViewState::Home { workflow_mode, .. } = &mut self.state.view {
                    *workflow_mode = mode;
                }
                Task::none()
            }

            // =================================================================
            // Home view messages
            // =================================================================
            Message::Home(home_msg) => self.handle_home_message(home_msg),

            // =================================================================
            // Domain editor messages
            // =================================================================
            Message::DomainEditor(editor_msg) => self.handle_domain_editor_message(editor_msg),

            // =================================================================
            // Export messages
            // =================================================================
            Message::Export(_export_msg) => {
                // TODO: Implement in Phase 5
                Task::none()
            }

            // =================================================================
            // Dialog messages
            // =================================================================
            Message::Dialog(_dialog_msg) => {
                // TODO: Implement in Phase 5
                Task::none()
            }

            // =================================================================
            // Menu messages
            // =================================================================
            Message::Menu(_menu_msg) => {
                // TODO: Implement in Phase 6
                Task::none()
            }

            // =================================================================
            // Background task results
            // =================================================================
            Message::StudyLoaded(result) => {
                self.state.is_loading = false;
                match result {
                    Ok((study, terminology)) => {
                        tracing::info!(
                            "Study loaded: {} with {} domains",
                            study.study_id,
                            study.domain_count()
                        );

                        // Add to recent studies
                        self.state
                            .settings
                            .general
                            .add_recent(study.study_folder.clone());
                        let _ = self.state.settings.save();

                        self.state.study = Some(study);
                        self.state.terminology = Some(terminology);
                        self.state.view = ViewState::home();
                    }
                    Err(err) => {
                        tracing::error!("Failed to load study: {}", err);
                        self.state.error = Some(err);
                    }
                }
                Task::none()
            }

            Message::PreviewReady { domain, result } => {
                if let ViewState::DomainEditor {
                    domain: current_domain,
                    preview_cache,
                    preview_ui,
                    ..
                } = &mut self.state.view
                {
                    if current_domain == &domain {
                        preview_ui.is_rebuilding = false;
                        match result {
                            Ok(df) => {
                                *preview_cache = Some(df);
                                preview_ui.error = None;
                            }
                            Err(e) => {
                                preview_ui.error = Some(e);
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::ValidationComplete { domain, report } => {
                if let ViewState::DomainEditor {
                    domain: current_domain,
                    validation_cache,
                    ..
                } = &mut self.state.view
                {
                    if current_domain == &domain {
                        *validation_cache = Some(report);
                    }
                }
                Task::none()
            }

            Message::UpdateCheckComplete(_result) => {
                // TODO: Implement in Phase 5
                Task::none()
            }

            // =================================================================
            // Global events
            // =================================================================
            Message::KeyPressed(key, modifiers) => self.handle_key_press(key, modifiers),

            Message::FolderSelected(path) => {
                if let Some(folder) = path {
                    self.load_study(folder)
                } else {
                    Task::none()
                }
            }

            Message::DismissError => {
                self.state.error = None;
                Task::none()
            }

            Message::Noop => Task::none(),
        }
    }

    /// Render the current view.
    ///
    /// This is a pure function that produces UI based on current state.
    pub fn view(&self) -> Element<'_, Message> {
        use crate::view::view_domain_editor;

        let content: Element<'_, Message> = match &self.state.view {
            ViewState::Home { .. } => view_home(&self.state),
            ViewState::DomainEditor { domain, tab, .. } => {
                view_domain_editor(&self.state, domain, *tab)
            }
            ViewState::Export(_) => self.view_export(),
        };

        // Wrap in main container
        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    /// Get the window title.
    pub fn title(&self) -> String {
        let study_name = self
            .state
            .study
            .as_ref()
            .map(|s| s.study_id.as_str())
            .unwrap_or("");

        match &self.state.view {
            ViewState::Home { .. } if study_name.is_empty() => {
                "Trial Submission Studio".to_string()
            }
            ViewState::Home { .. } => {
                format!("{} - Trial Submission Studio", study_name)
            }
            ViewState::DomainEditor { domain, .. } => {
                format!("{} ({}) - Trial Submission Studio", domain, study_name)
            }
            ViewState::Export(_) => {
                format!("Export - {} - Trial Submission Studio", study_name)
            }
        }
    }

    /// Get the current theme.
    pub fn theme(&self) -> Theme {
        clinical_light()
    }

    /// Subscribe to runtime events.
    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Message::KeyPressed(key, modifiers)
            }
            _ => Message::Noop,
        })
    }
}

// =============================================================================
// HOME MESSAGE HANDLERS
// =============================================================================

impl App {
    fn handle_home_message(&mut self, msg: HomeMessage) -> Task<Message> {
        match msg {
            HomeMessage::OpenStudyClicked => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Select Study Folder")
                        .pick_folder()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                Message::FolderSelected,
            ),

            HomeMessage::StudyFolderSelected(path) => self.load_study(path),

            HomeMessage::RecentStudyClicked(path) => self.load_study(path),

            HomeMessage::CloseStudyClicked => {
                if let ViewState::Home { close_confirm, .. } = &mut self.state.view {
                    *close_confirm = true;
                }
                Task::none()
            }

            HomeMessage::CloseStudyConfirmed => {
                self.state.study = None;
                self.state.view = ViewState::home();
                Task::none()
            }

            HomeMessage::CloseStudyCancelled => {
                if let ViewState::Home { close_confirm, .. } = &mut self.state.view {
                    *close_confirm = false;
                }
                Task::none()
            }

            HomeMessage::DomainClicked(domain) => {
                self.state.view = ViewState::domain_editor(domain, EditorTab::Mapping);
                Task::none()
            }

            HomeMessage::GoToExportClicked => {
                self.state.view = ViewState::export();
                Task::none()
            }

            HomeMessage::RemoveFromRecent(path) => {
                self.state.settings.general.remove_recent(&path);
                let _ = self.state.settings.save();
                Task::none()
            }

            HomeMessage::ClearRecentStudies => {
                self.state.settings.general.clear_recent();
                let _ = self.state.settings.save();
                Task::none()
            }
        }
    }

    /// Load a study from a folder path.
    fn load_study(&mut self, path: PathBuf) -> Task<Message> {
        self.state.is_loading = true;
        let header_rows = self.state.settings.general.header_rows;

        Task::perform(
            async move { load_study_async(path, header_rows).await },
            Message::StudyLoaded,
        )
    }
}

// =============================================================================
// DOMAIN EDITOR MESSAGE HANDLERS
// =============================================================================

impl App {
    fn handle_domain_editor_message(&mut self, msg: DomainEditorMessage) -> Task<Message> {
        match msg {
            DomainEditorMessage::TabSelected(tab) => {
                if let ViewState::DomainEditor {
                    tab: current_tab, ..
                } = &mut self.state.view
                {
                    *current_tab = tab;
                }
                Task::none()
            }

            DomainEditorMessage::BackClicked => {
                self.state.view = ViewState::home();
                Task::none()
            }

            DomainEditorMessage::Mapping(mapping_msg) => self.handle_mapping_message(mapping_msg),

            DomainEditorMessage::Normalization(norm_msg) => {
                self.handle_normalization_message(norm_msg)
            }

            DomainEditorMessage::Validation(validation_msg) => {
                self.handle_validation_message(validation_msg)
            }

            DomainEditorMessage::Preview(preview_msg) => self.handle_preview_message(preview_msg),

            DomainEditorMessage::Supp(supp_msg) => self.handle_supp_message(supp_msg),
        }
    }

    fn handle_mapping_message(
        &mut self,
        msg: crate::message::domain_editor::MappingMessage,
    ) -> Task<Message> {
        use crate::message::domain_editor::MappingMessage;

        // Get current domain code
        let domain_code = match &self.state.view {
            ViewState::DomainEditor { domain, .. } => domain.clone(),
            _ => return Task::none(),
        };

        match msg {
            MappingMessage::VariableSelected(idx) => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.selected_variable = Some(idx);
                }
                Task::none()
            }

            MappingMessage::SearchChanged(text) => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.search_filter = text;
                }
                Task::none()
            }

            MappingMessage::SearchCleared => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.search_filter.clear();
                }
                Task::none()
            }

            MappingMessage::AcceptSuggestion(variable) => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    let _ = domain.mapping.accept_suggestion(&variable);
                }
                // Invalidate caches
                if let ViewState::DomainEditor {
                    preview_cache,
                    validation_cache,
                    ..
                } = &mut self.state.view
                {
                    *preview_cache = None;
                    *validation_cache = None;
                }
                Task::none()
            }

            MappingMessage::ClearMapping(variable) => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    domain.mapping.clear_assignment(&variable);
                }
                // Invalidate caches
                if let ViewState::DomainEditor {
                    preview_cache,
                    validation_cache,
                    ..
                } = &mut self.state.view
                {
                    *preview_cache = None;
                    *validation_cache = None;
                }
                Task::none()
            }

            MappingMessage::ManualMap { variable, column } => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    let _ = domain.mapping.accept_manual(&variable, &column);
                }
                // Invalidate caches
                if let ViewState::DomainEditor {
                    preview_cache,
                    validation_cache,
                    ..
                } = &mut self.state.view
                {
                    *preview_cache = None;
                    *validation_cache = None;
                }
                Task::none()
            }

            MappingMessage::MarkNotCollected { variable } => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.not_collected_dialog = Some(NotCollectedDialog {
                        variable,
                        reason: String::new(),
                    });
                }
                Task::none()
            }

            MappingMessage::NotCollectedConfirmed { variable, reason } => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    let _ = domain.mapping.mark_not_collected(&variable, &reason);
                }
                if let ViewState::DomainEditor {
                    mapping_ui,
                    preview_cache,
                    validation_cache,
                    ..
                } = &mut self.state.view
                {
                    mapping_ui.not_collected_dialog = None;
                    *preview_cache = None;
                    *validation_cache = None;
                }
                Task::none()
            }

            MappingMessage::NotCollectedCancelled => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.not_collected_dialog = None;
                }
                Task::none()
            }

            MappingMessage::MarkOmitted(variable) => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    let _ = domain.mapping.mark_omit(&variable);
                }
                // Invalidate caches
                if let ViewState::DomainEditor {
                    preview_cache,
                    validation_cache,
                    ..
                } = &mut self.state.view
                {
                    *preview_cache = None;
                    *validation_cache = None;
                }
                Task::none()
            }

            MappingMessage::ClearOmitted(variable) => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    domain.mapping.clear_assignment(&variable);
                }
                // Invalidate caches
                if let ViewState::DomainEditor {
                    preview_cache,
                    validation_cache,
                    ..
                } = &mut self.state.view
                {
                    *preview_cache = None;
                    *validation_cache = None;
                }
                Task::none()
            }

            MappingMessage::FilterUnmappedToggled(enabled) => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.filter_unmapped = enabled;
                }
                Task::none()
            }

            MappingMessage::FilterRequiredToggled(enabled) => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.filter_required = enabled;
                }
                Task::none()
            }
        }
    }

    fn handle_normalization_message(
        &mut self,
        msg: crate::message::domain_editor::NormalizationMessage,
    ) -> Task<Message> {
        use crate::message::domain_editor::NormalizationMessage;

        match msg {
            NormalizationMessage::RuleSelected(index) => {
                if let ViewState::DomainEditor {
                    normalization_ui, ..
                } = &mut self.state.view
                {
                    normalization_ui.selected_rule = Some(index);
                }
                Task::none()
            }

            NormalizationMessage::RuleToggled { .. } => {
                // TODO: Implement rule toggling if needed
                Task::none()
            }

            NormalizationMessage::RefreshPreview => {
                // TODO: Implement preview refresh if needed
                Task::none()
            }
        }
    }

    fn handle_preview_message(
        &mut self,
        msg: crate::message::domain_editor::PreviewMessage,
    ) -> Task<Message> {
        use crate::message::domain_editor::PreviewMessage;
        use crate::service::preview::{PreviewInput, compute_preview};

        // Get current domain code
        let domain_code = match &self.state.view {
            ViewState::DomainEditor { domain, .. } => domain.clone(),
            _ => return Task::none(),
        };

        match msg {
            PreviewMessage::RebuildPreview => {
                // Get domain data for preview
                let domain = match self
                    .state
                    .study
                    .as_ref()
                    .and_then(|s| s.domain(&domain_code))
                {
                    Some(d) => d,
                    None => return Task::none(),
                };

                // Mark as rebuilding
                if let ViewState::DomainEditor { preview_ui, .. } = &mut self.state.view {
                    preview_ui.is_rebuilding = true;
                    preview_ui.error = None;
                }

                // Build preview input
                let input = PreviewInput {
                    source_df: domain.source.data.clone(),
                    mapping: domain.mapping.clone(),
                    ct_registry: self.state.terminology.clone(),
                };

                let domain_for_result = domain_code.clone();

                // Start async preview computation
                Task::perform(compute_preview(input), move |result| {
                    Message::PreviewReady {
                        domain: domain_for_result,
                        result: result.map_err(|e| e.to_string()),
                    }
                })
            }

            PreviewMessage::GoToPage(page) => {
                if let ViewState::DomainEditor { preview_ui, .. } = &mut self.state.view {
                    preview_ui.current_page = page;
                }
                Task::none()
            }

            PreviewMessage::NextPage => {
                if let ViewState::DomainEditor { preview_ui, .. } = &mut self.state.view {
                    preview_ui.current_page = preview_ui.current_page.saturating_add(1);
                }
                Task::none()
            }

            PreviewMessage::PreviousPage => {
                if let ViewState::DomainEditor { preview_ui, .. } = &mut self.state.view {
                    preview_ui.current_page = preview_ui.current_page.saturating_sub(1);
                }
                Task::none()
            }

            PreviewMessage::RowsPerPageChanged(rows) => {
                if let ViewState::DomainEditor { preview_ui, .. } = &mut self.state.view {
                    preview_ui.rows_per_page = rows;
                    preview_ui.current_page = 0; // Reset to first page
                }
                Task::none()
            }
        }
    }

    fn handle_validation_message(
        &mut self,
        msg: crate::message::domain_editor::ValidationMessage,
    ) -> Task<Message> {
        use crate::message::domain_editor::ValidationMessage;
        use crate::service::validation::{ValidationInput, compute_validation};

        // Get current domain code
        let domain_code = match &self.state.view {
            ViewState::DomainEditor { domain, .. } => domain.clone(),
            _ => return Task::none(),
        };

        match msg {
            ValidationMessage::RefreshValidation => {
                // Get domain data
                let domain = match self
                    .state
                    .study
                    .as_ref()
                    .and_then(|s| s.domain(&domain_code))
                {
                    Some(d) => d,
                    None => return Task::none(),
                };

                // Get preview DataFrame (validation runs on transformed data)
                let df = match &self.state.view {
                    ViewState::DomainEditor {
                        preview_cache: Some(df),
                        ..
                    } => df.clone(),
                    _ => {
                        // Fall back to source data if no preview available
                        domain.source.data.clone()
                    }
                };

                // Get SDTM domain definition
                let sdtm_domain = domain.mapping.domain().clone();

                // Get not collected variables (convert from BTreeMap to BTreeSet)
                let not_collected: std::collections::BTreeSet<String> =
                    domain.mapping.all_not_collected().keys().cloned().collect();

                // Build input
                let input = ValidationInput {
                    domain: sdtm_domain,
                    df,
                    ct_registry: self.state.terminology.clone(),
                    not_collected,
                };

                let domain_for_result = domain_code.clone();

                // Start async validation
                Task::perform(compute_validation(input), move |report| {
                    Message::ValidationComplete {
                        domain: domain_for_result,
                        report,
                    }
                })
            }

            ValidationMessage::IssueSelected(idx) => {
                if let ViewState::DomainEditor { validation_ui, .. } = &mut self.state.view {
                    validation_ui.selected_issue = Some(idx);
                }
                Task::none()
            }

            ValidationMessage::SeverityFilterChanged(filter) => {
                if let ViewState::DomainEditor { validation_ui, .. } = &mut self.state.view {
                    validation_ui.severity_filter = match filter {
                        crate::message::domain_editor::SeverityFilter::All => {
                            crate::state::SeverityFilter::All
                        }
                        crate::message::domain_editor::SeverityFilter::Errors => {
                            crate::state::SeverityFilter::Errors
                        }
                        crate::message::domain_editor::SeverityFilter::Warnings => {
                            crate::state::SeverityFilter::Warnings
                        }
                        crate::message::domain_editor::SeverityFilter::Info => {
                            crate::state::SeverityFilter::Info
                        }
                    };
                }
                Task::none()
            }

            ValidationMessage::GoToIssueSource { variable } => {
                // Navigate to mapping tab and select the variable
                if let ViewState::DomainEditor {
                    tab, mapping_ui, ..
                } = &mut self.state.view
                {
                    *tab = EditorTab::Mapping;
                    // Try to find and select the variable by name
                    if let Some(domain) = self
                        .state
                        .study
                        .as_ref()
                        .and_then(|s| s.domain(&domain_code))
                    {
                        let sdtm_domain = domain.mapping.domain();
                        if let Some(idx) = sdtm_domain
                            .variables
                            .iter()
                            .position(|v| v.name == variable)
                        {
                            mapping_ui.selected_variable = Some(idx);
                        }
                    }
                }
                Task::none()
            }
        }
    }

    /// Handle SUPP tab messages.
    ///
    /// # Message Flow
    ///
    /// - **Pending columns**: Field edits update `supp_config` directly
    /// - **Included columns (editing)**: Field edits update `edit_draft`, committed on Save
    /// - **Actions**: AddToSupp, Skip, UndoAction change `supp_config.action`
    fn handle_supp_message(
        &mut self,
        msg: crate::message::domain_editor::SuppMessage,
    ) -> Task<Message> {
        use crate::message::domain_editor::SuppMessage;
        use crate::state::{SuppAction, SuppColumnConfig, SuppEditDraft};

        // Get current domain code
        let domain_code = match &self.state.view {
            ViewState::DomainEditor { domain, .. } => domain.clone(),
            _ => return Task::none(),
        };

        match msg {
            // =================================================================
            // NAVIGATION & FILTERING
            // =================================================================
            SuppMessage::ColumnSelected(col_name) => {
                // Clear any edit draft when changing selection
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.selected_column = Some(col_name.clone());
                    supp_ui.edit_draft = None;
                }
                // Initialize config if not exists
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    domain
                        .supp_config
                        .entry(col_name.clone())
                        .or_insert_with(|| SuppColumnConfig::from_column(&col_name));
                }
                Task::none()
            }

            SuppMessage::SearchChanged(text) => {
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.search_filter = text;
                }
                Task::none()
            }

            SuppMessage::FilterModeChanged(mode) => {
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.filter_mode = mode;
                }
                Task::none()
            }

            // =================================================================
            // FIELD EDITING
            // =================================================================
            SuppMessage::QnamChanged(value) => {
                // Enforce max 8 chars, uppercase
                let value = value.chars().take(8).collect::<String>().to_uppercase();
                self.update_supp_field(&domain_code, |config, draft| {
                    if let Some(d) = draft {
                        d.qnam = value;
                    } else {
                        config.qnam = value;
                    }
                });
                Task::none()
            }

            SuppMessage::QlabelChanged(value) => {
                // Enforce max 40 chars
                let value: String = value.chars().take(40).collect();
                self.update_supp_field(&domain_code, |config, draft| {
                    if let Some(d) = draft {
                        d.qlabel = value;
                    } else {
                        config.qlabel = value;
                    }
                });
                Task::none()
            }

            SuppMessage::QorigChanged(value) => {
                self.update_supp_field(&domain_code, |config, draft| {
                    if let Some(d) = draft {
                        d.qorig = value;
                    } else {
                        config.qorig = value;
                    }
                });
                Task::none()
            }

            SuppMessage::QevalChanged(value) => {
                self.update_supp_field(&domain_code, |config, draft| {
                    if let Some(d) = draft {
                        d.qeval = value.clone();
                    } else {
                        config.qeval = if value.is_empty() { None } else { Some(value) };
                    }
                });
                Task::none()
            }

            // =================================================================
            // ACTIONS
            // =================================================================
            SuppMessage::AddToSupp => {
                // Get selected column
                let col = match &self.state.view {
                    ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                    _ => None,
                };

                if let Some(col_name) = col {
                    if let Some(domain) = self
                        .state
                        .study
                        .as_mut()
                        .and_then(|s| s.domain_mut(&domain_code))
                    {
                        if let Some(config) = domain.supp_config.get_mut(&col_name) {
                            config.action = SuppAction::Include;
                        }
                    }
                }
                // Clear draft after action
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.edit_draft = None;
                }
                Task::none()
            }

            SuppMessage::Skip => {
                let col = match &self.state.view {
                    ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                    _ => None,
                };

                if let Some(col_name) = col {
                    if let Some(domain) = self
                        .state
                        .study
                        .as_mut()
                        .and_then(|s| s.domain_mut(&domain_code))
                    {
                        if let Some(config) = domain.supp_config.get_mut(&col_name) {
                            config.action = SuppAction::Skip;
                        }
                    }
                }
                // Clear draft after action
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.edit_draft = None;
                }
                Task::none()
            }

            SuppMessage::UndoAction => {
                let col = match &self.state.view {
                    ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                    _ => None,
                };

                if let Some(col_name) = col {
                    if let Some(domain) = self
                        .state
                        .study
                        .as_mut()
                        .and_then(|s| s.domain_mut(&domain_code))
                    {
                        if let Some(config) = domain.supp_config.get_mut(&col_name) {
                            config.action = SuppAction::Pending;
                        }
                    }
                }
                // Clear draft after action
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.edit_draft = None;
                }
                Task::none()
            }

            // =================================================================
            // EDIT MODE (for included columns)
            // =================================================================
            SuppMessage::StartEdit => {
                // Get selected column and create draft from its config
                let col = match &self.state.view {
                    ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                    _ => None,
                };

                if let Some(col_name) = &col {
                    if let Some(domain) = self
                        .state
                        .study
                        .as_ref()
                        .and_then(|s| s.domain(&domain_code))
                    {
                        if let Some(config) = domain.supp_config.get(col_name) {
                            let draft = SuppEditDraft::from_config(config);
                            if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                                supp_ui.edit_draft = Some(draft);
                            }
                        }
                    }
                }
                Task::none()
            }

            SuppMessage::SaveEdit => {
                // Apply draft to config
                let (col, draft) = match &self.state.view {
                    ViewState::DomainEditor { supp_ui, .. } => {
                        (supp_ui.selected_column.clone(), supp_ui.edit_draft.clone())
                    }
                    _ => (None, None),
                };

                if let (Some(col_name), Some(draft)) = (col, draft) {
                    if let Some(domain) = self
                        .state
                        .study
                        .as_mut()
                        .and_then(|s| s.domain_mut(&domain_code))
                    {
                        if let Some(config) = domain.supp_config.get_mut(&col_name) {
                            config.qnam = draft.qnam;
                            config.qlabel = draft.qlabel;
                            config.qorig = draft.qorig;
                            config.qeval = if draft.qeval.is_empty() {
                                None
                            } else {
                                Some(draft.qeval)
                            };
                        }
                    }
                }
                // Clear draft
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.edit_draft = None;
                }
                Task::none()
            }

            SuppMessage::CancelEdit => {
                // Just discard the draft
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.edit_draft = None;
                }
                Task::none()
            }
        }
    }

    /// Helper to update a SUPP field, routing to draft or config as appropriate.
    fn update_supp_field<F>(&mut self, domain_code: &str, update: F)
    where
        F: FnOnce(&mut SuppColumnConfig, Option<&mut SuppEditDraft>),
    {
        // Get selected column
        let col = match &self.state.view {
            ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
            _ => return,
        };

        let Some(col_name) = col else { return };

        // Check if we're in edit mode (have a draft)
        let is_editing = match &self.state.view {
            ViewState::DomainEditor { supp_ui, .. } => supp_ui.edit_draft.is_some(),
            _ => false,
        };

        if is_editing {
            // Update the draft
            if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                if let Some(draft) = &mut supp_ui.edit_draft {
                    // Get a dummy config to satisfy the closure signature
                    let mut dummy = SuppColumnConfig::from_column("");
                    update(&mut dummy, Some(draft));
                }
            }
        } else {
            // Update the config directly
            if let Some(domain) = self
                .state
                .study
                .as_mut()
                .and_then(|s| s.domain_mut(domain_code))
            {
                if let Some(config) = domain.supp_config.get_mut(&col_name) {
                    update(config, None);
                }
            }
        }
    }
}

// =============================================================================
// KEYBOARD HANDLERS
// =============================================================================

impl App {
    fn handle_key_press(
        &mut self,
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    ) -> Task<Message> {
        use keyboard::key::Named;

        match key.as_ref() {
            // Cmd/Ctrl+O: Open study
            keyboard::Key::Character("o") if modifiers.command() => {
                Task::done(Message::Home(HomeMessage::OpenStudyClicked))
            }

            // Cmd/Ctrl+W: Close study
            keyboard::Key::Character("w") if modifiers.command() => {
                if self.state.has_study() {
                    Task::done(Message::Home(HomeMessage::CloseStudyClicked))
                } else {
                    Task::none()
                }
            }

            // Cmd/Ctrl+E: Export
            keyboard::Key::Character("e") if modifiers.command() => {
                if self.state.has_study() {
                    Task::done(Message::Navigate(ViewState::export()))
                } else {
                    Task::none()
                }
            }

            // Escape: Go home or close dialogs
            keyboard::Key::Named(Named::Escape) => match &mut self.state.view {
                ViewState::Home { close_confirm, .. } if *close_confirm => {
                    *close_confirm = false;
                    Task::none()
                }
                ViewState::DomainEditor { .. } | ViewState::Export(_) => {
                    Task::done(Message::Navigate(ViewState::home()))
                }
                _ => Task::none(),
            },

            _ => Task::none(),
        }
    }
}

// =============================================================================
// VIEW IMPLEMENTATIONS (Placeholder)
// =============================================================================

impl App {
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
// ASYNC STUDY LOADING
// =============================================================================

use tss_model::TerminologyRegistry;

/// Load a study asynchronously, including CT loading.
async fn load_study_async(
    folder: PathBuf,
    header_rows: usize,
) -> Result<(Study, TerminologyRegistry), String> {
    // Create study from folder
    let mut study = Study::from_folder(folder.clone());

    // Discover CSV files
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

    // Load metadata if available
    study.metadata = tss_ingest::load_study_metadata(&folder, header_rows).ok();

    // Load SDTM-IG
    let ig_domains =
        tss_standards::load_sdtm_ig().map_err(|e| format!("Failed to load SDTM-IG: {}", e))?;

    // Load Controlled Terminology
    let ct_version = tss_standards::ct::CtVersion::default();
    let terminology = tss_standards::ct::load(ct_version).map_err(|e| {
        format!(
            "Failed to load Controlled Terminology ({}): {}",
            ct_version, e
        )
    })?;
    tracing::info!(
        "Loaded CT {} with {} catalogs",
        ct_version,
        terminology.catalogs.len()
    );

    // Process each CSV file
    for csv_path in csv_files {
        let file_stem = csv_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        // Extract domain code from filename
        // Handles both simple names (DM.csv) and prefixed names (STUDY_DM.csv)
        let domain_code = extract_domain_code(file_stem);

        // Skip non-domain files
        if domain_code.is_empty()
            || domain_code.starts_with('_')
            || domain_code.eq_ignore_ascii_case("items")
            || domain_code.eq_ignore_ascii_case("codelists")
        {
            continue;
        }

        let domain_code = domain_code.to_uppercase();

        // Load CSV
        let (df, _headers) = tss_ingest::read_csv_table(&csv_path, header_rows)
            .map_err(|e| format!("Failed to load {}: {}", domain_code, e))?;

        // Find domain in SDTM-IG
        let ig_domain = ig_domains
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(&domain_code));

        let Some(ig_domain) = ig_domain else {
            tracing::warn!("Domain {} not found in SDTM-IG, skipping", domain_code);
            continue;
        };

        // Create source
        let source = DomainSource::new(csv_path, df.clone(), ig_domain.label.clone());

        // Create mapping state
        let hints = tss_ingest::build_column_hints(&df);
        let source_columns: Vec<String> = df
            .get_column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let mapping = tss_map::MappingState::new(
            ig_domain.clone(),
            &study.study_id,
            &source_columns,
            hints,
            0.6,
        );

        // Create domain and add to study
        let domain = Domain::new(source, mapping);
        study.add_domain(domain_code, domain);
    }

    if study.domain_count() == 0 {
        return Err("No valid SDTM domains found in the selected folder".to_string());
    }

    Ok((study, terminology))
}

/// Extract domain code from a filename.
///
/// Handles various naming conventions:
/// - Simple: `DM.csv` → `DM`
/// - Prefixed: `STUDY_DM.csv` → `DM`
/// - Full path: `DEMO_GDISC_20240903_072908_DM.csv` → `DM`
///
/// Returns the last underscore-separated segment.
fn extract_domain_code(file_stem: &str) -> &str {
    // If there's no underscore, return the whole string
    if !file_stem.contains('_') {
        return file_stem;
    }

    // Return the last segment after underscore
    file_stem.rsplit('_').next().unwrap_or(file_stem)
}
