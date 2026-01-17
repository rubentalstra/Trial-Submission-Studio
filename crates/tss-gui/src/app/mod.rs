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
mod update;
pub mod util;

// Re-export utility functions for internal use
use util::load_app_icon;

use iced::keyboard;
use iced::widget::container;
use iced::window;
use iced::{Element, Size, Subscription, Task, Theme};

use crate::message::{Message, SettingsCategory};
use crate::state::{AppState, DialogType, Settings, ViewState};
use crate::theme::clinical_light;
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
                        self.state.dialog_windows.third_party = Some((id, ThirdPartyState::new()));
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
                self.set_update_ready_to_install(info, data, verified);
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

            // =================================================================
            // Toast notifications
            // =================================================================
            Message::Toast(toast_msg) => self.handle_toast_message(toast_msg),

            Message::Noop => Task::none(),
        }
    }

    /// Handle toast notification messages.
    fn handle_toast_message(&mut self, msg: crate::message::ToastMessage) -> Task<Message> {
        use crate::component::toast::{ToastActionType, ToastMessage};

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
                            return self
                                .handle_menu_message(crate::message::MenuMessage::CheckUpdates);
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

        // If there's a toast, wrap content with an overlay
        if let Some(toast) = &self.state.toast {
            use crate::component::toast::view_toast;
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

        // Toast auto-dismiss timer (5 seconds)
        let toast_sub = if self.state.toast.is_some() {
            time::every(Duration::from_secs(5))
                .map(|_| Message::Toast(crate::message::ToastMessage::Dismiss))
        } else {
            Subscription::none()
        };

        Subscription::batch([keyboard_sub, menu_sub, window_sub, toast_sub])
    }
}

/// Checks for a post-update status file and returns a toast if the update was successful.
///
/// This is called on app startup to show a notification when the app has been updated.
/// The status file is written by the update helper and deleted after reading.
fn check_update_status() -> Option<crate::component::toast::ToastState> {
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
        Some(crate::component::toast::ToastState::update_success(
            &status.version,
        ))
    } else {
        tracing::warn!("Update to {} failed: {:?}", status.version, status.error);
        None
    }
}

/// JSON structure for the update status file (matches tss-updater-helper's UpdateStatus).
#[derive(serde::Deserialize)]
struct UpdateStatusJson {
    success: bool,
    version: String,
    previous_version: String,
    #[allow(dead_code)]
    timestamp: String,
    error: Option<String>,
    #[allow(dead_code)]
    log_file: String,
}
