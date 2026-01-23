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
//!
//! # Module Structure
//!
//! - `handler/` - Message handlers organized by category
//! - `util` - Utility functions (study loading, icon loading)

// Submodules - handlers are organized by category in handler/
mod handler;
pub mod util;

// Re-export utility functions for internal use
use util::load_app_icon;

use iced::keyboard;
use iced::widget::container;
use iced::window;
use iced::{Element, Size, Subscription, Task, Theme};

use crate::handler::{
    DialogHandler, DomainEditorHandler, ExportHandler, HomeHandler, MenuActionHandler,
    MessageHandler, SourceAssignmentHandler,
};
use crate::message::{Message, SettingsCategory};
use crate::state::{AppState, DialogType, Settings, ViewState};
use crate::theme::clinical_theme;
use crate::view::dialog::third_party::ThirdPartyState;
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

        // Check for post-update status and show toast if update was successful
        if let Some(toast) = check_update_status() {
            app.state.toast = Some(toast);
        }

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
            Message::Home(home_msg) => HomeHandler.handle(&mut self.state, home_msg),

            // =================================================================
            // Source assignment view messages
            // =================================================================
            Message::SourceAssignment(assignment_msg) => {
                SourceAssignmentHandler.handle(&mut self.state, assignment_msg)
            }

            // =================================================================
            // Domain editor messages
            // =================================================================
            Message::DomainEditor(editor_msg) => {
                DomainEditorHandler.handle(&mut self.state, editor_msg)
            }

            // =================================================================
            // Export messages
            // =================================================================
            Message::Export(export_msg) => ExportHandler.handle(&mut self.state, export_msg),

            // =================================================================
            // Project persistence
            // =================================================================
            Message::NewProject => crate::handler::project::handle_new_project(&mut self.state),

            Message::OpenProject => crate::handler::project::handle_open_project(&mut self.state),

            Message::OpenProjectSelected(path) => {
                crate::handler::project::handle_open_project_selected(&mut self.state, path)
            }

            Message::ProjectLoaded(result) => {
                crate::handler::project::handle_project_loaded(&mut self.state, result)
            }

            Message::SaveProject => crate::handler::project::handle_save_project(&mut self.state),

            Message::SaveProjectAs => {
                crate::handler::project::handle_save_project_as(&mut self.state)
            }

            Message::SavePathSelected(path) => {
                crate::handler::project::handle_save_path_selected(&mut self.state, path)
            }

            Message::ProjectSaved(result) => {
                crate::handler::project::handle_project_saved(&mut self.state, result)
            }

            Message::AutoSaveTick => {
                crate::handler::project::handle_auto_save_tick(&mut self.state)
            }

            Message::SourceFilesChanged(files) => {
                // Show a warning toast about changed source files
                tracing::warn!("Source files changed since last save: {:?}", files);
                let msg = format!(
                    "{} source file(s) changed since last save. Mappings may be outdated.",
                    files.len()
                );
                self.state.toast =
                    Some(crate::component::feedback::toast::ToastState::warning(msg));
                Task::none()
            }

            // =================================================================
            // Unsaved changes dialog
            // =================================================================
            Message::UnsavedChangesSave => {
                crate::handler::project::handle_unsaved_changes_save(&mut self.state)
            }

            Message::UnsavedChangesDiscard => {
                crate::handler::project::handle_unsaved_changes_discard(&mut self.state)
            }

            Message::UnsavedChangesCancelled => {
                crate::handler::project::handle_unsaved_changes_cancelled(&mut self.state)
            }

            // =================================================================
            // Dialog messages
            // =================================================================
            Message::Dialog(dialog_msg) => DialogHandler.handle(&mut self.state, dialog_msg),

            // =================================================================
            // Menu messages
            // =================================================================
            Message::MenuAction(action) => MenuActionHandler.handle(&mut self.state, action),

            Message::InitNativeMenu => {
                // Initialize native menu on macOS
                // This is called via a startup task to ensure proper timing
                #[cfg(target_os = "macos")]
                {
                    // create_menu() handles all initialization including:
                    // - Creating the menu structure
                    // - Initializing for NSApp
                    // - Setting up the window menu
                    // - Storing references in thread-local storage
                    let _menu = crate::menu::create_menu();

                    // Initialize the channel-based menu event system
                    // This spawns a background thread that forwards muda events
                    crate::menu::init_menu_channel();

                    // Populate recent projects submenu from saved settings
                    let projects: Vec<_> = self
                        .state
                        .settings
                        .general
                        .recent_projects_sorted()
                        .iter()
                        .map(|p| crate::menu::RecentProjectInfo::new(p.id, p.display_name.clone()))
                        .collect();
                    crate::menu::update_recent_projects_menu(&projects);

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
                        self.state.dialog_windows.third_party = Some((id, ThirdPartyState::new()));
                    }
                    DialogType::Update => {
                        self.state.dialog_windows.update = Some((id, UpdateState::Checking));
                    }
                    DialogType::CloseProjectConfirm => {
                        self.state.dialog_windows.close_project_confirm = Some(id);
                    }
                    DialogType::ExportProgress => {
                        // Export progress state is set when export starts
                    }
                    DialogType::ExportComplete => {
                        // Export complete state is set when export completes
                    }
                    DialogType::UnsavedChanges => {
                        // Unsaved changes state is set when dialog is opened
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
                    // This is the main window - check for unsaved changes before exiting
                    if self.state.dirty_tracker.is_dirty() && self.state.study.is_some() {
                        // Show unsaved changes dialog with quit action
                        crate::handler::project::show_unsaved_changes_dialog_for_quit(
                            &mut self.state,
                        )
                    } else {
                        // No unsaved changes - exit immediately
                        iced::exit()
                    }
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

                        // Store the study - it will be added to recent projects when saved
                        self.state.study = Some(study);
                        self.state.terminology = Some(terminology);
                        self.state.view = ViewState::home();

                        // Check if there's a pending project restoration
                        // (when opening a .tss file, we need to apply saved mappings)
                        if let Some((_, project)) = self.state.pending_project_restore.take() {
                            // Check for changed source files
                            let changed_files =
                                crate::handler::project::detect_changed_source_files(&project);

                            if !changed_files.is_empty() {
                                tracing::warn!(
                                    "Source files changed since last save: {:?}",
                                    changed_files
                                );
                                // Show a warning to the user via toast
                                let msg = format!(
                                    "{} source file(s) changed since last save. Mappings may be outdated.",
                                    changed_files.len()
                                );
                                self.state.toast = Some(
                                    crate::component::feedback::toast::ToastState::warning(msg),
                                );
                            }

                            // Restore mappings regardless (user can re-map if needed)
                            crate::handler::project::restore_project_mappings(
                                &mut self.state,
                                &project,
                            );
                        }
                    }
                    Err(err) => {
                        tracing::error!("Failed to load study: {}", err);
                        self.state.error = Some(crate::error::GuiError::study_load(err));
                        // Clear any pending restore on error
                        self.state.pending_project_restore = None;
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
                    && current_domain == &domain
                {
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
                Task::none()
            }

            Message::ValidationComplete { domain, report } => {
                // Store validation in DomainState so it persists across navigation
                if let Some(study) = &mut self.state.study
                    && let Some(domain_state) = study.domain_mut(&domain)
                {
                    domain_state.validation_cache = Some(report);
                }
                Task::none()
            }

            Message::UpdateCheckComplete(_result) => {
                // This is now handled via UpdateMessage::CheckComplete in dialog handler
                Task::none()
            }

            Message::UpdateReadyToInstall {
                info,
                data,
                verified,
            } => {
                // Update dialog state to ReadyToInstall
                if let Some((id, _)) = self.state.dialog_windows.update {
                    self.state.dialog_windows.update = Some((
                        id,
                        UpdateState::ReadyToInstall {
                            info,
                            data,
                            verified,
                        },
                    ));
                }
                Task::none()
            }

            // =================================================================
            // Global events
            // =================================================================
            Message::SystemThemeChanged(mode) => {
                self.state.system_is_dark = matches!(mode, iced::theme::Mode::Dark);
                Task::none()
            }

            Message::KeyPressed(key, modifiers) => self.handle_key_press(key, modifiers),

            Message::FolderSelected(path) => {
                if let Some(folder) = path {
                    crate::handler::home::load_study(&mut self.state, folder)
                } else {
                    Task::none()
                }
            }

            Message::DismissError => {
                self.state.error = None;
                Task::none()
            }

            // =================================================================
            // External actions
            // =================================================================
            Message::OpenUrl(url) => {
                let _ = open::that(&url);
                Task::none()
            }

            // =================================================================
            // Toast notifications
            // =================================================================
            Message::Toast(toast_msg) => self.handle_toast_message(toast_msg),

            Message::Noop => Task::none(),
        }
    }

    /// Handle toast notification messages.
    fn handle_toast_message(&mut self, msg: crate::message::ToastMessage) -> Task<Message> {
        use crate::component::feedback::toast::{ToastActionType, ToastMessage};

        match msg {
            ToastMessage::Dismiss => {
                self.state.toast = None;
                Task::none()
            }
            ToastMessage::Action => {
                // Handle the action based on the current toast
                if let Some(toast) = &self.state.toast
                    && let Some(action) = &toast.action
                {
                    match &action.on_click {
                        ToastActionType::ViewChangelog => {
                            // Open the update dialog
                            self.state.toast = None;
                            return MenuActionHandler
                                .handle(&mut self.state, crate::menu::MenuAction::CheckUpdates);
                        }
                        ToastActionType::OpenUrl(url) => {
                            let _ = open::that(url);
                        }
                    }
                }
                self.state.toast = None;
                Task::none()
            }
            ToastMessage::Show(toast_state) => {
                self.state.toast = Some(toast_state);
                Task::none()
            }
        }
    }

    /// Render the view for a specific window.
    ///
    /// This is a pure function that produces UI based on current state.
    /// In multi-window mode, each window gets its own view based on the window ID.
    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        use crate::view::{
            view_about_dialog_content, view_close_project_dialog_content, view_domain_editor,
            view_export, view_export_complete_dialog_content, view_export_progress_dialog_content,
            view_settings_dialog_content, view_third_party_dialog_content,
            view_unsaved_changes_dialog_content, view_update_dialog_content,
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
                DialogType::ThirdParty => {
                    if let Some((_, ref third_party_state)) = self.state.dialog_windows.third_party
                    {
                        view_third_party_dialog_content(third_party_state)
                    } else {
                        iced::widget::text("Loading...").into()
                    }
                }
                DialogType::Update => {
                    // Get reference to update state from dialog_windows
                    if let Some((_, ref update_state)) = self.state.dialog_windows.update {
                        view_update_dialog_content(update_state, id)
                    } else {
                        // This shouldn't happen - show loading text as fallback
                        iced::widget::text("Loading...").into()
                    }
                }
                DialogType::CloseProjectConfirm => view_close_project_dialog_content(id),
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
                DialogType::UnsavedChanges => view_unsaved_changes_dialog_content(id),
            };
        }

        // Main window content
        let content: Element<'_, Message> = match &self.state.view {
            ViewState::Home { .. } => view_home(&self.state),
            ViewState::SourceAssignment { .. } => crate::view::view_source_assignment(&self.state),
            ViewState::DomainEditor { domain, tab, .. } => {
                view_domain_editor(&self.state, domain, *tab)
            }
            ViewState::Export(_) => view_export(&self.state),
        };

        // On Windows/Linux, add the in-app menu bar at the top
        #[cfg(not(target_os = "macos"))]
        let content_with_menu: Element<'_, Message> = {
            use iced::widget::column;
            let menu_bar = crate::menu::view_menu_bar(
                &self.state.menu_dropdown,
                self.state.has_study(),
                &self.state,
            );
            column![menu_bar, content].into()
        };

        #[cfg(target_os = "macos")]
        let content_with_menu: Element<'_, Message> = content;

        // If there's a toast, wrap content with an overlay
        if let Some(toast) = &self.state.toast {
            use crate::component::feedback::toast::view_toast;
            use iced::widget::{Space, column, stack};

            let toast_element = view_toast(toast);

            // Position toast at bottom-right using a row with flex space
            let toast_row = iced::widget::row![
                Space::new().width(iced::Length::Fill),
                container(toast_element).padding([0.0, 24.0]),
            ];

            let toast_container = column![Space::new().height(iced::Length::Fill), toast_row,];

            // Stack the toast on top of the content
            return stack![
                container(content_with_menu)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill),
                toast_container,
            ]
            .into();
        }

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
                DialogType::CloseProjectConfirm => "Close Project?".to_string(),
                DialogType::ExportProgress => "Exporting...".to_string(),
                DialogType::ExportComplete => "Export Complete".to_string(),
                DialogType::UnsavedChanges => "Unsaved Changes".to_string(),
            };
        }

        // Dirty indicator - show "*" in title when there are unsaved changes
        let dirty_indicator = if self.state.dirty_tracker.is_dirty() {
            " *"
        } else {
            ""
        };

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
                format!(
                    "{}{} - Trial Submission Studio",
                    study_name, dirty_indicator
                )
            }
            ViewState::SourceAssignment { .. } => {
                format!(
                    "Assign Source Files{} - Trial Submission Studio",
                    dirty_indicator
                )
            }
            ViewState::DomainEditor { domain, .. } => {
                format!(
                    "{} ({}){} - Trial Submission Studio",
                    domain, study_name, dirty_indicator
                )
            }
            ViewState::Export(_) => {
                format!(
                    "Export - {}{} - Trial Submission Studio",
                    study_name, dirty_indicator
                )
            }
        }
    }

    /// Get the theme for a specific window.
    pub fn theme(&self, _id: window::Id) -> Theme {
        clinical_theme(
            self.state.theme_config.theme_mode,
            self.state.theme_config.accessibility_mode,
            self.state.system_is_dark,
        )
    }

    /// Subscribe to runtime events.
    pub fn subscription(&self) -> Subscription<Message> {
        use iced::{system, time};
        use std::time::Duration;

        // Keyboard events
        let keyboard_sub = keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Message::KeyPressed(key, modifiers)
            }
            _ => Message::Noop,
        });

        // System theme changes (for ThemeMode::System)
        let system_theme_sub = system::theme_changes().map(Message::SystemThemeChanged);

        // Native menu event polling (macOS only, polls every 50ms for responsiveness)
        #[cfg(target_os = "macos")]
        let menu_sub = crate::menu::menu_subscription().map(|action| match action {
            Some(a) => Message::MenuAction(a),
            None => Message::Noop,
        });

        #[cfg(not(target_os = "macos"))]
        let menu_sub = Subscription::none();

        // Window close events (for cleaning up dialog windows)
        let window_sub = window::close_requests().map(Message::DialogWindowClosed);

        // Toast auto-dismiss timer (5 seconds)
        let toast_sub = if self.state.toast.is_some() {
            time::every(Duration::from_secs(5))
                .map(|_| Message::Toast(crate::message::ToastMessage::Dismiss))
        } else {
            Subscription::none()
        };

        // Auto-save timer (polls every 500ms to check if auto-save should trigger)
        // The actual save only happens if the dirty tracker indicates it should
        let auto_save_sub = if self.state.auto_save_config.enabled && self.state.study.is_some() {
            time::every(Duration::from_millis(500)).map(|_| Message::AutoSaveTick)
        } else {
            Subscription::none()
        };

        Subscription::batch([
            keyboard_sub,
            system_theme_sub,
            menu_sub,
            window_sub,
            toast_sub,
            auto_save_sub,
        ])
    }
}

/// Checks for a post-update status file and returns a toast if the update was successful.
///
/// This is called on app startup to show a notification when the app has been updated.
/// The status file is written by the update helper and deleted after reading.
fn check_update_status() -> Option<crate::component::feedback::toast::ToastState> {
    use directories::ProjectDirs;

    // Get the status file path using the same location as the helper
    // On macOS: ~/Library/Application Support/Trial Submission Studio/update_status.json
    let proj_dirs = ProjectDirs::from("", "", "Trial Submission Studio")?;
    let status_path = proj_dirs.data_dir().join("update_status.json");

    if !status_path.exists() {
        return None;
    }

    // Read and parse the status file (helper writes JSON)
    let content = std::fs::read_to_string(&status_path).ok()?;
    let status: UpdateStatusJson = serde_json::from_str(&content).ok()?;

    // Delete the status file after reading
    let _ = std::fs::remove_file(&status_path);

    if status.success {
        tracing::info!(
            "App was updated from {} to {}",
            status.previous_version,
            status.version
        );
        Some(crate::component::feedback::toast::ToastState::success(
            format!("Updated to v{}", status.version),
        ))
    } else {
        tracing::warn!("Update to {} failed: {:?}", status.version, status.error);
        None
    }
}

/// JSON structure for the update status file (matches tss-updater-helper's UpdateStatus).
#[derive(serde::Deserialize)]
#[allow(dead_code)] // Fields present in JSON but not all read in code
struct UpdateStatusJson {
    success: bool,
    version: String,
    previous_version: String,
    timestamp: String,
    error: Option<String>,
    log_file: String,
}
