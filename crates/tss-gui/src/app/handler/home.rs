//! Home view message handlers.
//!
//! Handles:
//! - Study loading triggers
//! - Recent study selection
//! - Workflow mode changes

use std::path::PathBuf;

use iced::window;
use iced::{Size, Task};

use crate::app::App;
use crate::app::util::load_study_async;
use crate::message::{HomeMessage, Message};
use crate::state::{EditorTab, ViewState};

impl App {
    /// Handle home view messages.
    pub fn handle_home_message(&mut self, msg: HomeMessage) -> Task<Message> {
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
    pub fn load_study(&mut self, path: PathBuf) -> Task<Message> {
        self.state.is_loading = true;
        let header_rows = self.state.settings.general.header_rows;
        let confidence_threshold = self.state.settings.general.mapping_confidence_threshold;

        Task::perform(
            async move { load_study_async(path, header_rows, confidence_threshold).await },
            Message::StudyLoaded,
        )
    }
}
