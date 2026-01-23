//! Menu action message handler.
//!
//! Handles:
//! - File menu actions (New, Open, Save, Close, Settings, Exit)
//! - Edit menu actions (Undo, Redo, Cut, Copy, Paste)
//! - Help menu actions (Documentation, About, etc.)

use iced::window;
use iced::{Size, Task};

use crate::handler::MessageHandler;
use crate::menu::MenuAction;
use crate::message::{DialogMessage, HomeMessage, Message, SettingsCategory, UpdateMessage};
use crate::state::AppState;
use crate::view::dialog::third_party::ThirdPartyState;
use crate::view::dialog::update::UpdateState;

/// Handler for menu action messages.
pub struct MenuActionHandler;

impl MessageHandler<MenuAction> for MenuActionHandler {
    fn handle(&self, state: &mut AppState, action: MenuAction) -> Task<Message> {
        // Close in-app menu dropdown when any menu action is performed (desktop only)
        #[cfg(not(target_os = "macos"))]
        state.menu_dropdown.close();

        match action {
            // File menu - Project operations
            MenuAction::NewProject => Task::done(Message::NewProject),

            MenuAction::OpenProject => Task::done(Message::OpenProject),

            MenuAction::SaveProject => {
                if state.has_study() {
                    Task::done(Message::SaveProject)
                } else {
                    Task::none()
                }
            }

            MenuAction::SaveProjectAs => {
                if state.has_study() {
                    Task::done(Message::SaveProjectAs)
                } else {
                    Task::none()
                }
            }

            #[cfg(target_os = "macos")]
            MenuAction::OpenRecentProject(uuid) => {
                // Find the project path by UUID
                if let Some(project) = state
                    .settings
                    .general
                    .recent_projects
                    .iter()
                    .find(|p| p.id == uuid)
                {
                    let path = project.path.clone();
                    Task::done(Message::OpenProjectSelected(Some(path)))
                } else {
                    tracing::warn!("Recent project with UUID {} not found", uuid);
                    Task::none()
                }
            }

            MenuAction::CloseProject => {
                if state.has_study() {
                    Task::done(Message::Home(HomeMessage::CloseProjectClicked))
                } else {
                    Task::none()
                }
            }

            MenuAction::ClearRecentProjects => {
                state.settings.general.clear_all_recent_projects();
                let _ = state.settings.save();

                // Update native menu's recent projects submenu
                #[cfg(target_os = "macos")]
                crate::menu::update_recent_projects_menu(&[]);
                Task::none()
            }

            MenuAction::Settings => {
                // Don't open if already open
                if state.dialog_windows.settings.is_some() {
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
                state.dialog_windows.settings = Some((id, SettingsCategory::default()));
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
                if state.dialog_windows.third_party.is_some() {
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
                state.dialog_windows.third_party = Some((id, ThirdPartyState::new()));
                task.map(|_| Message::Noop)
            }

            MenuAction::CheckUpdates => {
                // Don't open if already open
                if state.dialog_windows.update.is_some() {
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
                state.dialog_windows.update = Some((id, UpdateState::Checking));

                // Start the update check task
                let update_settings = state.settings.updates.clone();
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
                if state.dialog_windows.about.is_some() {
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
                state.dialog_windows.about = Some(id);
                task.map(|_| Message::Noop)
            }

            // Desktop-only: Toggle dropdown menu
            #[cfg(not(target_os = "macos"))]
            MenuAction::ToggleDropdown(id) => {
                state.menu_dropdown.toggle(id);
                Task::none()
            }
        }
    }
}
