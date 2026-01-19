//! Menu action message handlers.
//!
//! Handles:
//! - File menu actions (Open, Close, Settings, Exit)
//! - Edit menu actions (Undo, Redo, Cut, Copy, Paste)
//! - Help menu actions (Documentation, About, etc.)

use iced::window;
use iced::{Size, Task};

use crate::app::App;
use crate::menu::MenuAction;
use crate::message::{
    DialogMessage, HomeMessage, MenuMessage, Message, SettingsCategory, UpdateMessage,
};
use crate::view::dialog::third_party::ThirdPartyState;
use crate::view::dialog::update::UpdateState;

impl App {
    /// Handle the unified MenuAction.
    ///
    /// This is the new handler for all menu actions from both native (macOS)
    /// and desktop (Windows/Linux) menus.
    pub fn handle_menu_action(&mut self, action: MenuAction) -> Task<Message> {
        // Close in-app menu dropdown when any menu action is performed (desktop only)
        #[cfg(not(target_os = "macos"))]
        self.state.menu_dropdown.close();

        match action {
            // File menu
            MenuAction::OpenStudy => Task::done(Message::Home(HomeMessage::OpenStudyClicked)),

            #[cfg(target_os = "macos")]
            MenuAction::OpenRecentStudy(uuid) => {
                // Find the study path by UUID
                if let Some(study) = self
                    .state
                    .settings
                    .general
                    .recent_studies
                    .iter()
                    .find(|s| s.id == uuid)
                {
                    let path = study.path.clone();
                    Task::done(Message::Home(HomeMessage::RecentStudyClicked(path)))
                } else {
                    tracing::warn!("Recent study with UUID {} not found", uuid);
                    Task::none()
                }
            }

            MenuAction::CloseStudy => {
                if self.state.has_study() {
                    Task::done(Message::Home(HomeMessage::CloseStudyClicked))
                } else {
                    Task::none()
                }
            }

            MenuAction::ClearRecentStudies => {
                self.state.settings.general.clear_all_recent();
                let _ = self.state.settings.save();

                // Update native menu's recent studies submenu
                #[cfg(target_os = "macos")]
                crate::menu::update_recent_studies_menu(&[]);
                Task::none()
            }

            MenuAction::Settings => {
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
                self.state.dialog_windows.settings = Some((id, SettingsCategory::default()));
                task.map(|_| Message::Noop)
            }

            MenuAction::Quit => {
                std::process::exit(0);
            }

            // Edit menu (stubs - Iced text widgets handle these natively)
            MenuAction::Undo
            | MenuAction::Redo
            | MenuAction::Cut
            | MenuAction::Copy
            | MenuAction::Paste
            | MenuAction::SelectAll => Task::none(),

            // Help menu
            MenuAction::Documentation => {
                let _ = open::that("https://docs.trialsubmissionstudio.com");
                Task::none()
            }

            MenuAction::ReleaseNotes => {
                let _ =
                    open::that("https://github.com/rubentalstra/trial-submission-studio/releases");
                Task::none()
            }

            MenuAction::ViewOnGitHub => {
                let _ = open::that("https://github.com/rubentalstra/trial-submission-studio");
                Task::none()
            }

            MenuAction::ReportIssue => {
                let _ = open::that(
                    "https://github.com/rubentalstra/trial-submission-studio/issues/new",
                );
                Task::none()
            }

            MenuAction::ViewLicense => {
                let _ = open::that(
                    "https://github.com/rubentalstra/trial-submission-studio/blob/main/LICENSE",
                );
                Task::none()
            }

            MenuAction::ThirdPartyLicenses => {
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
                self.state.dialog_windows.third_party = Some((id, ThirdPartyState::new()));
                task.map(|_| Message::Noop)
            }

            MenuAction::CheckUpdates => {
                // Don't open if already open
                if self.state.dialog_windows.update.is_some() {
                    return Task::none();
                }
                // Open update dialog in a new window
                let settings = window::Settings {
                    size: Size::new(600.0, 420.0),
                    resizable: false,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, open_task) = window::open(settings);
                self.state.dialog_windows.update = Some((id, UpdateState::Checking));

                // Start the update check task
                let update_settings = self.state.settings.updates.clone();
                let check_task = Task::perform(
                    async move {
                        tss_updater::check_for_update(&update_settings)
                            .await
                            .map_err(|e| e.user_message().to_string())
                    },
                    |result| {
                        Message::Dialog(DialogMessage::Update(UpdateMessage::CheckResult(result)))
                    },
                );

                Task::batch([open_task.map(|_| Message::Noop), check_task])
            }

            MenuAction::About => {
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

            // Desktop-only: Toggle dropdown menu
            #[cfg(not(target_os = "macos"))]
            MenuAction::ToggleDropdown(id) => {
                self.state.menu_dropdown.toggle(id);
                Task::none()
            }
        }
    }

    /// Handle legacy menu messages (for backward compatibility during transition).
    pub fn handle_menu_message(&mut self, msg: MenuMessage) -> Task<Message> {
        // Close in-app menu dropdown when any menu action is performed
        #[cfg(not(target_os = "macos"))]
        self.state.menu_dropdown.close();

        match msg {
            // File menu
            MenuMessage::OpenStudy => Task::done(Message::Home(HomeMessage::OpenStudyClicked)),
            MenuMessage::OpenRecentStudy(path) => {
                Task::done(Message::Home(HomeMessage::RecentStudyClicked(path)))
            }
            MenuMessage::CloseStudy => {
                if self.state.has_study() {
                    Task::done(Message::Home(HomeMessage::CloseStudyClicked))
                } else {
                    Task::none()
                }
            }
            MenuMessage::ClearRecentStudies => {
                self.state.settings.general.clear_all_recent();
                let _ = self.state.settings.save();

                // Update native menu's recent studies submenu
                #[cfg(target_os = "macos")]
                crate::menu::update_recent_studies_menu(&[]);
                Task::none()
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
                self.state.dialog_windows.settings = Some((id, SettingsCategory::default()));
                task.map(|_| Message::Noop)
            }
            MenuMessage::Quit => {
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
                let settings = window::Settings {
                    size: Size::new(700.0, 550.0),
                    resizable: true,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, task) = window::open(settings);
                self.state.dialog_windows.third_party = Some((id, ThirdPartyState::new()));
                task.map(|_| Message::Noop)
            }
            MenuMessage::CheckUpdates => {
                // Don't open if already open
                if self.state.dialog_windows.update.is_some() {
                    return Task::none();
                }
                let settings = window::Settings {
                    size: Size::new(600.0, 420.0),
                    resizable: false,
                    decorations: true,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..Default::default()
                };
                let (id, open_task) = window::open(settings);
                self.state.dialog_windows.update = Some((id, UpdateState::Checking));

                let update_settings = self.state.settings.updates.clone();
                let check_task = Task::perform(
                    async move {
                        tss_updater::check_for_update(&update_settings)
                            .await
                            .map_err(|e| e.user_message().to_string())
                    },
                    |result| {
                        Message::Dialog(DialogMessage::Update(UpdateMessage::CheckResult(result)))
                    },
                );

                Task::batch([open_task.map(|_| Message::Noop), check_task])
            }
            MenuMessage::About => {
                // Don't open if already open
                if self.state.dialog_windows.about.is_some() {
                    return Task::none();
                }
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

            // Edit menu (stubs)
            MenuMessage::Undo
            | MenuMessage::Redo
            | MenuMessage::Cut
            | MenuMessage::Copy
            | MenuMessage::Paste
            | MenuMessage::SelectAll => Task::none(),
        }
    }
}
