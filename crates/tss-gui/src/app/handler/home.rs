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
#[cfg(target_os = "macos")]
use crate::app::util::read_csv_files_sync;
use crate::app::util::{StudyLoadInput, load_study_async};
use crate::message::{HomeMessage, Message};
use crate::state::{EditorTab, ViewState};

impl App {
    /// Handle home view messages.
    pub fn handle_home_message(&mut self, msg: HomeMessage) -> Task<Message> {
        match msg {
            HomeMessage::OpenStudyClicked => {
                // On macOS, use synchronous dialog to avoid security-scoped access issues
                // with hardened runtime. The async dialog can lose access when the
                // FileHandle is dropped across thread boundaries.
                #[cfg(target_os = "macos")]
                {
                    let path = rfd::FileDialog::new()
                        .set_title("Select Study Folder")
                        .pick_folder();

                    if let Some(p) = path {
                        return self.load_study(p);
                    }
                    Task::none()
                }

                #[cfg(not(target_os = "macos"))]
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

        // On macOS, read CSV files synchronously on the main thread to maintain
        // security-scoped file access from the file dialog. The hardened runtime
        // restricts file access when crossing thread boundaries.
        #[cfg(target_os = "macos")]
        {
            match read_csv_files_sync(&path, header_rows) {
                Ok(csv_files) => {
                    let input = StudyLoadInput::Preloaded {
                        folder: path,
                        csv_files,
                    };
                    Task::perform(
                        async move { load_study_async(input, header_rows, confidence_threshold).await },
                        Message::StudyLoaded,
                    )
                }
                Err(e) => {
                    self.state.is_loading = false;
                    Task::done(Message::StudyLoaded(Err(e)))
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let input = StudyLoadInput::Path(path);
            Task::perform(
                async move { load_study_async(input, header_rows, confidence_threshold).await },
                Message::StudyLoaded,
            )
        }
    }
}
