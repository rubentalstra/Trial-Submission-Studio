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
use iced::window;
use iced::{Element, Size, Subscription, Task, Theme};

use crate::message::{
    AboutMessage, DialogMessage, DomainEditorMessage, ExportMessage, HomeMessage, Message,
    SettingsCategory, SettingsMessage, ThirdPartyMessage, UpdateMessage,
};
use crate::state::{
    ActiveDialog, AppState, DialogType, DialogWindows, Domain, DomainSource, EditorTab,
    ExportPhase, ExportResult, NotCollectedEdit, Settings, Study, SuppColumnConfig, SuppEditDraft,
    ViewState,
};
use crate::theme::clinical_light;
use crate::view::dialog::update::UpdateState;
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
    /// In daemon mode, we must open the main window explicitly.
    pub fn new() -> (Self, Task<Message>) {
        // Load settings from disk
        let settings = Settings::load();

        let mut app = Self {
            state: AppState::with_settings(settings),
        };

        // Open the main window (daemon mode requires explicit window creation)
        // exit_on_close_request: false allows us to handle close events in our subscription
        let main_window_settings = window::Settings {
            size: Size::new(1280.0, 800.0),
            min_size: Some(Size::new(1024.0, 600.0)),
            icon: load_app_icon(),
            exit_on_close_request: false,
            ..Default::default()
        };

        // window::open returns (Id, Task<Id>)
        let (main_id, open_window_task) = window::open(main_window_settings);

        // Store the main window ID for proper close handling
        app.state.main_window_id = Some(main_id);

        let open_window = open_window_task.map(|_| Message::Noop);
        let init_menu = Task::perform(async {}, |_| Message::InitNativeMenu);

        // Chain the tasks
        let startup = open_window.chain(init_menu);
        (app, startup)
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
            Message::Export(export_msg) => self.handle_export_message(export_msg),

            // =================================================================
            // Dialog messages
            // =================================================================
            Message::Dialog(dialog_msg) => self.handle_dialog_message(dialog_msg),

            // =================================================================
            // Menu messages
            // =================================================================
            Message::Menu(menu_msg) => self.handle_menu_message(menu_msg),

            // =================================================================
            // In-app menu bar messages (Windows/Linux)
            // =================================================================
            Message::MenuBarToggle(menu_id) => {
                self.state.menu_bar.toggle(menu_id);
                Task::none()
            }

            Message::MenuBarClose => {
                self.state.menu_bar.close();
                Task::none()
            }

            Message::NativeMenuEvent => {
                // Poll for native menu events and dispatch
                if let Some(menu_msg) = crate::menu::poll_native_menu_event() {
                    return self.handle_menu_message(menu_msg);
                }
                Task::none()
            }

            Message::InitNativeMenu => {
                // Initialize native menu on macOS
                // This is called via a startup task to ensure proper timing
                #[cfg(target_os = "macos")]
                {
                    let menu = crate::menu::native::create_menu();
                    menu.init_for_nsapp();

                    // Set window menu for proper macOS window management
                    if let Some(window_menu) = crate::menu::native::create_window_submenu() {
                        window_menu.set_as_windows_menu_for_nsapp();
                        std::mem::forget(window_menu);
                    }

                    // Keep the menu alive
                    std::mem::forget(menu);
                    tracing::info!("Initialized native macOS menu bar");
                }
                Task::none()
            }

            // =================================================================
            // Multi-window dialog management
            // =================================================================
            Message::DialogWindowOpened(dialog_type, id) => {
                match dialog_type {
                    DialogType::About => {
                        self.state.dialog_windows.about = Some(id);
                    }
                    DialogType::Settings => {
                        self.state.dialog_windows.settings =
                            Some((id, SettingsCategory::default()));
                    }
                    DialogType::ThirdParty => {
                        self.state.dialog_windows.third_party = Some(id);
                    }
                    DialogType::Update => {
                        self.state.dialog_windows.update = Some((id, UpdateState::Idle));
                    }
                    DialogType::CloseStudyConfirm => {
                        self.state.dialog_windows.close_study_confirm = Some(id);
                    }
                    DialogType::ExportProgress => {
                        // Export progress state is set when export starts
                    }
                    DialogType::ExportComplete => {
                        // Export complete state is set when export completes
                    }
                }
                Task::none()
            }

            Message::DialogWindowClosed(id) => {
                // Check if this is a dialog window
                if self.state.dialog_windows.is_dialog_window(id) {
                    // Clean up state and close the dialog window
                    self.state.dialog_windows.close(id);
                    window::close(id)
                } else if self.state.main_window_id == Some(id) {
                    // This is the main window - exit the application
                    iced::exit()
                } else {
                    // Unknown window (already closed dialog) - just ignore
                    Task::none()
                }
            }

            Message::CloseWindow(id) => {
                // Clean up dialog state before closing the window
                self.state.dialog_windows.close(id);
                window::close(id)
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

    /// Render the view for a specific window.
    ///
    /// This is a pure function that produces UI based on current state.
    /// In multi-window mode, each window gets its own view based on the window ID.
    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        use crate::view::{
            view_about_dialog_content, view_close_study_dialog_content, view_domain_editor,
            view_export, view_export_complete_dialog_content, view_export_progress_dialog_content,
            view_settings_dialog_content, view_third_party_dialog_content,
            view_update_dialog_content,
        };

        // Check if this is a dialog window
        if let Some(dialog_type) = self.state.dialog_windows.dialog_type(id) {
            return match dialog_type {
                DialogType::About => view_about_dialog_content(id),
                DialogType::Settings => {
                    let category = self
                        .state
                        .dialog_windows
                        .settings
                        .as_ref()
                        .map(|(_, cat)| *cat)
                        .unwrap_or_default();
                    view_settings_dialog_content(&self.state.settings, category, id)
                }
                DialogType::ThirdParty => view_third_party_dialog_content(),
                DialogType::Update => {
                    // Get reference to update state from dialog_windows
                    // Use default Idle state if somehow missing
                    if let Some((_, ref update_state)) = self.state.dialog_windows.update {
                        view_update_dialog_content(update_state, id)
                    } else {
                        // Fallback to Idle state - this shouldn't happen
                        view_update_dialog_content(&UpdateState::Idle, id)
                    }
                }
                DialogType::CloseStudyConfirm => view_close_study_dialog_content(id),
                DialogType::ExportProgress => {
                    if let Some((_, ref progress_state)) = self.state.dialog_windows.export_progress
                    {
                        view_export_progress_dialog_content(progress_state, id)
                    } else {
                        // This shouldn't happen - show a simple loading text
                        iced::widget::text("Loading...").into()
                    }
                }
                DialogType::ExportComplete => {
                    if let Some((_, ref result)) = self.state.dialog_windows.export_complete {
                        view_export_complete_dialog_content(result, id)
                    } else {
                        // This shouldn't happen - show a simple close button
                        iced::widget::text("Export dialog").into()
                    }
                }
            };
        }

        // Main window content
        let content: Element<'_, Message> = match &self.state.view {
            ViewState::Home { .. } => view_home(&self.state),
            ViewState::DomainEditor { domain, tab, .. } => {
                view_domain_editor(&self.state, domain, *tab)
            }
            ViewState::Export(_) => view_export(&self.state),
        };

        // On Windows/Linux, add the in-app menu bar at the top
        #[cfg(not(target_os = "macos"))]
        let content_with_menu: Element<'_, Message> = {
            use iced::widget::column;
            let menu_bar =
                crate::menu::in_app::view_menu_bar(&self.state.menu_bar, self.state.has_study());
            column![menu_bar, content].into()
        };

        #[cfg(target_os = "macos")]
        let content_with_menu: Element<'_, Message> = content;

        // Wrap in main container
        container(content_with_menu)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    /// Get the window title for a specific window.
    pub fn title(&self, id: window::Id) -> String {
        // Check if this is a dialog window
        if let Some(dialog_type) = self.state.dialog_windows.dialog_type(id) {
            return match dialog_type {
                DialogType::About => "About Trial Submission Studio".to_string(),
                DialogType::Settings => "Settings".to_string(),
                DialogType::ThirdParty => "Third-Party Licenses".to_string(),
                DialogType::Update => "Check for Updates".to_string(),
                DialogType::CloseStudyConfirm => "Close Study?".to_string(),
                DialogType::ExportProgress => "Exporting...".to_string(),
                DialogType::ExportComplete => "Export Complete".to_string(),
            };
        }

        // Main window title
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

    /// Get the theme for a specific window.
    pub fn theme(&self, _id: window::Id) -> Theme {
        clinical_light()
    }

    /// Subscribe to runtime events.
    pub fn subscription(&self) -> Subscription<Message> {
        use iced::time;
        use std::time::Duration;

        // Keyboard events
        let keyboard_sub = keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Message::KeyPressed(key, modifiers)
            }
            _ => Message::Noop,
        });

        // Native menu event polling (polls every 50ms)
        let menu_sub = time::every(Duration::from_millis(50)).map(|_| Message::NativeMenuEvent);

        // Window close events (for cleaning up dialog windows)
        let window_sub = window::close_requests().map(Message::DialogWindowClosed);

        Subscription::batch([keyboard_sub, menu_sub, window_sub])
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
                // Don't open if already open
                if self.state.dialog_windows.close_study_confirm.is_some() {
                    return Task::none();
                }
                // Open close study confirmation dialog in a new window
                let settings = window::Settings {
                    size: Size::new(350.0, 250.0),
                    resizable: false,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, task) = window::open(settings);
                self.state.dialog_windows.close_study_confirm = Some(id);
                task.map(|_| Message::Noop)
            }

            HomeMessage::CloseStudyConfirmed => {
                // Close the confirmation dialog window if open
                let close_task = if let Some(id) = self.state.dialog_windows.close_study_confirm {
                    self.state.dialog_windows.close_study_confirm = None;
                    window::close(id)
                } else {
                    Task::none()
                };
                // Close the study
                self.state.study = None;
                self.state.view = ViewState::home();
                close_task
            }

            HomeMessage::CloseStudyCancelled => {
                // Close the confirmation dialog window if open
                if let Some(id) = self.state.dialog_windows.close_study_confirm {
                    self.state.dialog_windows.close_study_confirm = None;
                    return window::close(id);
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
                // Start inline editing for new "Not Collected" marking
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.not_collected_edit = Some(NotCollectedEdit {
                        variable,
                        reason: String::new(),
                    });
                }
                Task::none()
            }

            MappingMessage::NotCollectedReasonChanged(reason) => {
                // Update the reason text while editing
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    if let Some(edit) = &mut mapping_ui.not_collected_edit {
                        edit.reason = reason;
                    }
                }
                Task::none()
            }

            MappingMessage::NotCollectedSave { variable, reason } => {
                // Validate reason is not empty
                if reason.trim().is_empty() {
                    return Task::none();
                }

                // Save the "Not Collected" status with reason
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    let _ = domain.mapping.mark_not_collected(&variable, &reason);
                }
                // Clear edit state and invalidate caches
                if let ViewState::DomainEditor {
                    mapping_ui,
                    preview_cache,
                    validation_cache,
                    ..
                } = &mut self.state.view
                {
                    mapping_ui.not_collected_edit = None;
                    *preview_cache = None;
                    *validation_cache = None;
                }
                Task::none()
            }

            MappingMessage::NotCollectedCancel => {
                // Cancel inline editing
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.not_collected_edit = None;
                }
                Task::none()
            }

            MappingMessage::EditNotCollectedReason {
                variable,
                current_reason,
            } => {
                // Start editing an existing "Not Collected" reason
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.not_collected_edit = Some(NotCollectedEdit {
                        variable,
                        reason: current_reason,
                    });
                }
                Task::none()
            }

            MappingMessage::ClearNotCollected(variable) => {
                // Revert "Not Collected" back to unmapped
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
                            // Validate required fields before adding
                            if config.qnam.trim().is_empty() || config.qlabel.trim().is_empty() {
                                // Don't add - QNAM and QLABEL are required
                                return Task::none();
                            }
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
                    // Validate required fields before saving
                    if draft.qnam.trim().is_empty() || draft.qlabel.trim().is_empty() {
                        // Don't save - QNAM and QLABEL are required
                        return Task::none();
                    }

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
            keyboard::Key::Named(Named::Escape) => match &self.state.view {
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
// EXPORT MESSAGE HANDLERS
// =============================================================================

impl App {
    fn handle_export_message(&mut self, msg: ExportMessage) -> Task<Message> {
        match msg {
            ExportMessage::DomainToggled(domain) => {
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.toggle_domain(&domain);
                }
                Task::none()
            }

            ExportMessage::SelectAll => {
                if let Some(study) = &self.state.study {
                    let domains: Vec<String> = study
                        .domain_codes()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect();
                    if let ViewState::Export(export_state) = &mut self.state.view {
                        export_state.select_all(domains);
                    }
                }
                Task::none()
            }

            ExportMessage::DeselectAll => {
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.deselect_all();
                }
                Task::none()
            }

            ExportMessage::FormatChanged(format) => {
                self.state.settings.export.default_format = format;
                let _ = self.state.settings.save();
                Task::none()
            }

            ExportMessage::OutputDirChangeClicked => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Select Output Directory")
                        .pick_folder()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                |path| match path {
                    Some(p) => Message::Export(ExportMessage::OutputDirSelected(p)),
                    None => Message::Noop,
                },
            ),

            ExportMessage::OutputDirSelected(path) => {
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.output_dir = Some(path);
                }
                Task::none()
            }

            ExportMessage::XptVersionChanged(version) => {
                self.state.settings.export.xpt_version = version;
                let _ = self.state.settings.save();
                Task::none()
            }

            ExportMessage::ToggleDefineXml => {
                // Define-XML is always generated - this is a no-op
                Task::none()
            }

            ExportMessage::StartExport => {
                // Get export configuration
                let (selected_domains, output_dir) = match &self.state.view {
                    ViewState::Export(export_state) => {
                        let study_folder = self
                            .state
                            .study
                            .as_ref()
                            .map(|s| s.study_folder.clone())
                            .unwrap_or_default();
                        (
                            export_state.selected_domains.clone(),
                            export_state.effective_output_dir(&study_folder),
                        )
                    }
                    _ => return Task::none(),
                };

                if selected_domains.is_empty() {
                    return Task::none();
                }

                // Get study data
                let study = match &self.state.study {
                    Some(s) => s,
                    None => return Task::none(),
                };

                let study_id = study.study_id.clone();
                let terminology = self.state.terminology.clone();

                // Build export data for each selected domain
                let mut domain_data = Vec::new();
                let mut not_collected_map = std::collections::HashMap::new();

                for code in &selected_domains {
                    if let Some(gui_domain) = study.domain(code) {
                        // Collect not_collected variables for validation
                        let not_collected: std::collections::BTreeSet<String> = gui_domain
                            .mapping
                            .all_not_collected()
                            .keys()
                            .cloned()
                            .collect();
                        if !not_collected.is_empty() {
                            not_collected_map.insert(code.clone(), not_collected);
                        }

                        match crate::service::export::build_domain_export_data(
                            code,
                            gui_domain,
                            &study_id,
                            terminology.as_ref(),
                        ) {
                            Ok(data) => domain_data.push(data),
                            Err(e) => {
                                // Return error immediately
                                return Task::done(Message::Export(ExportMessage::Complete(
                                    ExportResult::Error {
                                        message: e.message,
                                        domain: e.domain,
                                    },
                                )));
                            }
                        }
                    }
                }

                if domain_data.is_empty() {
                    return Task::none();
                }

                // Set exporting state in ViewState
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.phase = ExportPhase::Exporting {
                        current_domain: None,
                        current_step: "Preparing...".to_string(),
                        progress: 0.0,
                        files_written: vec![],
                    };
                }

                // Open progress dialog window
                let settings = window::Settings {
                    size: Size::new(400.0, 300.0),
                    resizable: false,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, open_task) = window::open(settings);

                // Store the progress state with window ID
                self.state.dialog_windows.export_progress = Some((
                    id,
                    crate::state::ExportProgressState {
                        current_domain: None,
                        current_step: "Exporting...".to_string(),
                        progress: 0.0,
                        files_written: 0,
                    },
                ));

                // Build export input with validation settings
                let export_input = crate::service::export::ExportInput {
                    output_dir,
                    format: self.state.settings.export.default_format,
                    xpt_version: self.state.settings.export.xpt_version,
                    domains: domain_data,
                    study_id,
                    bypass_validation: self.state.settings.developer.bypass_validation,
                    ct_registry: terminology,
                    not_collected: not_collected_map,
                };

                // Start actual export task
                let export_task = Task::perform(
                    crate::service::export::execute_export(export_input),
                    |result| Message::Export(ExportMessage::Complete(result)),
                );

                Task::batch([open_task.map(|_| Message::Noop), export_task])
            }

            ExportMessage::CancelExport => {
                // Close progress dialog and open completion dialog with Cancelled result
                let mut tasks = vec![];

                // Close progress dialog if open
                if let Some((id, _)) = self.state.dialog_windows.export_progress.take() {
                    tasks.push(window::close(id));
                }

                // Update view state
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.phase = ExportPhase::Complete(ExportResult::Cancelled);
                }

                // Open completion dialog
                let settings = window::Settings {
                    size: Size::new(400.0, 350.0),
                    resizable: false,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, open_task) = window::open(settings);
                self.state.dialog_windows.export_complete = Some((id, ExportResult::Cancelled));
                tasks.push(open_task.map(|_| Message::Noop));

                Task::batch(tasks)
            }

            ExportMessage::Progress(progress) => {
                // Update both ViewState and dialog window state
                if let ViewState::Export(export_state) = &mut self.state.view {
                    if let ExportPhase::Exporting {
                        current_domain,
                        current_step,
                        progress: prog,
                        files_written,
                    } = &mut export_state.phase
                    {
                        use crate::message::export::ExportProgress;
                        match &progress {
                            ExportProgress::StartingDomain(domain) => {
                                *current_domain = Some(domain.clone());
                            }
                            ExportProgress::Step(step) => {
                                *current_step = step.label().to_string();
                            }
                            ExportProgress::DomainComplete(_domain) => {
                                // Domain done
                            }
                            ExportProgress::OverallProgress(p) => {
                                *prog = *p;
                            }
                        }

                        // Also update dialog window state
                        if let Some((_, ref mut dialog_state)) =
                            self.state.dialog_windows.export_progress
                        {
                            match progress {
                                ExportProgress::StartingDomain(domain) => {
                                    dialog_state.current_domain = Some(domain);
                                }
                                ExportProgress::Step(step) => {
                                    dialog_state.current_step = step.label().to_string();
                                }
                                ExportProgress::DomainComplete(_) => {}
                                ExportProgress::OverallProgress(p) => {
                                    dialog_state.progress = p;
                                }
                            }
                        }
                    }
                }
                Task::none()
            }

            ExportMessage::Complete(result) => {
                let mut tasks = vec![];

                // Close progress dialog if open
                if let Some((id, _)) = self.state.dialog_windows.export_progress.take() {
                    tasks.push(window::close(id));
                }

                // Update view state
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.phase = ExportPhase::Complete(result.clone());
                }

                // Open completion dialog
                let settings = window::Settings {
                    size: Size::new(450.0, 400.0),
                    resizable: false,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, open_task) = window::open(settings);
                self.state.dialog_windows.export_complete = Some((id, result));
                tasks.push(open_task.map(|_| Message::Noop));

                Task::batch(tasks)
            }

            ExportMessage::DismissCompletion => {
                // Close completion dialog
                let task = if let Some((id, _)) = self.state.dialog_windows.export_complete.take() {
                    window::close(id)
                } else {
                    Task::none()
                };

                // Reset view state
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.reset_phase();
                }

                task
            }

            ExportMessage::RetryExport => {
                // Close completion dialog and restart export
                let close_task =
                    if let Some((id, _)) = self.state.dialog_windows.export_complete.take() {
                        window::close(id)
                    } else {
                        Task::none()
                    };

                // Reset view state
                if let ViewState::Export(export_state) = &mut self.state.view {
                    export_state.reset_phase();
                }

                // Batch close and then trigger start export
                Task::batch([
                    close_task,
                    Task::done(Message::Export(ExportMessage::StartExport)),
                ])
            }

            ExportMessage::OpenOutputFolder => {
                // Get output dir from completion dialog state
                if let Some((_, ref result)) = self.state.dialog_windows.export_complete {
                    if let ExportResult::Success { output_dir, .. } = result {
                        let _ = open::that(output_dir);
                    }
                }
                Task::none()
            }
        }
    }
}

// =============================================================================
// DIALOG MESSAGE HANDLERS
// =============================================================================

impl App {
    fn handle_dialog_message(&mut self, msg: DialogMessage) -> Task<Message> {
        match msg {
            DialogMessage::About(about_msg) => self.handle_about_message(about_msg),
            DialogMessage::Settings(settings_msg) => self.handle_settings_message(settings_msg),
            DialogMessage::ThirdParty(tp_msg) => self.handle_third_party_message(tp_msg),
            DialogMessage::Update(update_msg) => self.handle_update_message(update_msg),
            DialogMessage::CloseAll => {
                self.state.active_dialog = None;
                Task::none()
            }
        }
    }

    fn handle_about_message(&mut self, msg: AboutMessage) -> Task<Message> {
        match msg {
            AboutMessage::Open => {
                self.state.active_dialog = Some(ActiveDialog::About);
                Task::none()
            }
            AboutMessage::Close => {
                // Close dialog window in multi-window mode
                if let Some(id) = self.state.dialog_windows.about.take() {
                    return window::close(id);
                }
                self.state.active_dialog = None;
                Task::none()
            }
            AboutMessage::CopyAndClose => {
                // Copy system info to clipboard using Iced's clipboard
                let info = crate::view::dialog::about::generate_system_info();
                let copy_task = iced::clipboard::write(info);

                // Close dialog window in multi-window mode
                if let Some(id) = self.state.dialog_windows.about.take() {
                    return Task::batch([copy_task, window::close(id)]);
                }
                self.state.active_dialog = None;
                copy_task
            }
            AboutMessage::OpenWebsite => {
                let _ = open::that("https://trialsubmissionstudio.com");
                Task::none()
            }
            AboutMessage::OpenGitHub => {
                let _ = open::that("https://github.com/rubentalstra/trial-submission-studio");
                Task::none()
            }
            AboutMessage::OpenOpenSource => {
                // Open the third-party licenses or open source page
                let _ = open::that(
                    "https://github.com/rubentalstra/trial-submission-studio/blob/main/THIRD_PARTY_LICENSES.md",
                );
                Task::none()
            }
        }
    }

    fn handle_settings_message(&mut self, msg: SettingsMessage) -> Task<Message> {
        match msg {
            SettingsMessage::Open => {
                self.state.active_dialog =
                    Some(ActiveDialog::Settings(SettingsCategory::default()));
                Task::none()
            }
            SettingsMessage::Close => {
                // Close dialog window in multi-window mode
                if let Some((id, _)) = self.state.dialog_windows.settings.take() {
                    return window::close(id);
                }
                // Fallback for overlay mode (deprecated)
                self.state.active_dialog = None;
                Task::none()
            }
            SettingsMessage::Apply => {
                // Save settings
                let _ = self.state.settings.save();
                tracing::info!("Settings saved");

                // Close dialog window in multi-window mode
                if let Some((id, _)) = self.state.dialog_windows.settings.take() {
                    return window::close(id);
                }
                // Fallback for overlay mode (deprecated)
                self.state.active_dialog = None;
                Task::none()
            }
            SettingsMessage::ResetToDefaults => {
                self.state.settings = Settings::default();
                Task::none()
            }
            SettingsMessage::CategorySelected(category) => {
                // Update dialog_windows.settings for multi-window mode
                if let Some((id, _)) = self.state.dialog_windows.settings {
                    self.state.dialog_windows.settings = Some((id, category));
                }
                // Also update active_dialog for overlay mode (deprecated)
                self.state.active_dialog = Some(ActiveDialog::Settings(category));
                Task::none()
            }
            SettingsMessage::General(general_msg) => {
                use crate::message::GeneralSettingsMessage;
                match general_msg {
                    GeneralSettingsMessage::CtVersionChanged(_version) => {
                        // CT version change - would reload terminology
                    }
                    GeneralSettingsMessage::HeaderRowsChanged(rows) => {
                        self.state.settings.general.header_rows = rows;
                    }
                }
                Task::none()
            }
            SettingsMessage::Validation(_val_msg) => {
                // Handle validation settings
                Task::none()
            }
            SettingsMessage::Developer(dev_msg) => {
                use crate::message::DeveloperSettingsMessage;
                match dev_msg {
                    DeveloperSettingsMessage::BypassValidationToggled(enabled) => {
                        self.state.settings.developer.bypass_validation = enabled;
                        tracing::info!("Bypass validation: {}", enabled);
                    }
                    DeveloperSettingsMessage::DeveloperModeToggled(enabled) => {
                        self.state.settings.developer.developer_mode = enabled;
                        tracing::info!("Developer mode: {}", enabled);
                    }
                }
                Task::none()
            }
            SettingsMessage::Export(export_msg) => {
                use crate::message::ExportSettingsMessage;
                match export_msg {
                    ExportSettingsMessage::DefaultOutputDirChanged(_dir) => {
                        // Handle output dir change
                    }
                    ExportSettingsMessage::DefaultFormatChanged(format) => {
                        self.state.settings.export.default_format = format;
                    }
                    ExportSettingsMessage::DefaultXptVersionChanged(version) => {
                        self.state.settings.export.xpt_version = version;
                    }
                }
                Task::none()
            }
            SettingsMessage::Display(_display_msg) => {
                // Handle display settings
                Task::none()
            }
            SettingsMessage::Updates(update_msg) => {
                use crate::message::UpdateSettingsMessage;
                match update_msg {
                    UpdateSettingsMessage::AutoCheckToggled(enabled) => {
                        self.state.settings.general.auto_check_updates = enabled;
                    }
                    UpdateSettingsMessage::CheckFrequencyChanged(_freq) => {
                        // Handle frequency change
                    }
                }
                Task::none()
            }
        }
    }

    fn handle_third_party_message(&mut self, msg: ThirdPartyMessage) -> Task<Message> {
        match msg {
            ThirdPartyMessage::Open => {
                self.state.active_dialog = Some(ActiveDialog::ThirdParty);
                Task::none()
            }
            ThirdPartyMessage::Close => {
                self.state.active_dialog = None;
                Task::none()
            }
            ThirdPartyMessage::ScrollTo(_position) => {
                // Handle scroll - would need scrollable state
                Task::none()
            }
        }
    }

    fn handle_update_message(&mut self, msg: UpdateMessage) -> Task<Message> {
        match msg {
            UpdateMessage::Open => {
                self.state.active_dialog = Some(ActiveDialog::Update(UpdateState::Idle));
                Task::none()
            }
            UpdateMessage::Close => {
                self.state.active_dialog = None;
                Task::none()
            }
            UpdateMessage::CheckForUpdates => {
                // Set checking state
                self.state.active_dialog = Some(ActiveDialog::Update(UpdateState::Checking));
                // TODO: Start actual update check
                // For now, simulate up-to-date
                Task::done(Message::Dialog(DialogMessage::Update(
                    UpdateMessage::CheckResult(Ok(None)),
                )))
            }
            UpdateMessage::CheckResult(result) => {
                match result {
                    Ok(Some(info)) => {
                        self.state.active_dialog =
                            Some(ActiveDialog::Update(UpdateState::Available(info)));
                    }
                    Ok(None) => {
                        self.state.active_dialog =
                            Some(ActiveDialog::Update(UpdateState::UpToDate));
                    }
                    Err(err) => {
                        self.state.active_dialog =
                            Some(ActiveDialog::Update(UpdateState::Error(err)));
                    }
                }
                Task::none()
            }
            UpdateMessage::StartInstall => {
                self.state.active_dialog = Some(ActiveDialog::Update(UpdateState::Installing {
                    progress: 0.0,
                }));
                // TODO: Start actual installation
                Task::none()
            }
            UpdateMessage::InstallProgress(progress) => {
                self.state.active_dialog =
                    Some(ActiveDialog::Update(UpdateState::Installing { progress }));
                Task::none()
            }
            UpdateMessage::InstallComplete(result) => {
                match result {
                    Ok(()) => {
                        self.state.active_dialog =
                            Some(ActiveDialog::Update(UpdateState::InstallComplete));
                    }
                    Err(err) => {
                        self.state.active_dialog =
                            Some(ActiveDialog::Update(UpdateState::Error(err)));
                    }
                }
                Task::none()
            }
            UpdateMessage::RestartApp => {
                // Would restart the application
                // For now, just close the dialog
                self.state.active_dialog = None;
                Task::none()
            }
        }
    }
}

// =============================================================================
// MENU MESSAGE HANDLERS
// =============================================================================

impl App {
    fn handle_menu_message(&mut self, msg: crate::message::MenuMessage) -> Task<Message> {
        use crate::message::MenuMessage;

        // Close in-app menu dropdown when any menu action is performed
        self.state.menu_bar.close();

        match msg {
            // File menu
            MenuMessage::OpenStudy => Task::done(Message::Home(HomeMessage::OpenStudyClicked)),
            MenuMessage::CloseStudy => {
                if self.state.has_study() {
                    Task::done(Message::Home(HomeMessage::CloseStudyClicked))
                } else {
                    Task::none()
                }
            }
            MenuMessage::Settings => {
                // Don't open if already open
                if self.state.dialog_windows.settings.is_some() {
                    return Task::none();
                }
                // Open settings dialog in a new window
                let settings = window::Settings {
                    size: Size::new(720.0, 500.0),
                    resizable: false,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, task) = window::open(settings);
                self.state.dialog_windows.settings =
                    Some((id, crate::message::SettingsCategory::default()));
                task.map(|_| Message::Noop)
            }
            MenuMessage::Quit => {
                // Request application quit
                // In Iced, this is typically handled by window close event
                std::process::exit(0);
            }

            // Help menu
            MenuMessage::Documentation => {
                let _ = open::that("https://docs.trialsubmissionstudio.com");
                Task::none()
            }
            MenuMessage::ReleaseNotes => {
                let _ =
                    open::that("https://github.com/rubentalstra/trial-submission-studio/releases");
                Task::none()
            }
            MenuMessage::ViewOnGitHub => {
                let _ = open::that("https://github.com/rubentalstra/trial-submission-studio");
                Task::none()
            }
            MenuMessage::ReportIssue => {
                let _ = open::that(
                    "https://github.com/rubentalstra/trial-submission-studio/issues/new",
                );
                Task::none()
            }
            MenuMessage::ViewLicense => {
                let _ = open::that(
                    "https://github.com/rubentalstra/trial-submission-studio/blob/main/LICENSE",
                );
                Task::none()
            }
            MenuMessage::ThirdPartyLicenses => {
                // Don't open if already open
                if self.state.dialog_windows.third_party.is_some() {
                    return Task::none();
                }
                // Open third-party licenses dialog in a new window
                let settings = window::Settings {
                    size: Size::new(700.0, 550.0),
                    resizable: true,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, task) = window::open(settings);
                self.state.dialog_windows.third_party = Some(id);
                task.map(|_| Message::Noop)
            }
            MenuMessage::CheckUpdates => {
                // Don't open if already open
                if self.state.dialog_windows.update.is_some() {
                    return Task::none();
                }
                // Open update dialog in a new window
                let settings = window::Settings {
                    size: Size::new(450.0, 350.0),
                    resizable: false,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, task) = window::open(settings);
                self.state.dialog_windows.update = Some((id, UpdateState::Idle));
                task.map(|_| Message::Noop)
            }
            MenuMessage::About => {
                // Don't open if already open
                if self.state.dialog_windows.about.is_some() {
                    return Task::none();
                }
                // Open about dialog in a new window
                let settings = window::Settings {
                    size: Size::new(480.0, 300.0),
                    resizable: false,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, task) = window::open(settings);
                self.state.dialog_windows.about = Some(id);
                task.map(|_| Message::Noop)
            }

            // Edit menu (these typically interact with focused widget - noop for now)
            MenuMessage::Undo
            | MenuMessage::Redo
            | MenuMessage::Cut
            | MenuMessage::Copy
            | MenuMessage::Paste
            | MenuMessage::SelectAll => {
                // These are typically handled by the text input widgets themselves
                // through the native edit menu on macOS or platform-specific mechanisms
                Task::none()
            }
        }
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

/// Load the application icon from embedded PNG data.
fn load_app_icon() -> Option<window::Icon> {
    let icon_data = include_bytes!("../assets/icon.png");
    window::icon::from_file_data(icon_data, Some(image::ImageFormat::Png)).ok()
}
