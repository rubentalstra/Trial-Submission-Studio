//! Project persistence handler.
//!
//! Handles saving and loading .tss project files:
//! - New project creation
//! - Opening existing projects
//! - Saving projects (manual and auto-save)
//! - Source file change detection

use std::path::PathBuf;

use iced::Task;

use crate::message::Message;
use crate::state::AppState;
use tss_persistence::{
    DomainSnapshot, MappingEntry, MappingSnapshot, ProjectFile, SourceAssignment, StudyMetadata,
    SuppActionSnapshot, SuppColumnSnapshot, SuppOriginSnapshot, WorkflowTypeSnapshot,
    load_project_async, save_project_async,
};

// =============================================================================
// NEW PROJECT
// =============================================================================

/// Handle creating a new project from a folder of CSV files.
///
/// Opens a folder picker dialog, then navigates to the source assignment
/// screen where users can map CSV files to CDISC domains.
pub fn handle_new_project(state: &mut AppState) -> Task<Message> {
    // Check if there are unsaved changes
    if state.dirty_tracker.is_dirty() && state.study.is_some() {
        // TODO: Show confirmation dialog before discarding changes
        // For now, just proceed
    }

    // Reset project state
    state.study = None;
    state.project_path = None;
    state.dirty_tracker = tss_persistence::DirtyTracker::new();

    // Open folder dialog
    #[cfg(target_os = "macos")]
    {
        let path = rfd::FileDialog::new()
            .set_title("Select Study Folder")
            .pick_folder();

        if let Some(p) = path {
            return Task::done(Message::Home(
                crate::message::HomeMessage::StudyFolderSelected(p),
            ));
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

// =============================================================================
// OPEN PROJECT
// =============================================================================

/// Handle opening a project file.
pub fn handle_open_project(_state: &mut AppState) -> Task<Message> {
    // On macOS, use synchronous dialog to avoid security-scoped access issues
    #[cfg(target_os = "macos")]
    {
        let path = rfd::FileDialog::new()
            .set_title("Open Project")
            .add_filter("TSS Project", &["tss"])
            .pick_file();

        Task::done(Message::OpenProjectSelected(path))
    }

    #[cfg(not(target_os = "macos"))]
    Task::perform(
        async {
            rfd::AsyncFileDialog::new()
                .set_title("Open Project")
                .add_filter("TSS Project", &["tss"])
                .pick_file()
                .await
                .map(|f| f.path().to_path_buf())
        },
        Message::OpenProjectSelected,
    )
}

/// Handle project file selection from dialog.
pub fn handle_open_project_selected(state: &mut AppState, path: Option<PathBuf>) -> Task<Message> {
    let Some(path) = path else {
        return Task::none();
    };

    state.is_loading = true;

    Task::perform(
        async move {
            match load_project_async(path).await {
                Ok(project) => Ok(project),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::ProjectLoaded,
    )
}

/// Handle a loaded project file.
///
/// This is complex because we need to:
/// 1. Reload source CSV files from their original paths
/// 2. Recreate MappingState with fresh suggestions
/// 3. Apply saved decisions (accepted, not_collected, omitted, auto_generated)
/// 4. Restore SUPP configurations
pub fn handle_project_loaded(
    state: &mut AppState,
    result: Result<tss_persistence::ProjectFile, String>,
) -> Task<Message> {
    state.is_loading = false;

    match result {
        Ok(project) => {
            tracing::info!(
                "Project loaded: {} from {:?}",
                project.study.study_id,
                project.study.study_folder
            );

            // For now, we'll just store the project metadata
            // Full restoration requires reloading CSV files which we'll do via the existing study loading mechanism
            // The project path will be stored for future saves

            // Store the project path for future saves
            // Note: We don't have the path here - we'd need to pass it through the message
            // For now, we'll trigger a study folder load which will rebuild everything

            let study_folder = PathBuf::from(&project.study.study_folder);

            // Trigger study loading from the folder
            // TODO: After loading, apply the saved mappings from the project file
            // This requires extending the StudyLoaded message to include the project data
            crate::handler::home::load_study(state, study_folder)
        }
        Err(e) => {
            tracing::error!("Failed to load project: {}", e);
            state.error = Some(crate::error::GuiError::operation("Load project", e));
            Task::none()
        }
    }
}

// =============================================================================
// SAVE PROJECT
// =============================================================================

/// Handle saving the current project.
pub fn handle_save_project(state: &mut AppState) -> Task<Message> {
    match &state.project_path {
        Some(path) => {
            // Save to existing path
            do_save_project(state, path.clone())
        }
        None => {
            // No existing path - trigger Save As
            handle_save_project_as(state)
        }
    }
}

/// Handle saving the project to a new path.
pub fn handle_save_project_as(_state: &mut AppState) -> Task<Message> {
    // On macOS, use synchronous dialog to avoid security-scoped access issues
    #[cfg(target_os = "macos")]
    {
        let path = rfd::FileDialog::new()
            .set_title("Save Project As")
            .add_filter("TSS Project", &["tss"])
            .set_file_name("project.tss")
            .save_file();

        Task::done(Message::SavePathSelected(path))
    }

    #[cfg(not(target_os = "macos"))]
    Task::perform(
        async {
            rfd::AsyncFileDialog::new()
                .set_title("Save Project As")
                .add_filter("TSS Project", &["tss"])
                .set_file_name("project.tss")
                .save_file()
                .await
                .map(|f| f.path().to_path_buf())
        },
        Message::SavePathSelected,
    )
}

/// Handle save path selection from dialog.
pub fn handle_save_path_selected(state: &mut AppState, path: Option<PathBuf>) -> Task<Message> {
    let Some(mut path) = path else {
        return Task::none();
    };

    // Ensure .tss extension
    if path.extension().is_none() || path.extension() != Some(std::ffi::OsStr::new("tss")) {
        path.set_extension("tss");
    }

    do_save_project(state, path)
}

/// Actually perform the save operation.
fn do_save_project(state: &mut AppState, path: PathBuf) -> Task<Message> {
    // Create project file from current state
    let Some(study) = &state.study else {
        return Task::none();
    };

    // Build the project file
    let project = create_project_file_from_state(study, state);

    // Mark that we're saving
    state.dirty_tracker.start_save();

    // Store the path for future saves
    let save_path = path.clone();
    state.project_path = Some(path);

    Task::perform(
        async move {
            match save_project_async(project, save_path.clone()).await {
                Ok(()) => Ok(save_path),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::ProjectSaved,
    )
}

/// Handle save completion.
pub fn handle_project_saved(
    state: &mut AppState,
    result: Result<PathBuf, String>,
) -> Task<Message> {
    match result {
        Ok(path) => {
            tracing::info!("Project saved to {:?}", path);
            state.dirty_tracker.save_complete();
            state.project_path = Some(path.clone());

            // Add to recent projects
            if let Some(study) = &state.study {
                let workflow_type = match state.view.workflow_mode() {
                    crate::state::WorkflowMode::Sdtm => crate::state::WorkflowType::Sdtm,
                    crate::state::WorkflowMode::Adam => crate::state::WorkflowType::Adam,
                    crate::state::WorkflowMode::Send => crate::state::WorkflowType::Send,
                };
                let recent = crate::state::RecentProject::new(
                    path,
                    study.study_id.clone(),
                    workflow_type,
                    study.domain_count(),
                );
                state.settings.general.add_recent_project(recent);
                let _ = state.settings.save();

                // Update native menu on macOS
                #[cfg(target_os = "macos")]
                {
                    let projects: Vec<_> = state
                        .settings
                        .general
                        .recent_projects_sorted()
                        .iter()
                        .map(|p| crate::menu::RecentProjectInfo::new(p.id, p.display_name.clone()))
                        .collect();
                    crate::menu::update_recent_projects_menu(&projects);
                }
            }

            Task::none()
        }
        Err(e) => {
            tracing::error!("Failed to save project: {}", e);
            state.dirty_tracker.save_failed();
            state.error = Some(crate::error::GuiError::operation("Save project", e));
            Task::none()
        }
    }
}

// =============================================================================
// AUTO-SAVE
// =============================================================================

/// Handle auto-save tick.
pub fn handle_auto_save_tick(state: &mut AppState) -> Task<Message> {
    // Check if we should auto-save
    if !state
        .dirty_tracker
        .should_auto_save(&state.auto_save_config)
    {
        return Task::none();
    }

    // Check if we have a project path
    let Some(path) = state.project_path.clone() else {
        // No project path - can't auto-save
        // TODO: Could prompt user to save first time
        return Task::none();
    };

    tracing::debug!("Auto-saving project to {:?}", path);
    do_save_project(state, path)
}

// =============================================================================
// HELPERS
// =============================================================================

/// Create a ProjectFile from the current application state.
fn create_project_file_from_state(study: &crate::state::Study, state: &AppState) -> ProjectFile {
    use crate::state::WorkflowMode;

    // Determine workflow type from current view
    let workflow_type = match state.view.workflow_mode() {
        WorkflowMode::Sdtm => WorkflowTypeSnapshot::Sdtm,
        WorkflowMode::Adam => WorkflowTypeSnapshot::Adam,
        WorkflowMode::Send => WorkflowTypeSnapshot::Send,
    };

    // Create study metadata
    let study_meta = StudyMetadata::new(
        &study.study_id,
        study.study_folder.to_string_lossy(),
        workflow_type,
    );

    let mut project = ProjectFile::new(study_meta);

    // Add source assignments and domain snapshots
    for code in study.domain_codes() {
        if let Some(domain_state) = study.domain(code) {
            // Get file size for source assignment
            let file_size = std::fs::metadata(&domain_state.source.file_path)
                .map(|m| m.len())
                .unwrap_or(0);

            // Add source assignment
            let source_path = domain_state.source.file_path.to_string_lossy().to_string();
            // TODO: Compute actual content hash using tss_persistence::compute_file_hash
            let content_hash = String::new();
            let assignment = SourceAssignment::new(&source_path, code, content_hash, file_size);
            project.source_assignments.push(assignment);

            // Create domain snapshot
            let mut snapshot = DomainSnapshot::new(code);
            snapshot.label = domain_state.source.label.clone();

            // Create mapping snapshot
            let mapping = &domain_state.mapping;
            let mapping_snapshot = MappingSnapshot {
                study_id: mapping.study_id().to_string(),
                accepted: mapping
                    .all_accepted()
                    .iter()
                    .map(|(var, (col, conf))| (var.clone(), MappingEntry::new(col.clone(), *conf)))
                    .collect(),
                not_collected: mapping.all_not_collected().clone(),
                omitted: mapping.all_omitted().clone(),
                auto_generated: mapping.all_auto_generated().clone(),
            };
            snapshot.mapping = mapping_snapshot;

            // Add SUPP config
            for (col, config) in &domain_state.supp_config {
                let supp_snapshot = SuppColumnSnapshot {
                    column: col.clone(),
                    qnam: config.qnam.clone(),
                    qlabel: config.qlabel.clone(),
                    qorig: match config.qorig {
                        crate::state::SuppOrigin::Crf => SuppOriginSnapshot::Crf,
                        crate::state::SuppOrigin::Derived => SuppOriginSnapshot::Derived,
                        crate::state::SuppOrigin::Assigned => SuppOriginSnapshot::Assigned,
                    },
                    qeval: config.qeval.clone(),
                    action: match config.action {
                        crate::state::SuppAction::Pending => SuppActionSnapshot::Pending,
                        crate::state::SuppAction::Include => SuppActionSnapshot::Include,
                        crate::state::SuppAction::Skip => SuppActionSnapshot::Skip,
                    },
                };
                snapshot.supp_config.insert(col.clone(), supp_snapshot);
            }

            project.domains.insert(code.to_string(), snapshot);
        }
    }

    project
}

/// Mark the project as dirty (has unsaved changes).
///
/// Call this from handlers when state changes that should be persisted.
pub fn mark_dirty(state: &mut AppState) {
    state.dirty_tracker.mark_dirty();
}
