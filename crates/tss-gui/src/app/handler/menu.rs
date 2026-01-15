//! Menu action message handlers.
//!
//! Handles:
//! - File menu actions (Open, Close, Settings, Exit)
//! - Edit menu actions (Undo, Redo, Cut, Copy, Paste)
//! - Help menu actions (Documentation, About, etc.)

use iced::window;
use iced::{Size, Task};

use crate::app::App;
use crate::message::{HomeMessage, MenuMessage, Message, SettingsCategory};
use crate::view::dialog::third_party::ThirdPartyState;
use crate::view::dialog::update::UpdateState;

impl App {
    /// Handle menu messages.
    pub fn handle_menu_message(&mut self, msg: MenuMessage) -> Task<Message> {
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
                self.state.dialog_windows.settings = Some((id, SettingsCategory::default()));
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
                self.state.dialog_windows.third_party = Some((id, ThirdPartyState::new()));
                task.map(|_| Message::Noop)
            }
            MenuMessage::CheckUpdates => {
                // Don't open if already open
                if self.state.dialog_windows.update.is_some() {
                    return Task::none();
                }
                // Open update dialog in a new window
                // Size accommodates: icon, version info, changelog, and action buttons
                let settings = window::Settings {
                    size: Size::new(550.0, 500.0),
                    resizable: true,
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
