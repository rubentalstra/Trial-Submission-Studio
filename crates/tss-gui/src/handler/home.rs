//! Home view message handler.
//!
//! Handles:
//! - Study loading triggers
//! - Recent project selection
//! - Project closing
//! - Navigation to domain editor

use std::path::PathBuf;

use iced::window;
use iced::{Size, Task};

use super::MessageHandler;
use crate::error::GuiError;
use crate::message::{HomeMessage, Message};
use crate::state::{
    AppState, EditorTab, SourceAssignmentUiState, TargetDomainEntry, ViewState, WorkflowMode,
};

/// Handler for home view messages.
///
/// This handler manages:
/// - Project operations
/// - Recent project list interactions
/// - Project closing flow
/// - Navigation to other views
pub struct HomeHandler;

impl MessageHandler<HomeMessage> for HomeHandler {
    fn handle(&self, state: &mut AppState, msg: HomeMessage) -> Task<Message> {
        match msg {
            HomeMessage::StudyFolderSelected(path) => load_study(state, path),

            HomeMessage::CloseProjectClicked => handle_close_project_clicked(state),

            HomeMessage::CloseProjectConfirmed => handle_close_project_confirmed(state),

            HomeMessage::CloseProjectCancelled => handle_close_project_cancelled(state),

            HomeMessage::DomainClicked(domain) => {
                let rows = state.settings.display.preview_rows_per_page;
                state.view = ViewState::domain_editor_with_rows(domain, EditorTab::Mapping, rows);
                Task::none()
            }

            HomeMessage::GeneratedDomainClicked(domain_type) => {
                state.view = ViewState::generated_domain_builder_for(domain_type);
                Task::none()
            }

            HomeMessage::GoToExportClicked => {
                state.view = ViewState::export();
                Task::none()
            }

            // Recent projects
            HomeMessage::RecentProjectClicked(path) => {
                // Open the project file
                Task::done(Message::OpenProjectSelected(Some(path)))
            }

            HomeMessage::RemoveFromRecentProjects(path) => {
                state.settings.general.remove_recent_project(&path);
                let _ = state.settings.save();
                Task::none()
            }

            HomeMessage::ClearAllRecentProjects => {
                state.settings.general.clear_all_recent_projects();
                let _ = state.settings.save();
                Task::none()
            }

            HomeMessage::PruneStaleProjects => {
                state.settings.general.prune_stale_projects();
                let _ = state.settings.save();
                Task::none()
            }
        }
    }
}

// =============================================================================
// HANDLER FUNCTIONS
// =============================================================================

/// Handle close project button click - opens confirmation dialog.
fn handle_close_project_clicked(state: &mut AppState) -> Task<Message> {
    // Don't open if already open
    if state.dialog_windows.close_project_confirm.is_some() {
        return Task::none();
    }

    // Open close project confirmation dialog in a new window
    let settings = window::Settings {
        size: Size::new(350.0, 250.0),
        resizable: false,
        decorations: true,
        level: window::Level::AlwaysOnTop,
        exit_on_close_request: false,
        ..Default::default()
    };
    let (id, task) = window::open(settings);
    state.dialog_windows.close_project_confirm = Some(id);
    task.map(|_| Message::Noop)
}

/// Handle close project confirmation - actually closes the project.
fn handle_close_project_confirmed(state: &mut AppState) -> Task<Message> {
    // Close the confirmation dialog window if open
    let close_task = if let Some(id) = state.dialog_windows.close_project_confirm {
        state.dialog_windows.close_project_confirm = None;
        window::close(id)
    } else {
        Task::none()
    };

    // Close the project
    state.study = None;
    state.project_path = None;
    state.dirty_tracker = tss_persistence::DirtyTracker::new();
    state.view = ViewState::home();
    close_task
}

/// Handle close project cancellation - just closes the dialog.
fn handle_close_project_cancelled(state: &mut AppState) -> Task<Message> {
    if let Some(id) = state.dialog_windows.close_project_confirm {
        state.dialog_windows.close_project_confirm = None;
        return window::close(id);
    }
    Task::none()
}

// =============================================================================
// STUDY LOADING
// =============================================================================

/// Load a study from a folder path.
///
/// This navigates to the source assignment screen where users manually
/// map CSV files to CDISC domains.
pub fn load_study(state: &mut AppState, path: PathBuf) -> Task<Message> {
    // Check for macOS quarantine (#104)
    if let Some(quarantine_msg) = crate::util::check_macos_quarantine(&path) {
        state.error = Some(GuiError::Operation {
            operation: "Load Study Folder".to_string(),
            reason: quarantine_msg,
        });
        return Task::none();
    }

    // Get workflow mode
    let workflow_mode = state.view.workflow_mode();

    // Navigate to source assignment screen
    match navigate_to_source_assignment(state, path, workflow_mode) {
        Ok(()) => Task::none(),
        Err(e) => {
            state.error = Some(GuiError::study_load(e));
            Task::none()
        }
    }
}

/// Navigate to the source assignment screen.
///
/// Lists CSV files in the folder and loads target domains from standards.
fn navigate_to_source_assignment(
    state: &mut AppState,
    folder: PathBuf,
    workflow_mode: WorkflowMode,
) -> Result<(), String> {
    // List CSV files in the folder
    let csv_files = list_csv_files(&folder)?;

    if csv_files.is_empty() {
        return Err("No CSV files found in the selected folder".to_string());
    }

    // Load target domains from standards
    let target_domains = load_target_domains(workflow_mode)?;

    // Create assignment UI state (handles source file entries and domain grouping)
    let assignment_ui = SourceAssignmentUiState::new(folder, csv_files, target_domains);

    // Navigate to source assignment view
    state.view = ViewState::source_assignment(workflow_mode, assignment_ui);

    Ok(())
}

/// List CSV files in a folder.
fn list_csv_files(folder: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(folder).map_err(|e| format!("Failed to read folder: {}", e))?;

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
fn load_target_domains(workflow_mode: WorkflowMode) -> Result<Vec<TargetDomainEntry>, String> {
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
