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
use crate::message::{HomeMessage, Message};
use crate::state::{
    EditorTab, SourceAssignmentUiState, TargetDomainEntry, ViewState, WorkflowMode,
};

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
                let rows = self.state.settings.display.preview_rows_per_page;
                self.state.view =
                    ViewState::domain_editor_with_rows(domain, EditorTab::Mapping, rows);
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

            HomeMessage::ClearAllRecentStudies => {
                self.state.settings.general.clear_all_recent();
                let _ = self.state.settings.save();
                Task::none()
            }

            HomeMessage::PruneStaleStudies => {
                self.state.settings.general.prune_stale();
                let _ = self.state.settings.save();
                Task::none()
            }
        }
    }

    /// Load a study from a folder path.
    ///
    /// This navigates to the source assignment screen where users manually
    /// map CSV files to CDISC domains.
    pub fn load_study(&mut self, path: PathBuf) -> Task<Message> {
        // Get workflow mode
        let workflow_mode = self.state.view.workflow_mode();

        // Navigate to source assignment screen
        match self.navigate_to_source_assignment(path, workflow_mode) {
            Ok(()) => Task::none(),
            Err(e) => {
                self.state.error = Some(e);
                Task::none()
            }
        }
    }

    /// Navigate to the source assignment screen.
    ///
    /// Lists CSV files in the folder and loads target domains from standards.
    fn navigate_to_source_assignment(
        &mut self,
        folder: PathBuf,
        workflow_mode: WorkflowMode,
    ) -> Result<(), String> {
        // List CSV files in the folder
        let csv_files = self.list_csv_files(&folder)?;

        if csv_files.is_empty() {
            return Err("No CSV files found in the selected folder".to_string());
        }

        // Load target domains from standards
        let target_domains = self.load_target_domains(workflow_mode)?;

        // Create assignment UI state (handles source file entries and domain grouping)
        let assignment_ui = SourceAssignmentUiState::new(folder, csv_files, target_domains);

        // Navigate to source assignment view
        self.state.view = ViewState::source_assignment(workflow_mode, assignment_ui);

        Ok(())
    }

    /// List CSV files in a folder.
    fn list_csv_files(&self, folder: &PathBuf) -> Result<Vec<PathBuf>, String> {
        let entries =
            std::fs::read_dir(folder).map_err(|e| format!("Failed to read folder: {}", e))?;

        let csv_files: Vec<PathBuf> = entries
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .map(|ext| ext.eq_ignore_ascii_case("csv"))
                    .unwrap_or(false)
            })
            .collect();

        Ok(csv_files)
    }

    /// Load target domains from standards based on workflow mode.
    fn load_target_domains(
        &self,
        workflow_mode: WorkflowMode,
    ) -> Result<Vec<TargetDomainEntry>, String> {
        match workflow_mode {
            WorkflowMode::Sdtm => {
                let domains = tss_standards::load_sdtm_ig()
                    .map_err(|e| format!("Failed to load SDTM-IG: {}", e))?;

                let entries: Vec<TargetDomainEntry> = domains
                    .iter()
                    .map(|d| {
                        TargetDomainEntry::new(
                            d.name.clone(),
                            d.label.clone(),
                            d.class.map(|c| c.to_string()),
                        )
                        .with_description(d.structure.clone())
                    })
                    .collect();

                Ok(entries)
            }
            WorkflowMode::Adam | WorkflowMode::Send => Err(format!(
                "{} workflow not yet fully supported",
                workflow_mode.display_name()
            )),
        }
    }
}
