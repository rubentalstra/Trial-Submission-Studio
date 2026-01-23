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
    compute_file_hash, load_project_async, save_project_async,
};

// =============================================================================
// NEW PROJECT
// =============================================================================

/// Handle creating a new project from a folder of CSV files.
///
/// Opens a folder picker dialog, then navigates to the source assignment
/// screen where users can map CSV files to CDISC domains.
pub fn handle_new_project(state: &mut AppState) -> Task<Message> {
    // Check if there are unsaved changes - show confirmation dialog
    if state.dirty_tracker.is_dirty() && state.study.is_some() {
        return show_unsaved_changes_dialog(state, crate::state::PendingAction::NewProject);
    }

    // No unsaved changes, proceed directly
    do_new_project(state)
}

/// Actually create a new project (after confirmation or when clean).
pub fn do_new_project(state: &mut AppState) -> Task<Message> {
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

    // Clone path for the closure
    let project_path = path.clone();

    Task::perform(
        async move {
            match load_project_async(path).await {
                Ok(project) => Ok((project_path, project)),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::ProjectLoaded,
    )
}

/// Handle a loaded project file.
///
/// When loading a project:
/// 1. Build assignments from saved source_assignments (skip assignment screen)
/// 2. Create Study directly using create_study_from_assignments
/// 3. StudyLoaded handler will apply saved mappings from pending_project_restore
///
/// Note: If source files are missing, we show an error instead of falling back
/// to the assignment screen.
pub fn handle_project_loaded(
    state: &mut AppState,
    result: Result<(PathBuf, tss_persistence::ProjectFile), String>,
) -> Task<Message> {
    state.is_loading = false;

    match result {
        Ok((project_path, project)) => {
            tracing::info!(
                "Project loaded: {} from {:?}",
                project.study.study_id,
                project.study.study_folder
            );

            // Check if there are source assignments to restore
            if project.source_assignments.is_empty() {
                tracing::warn!("Project has no source assignments - cannot restore study");
                state.error = Some(crate::error::GuiError::operation(
                    "Load project",
                    "Project has no source file assignments. Please create a new project.",
                ));
                return Task::none();
            }

            // Build assignments map from saved source_assignments
            let mut assignments: std::collections::BTreeMap<String, PathBuf> =
                std::collections::BTreeMap::new();
            let mut missing_files = Vec::new();

            for assignment in &project.source_assignments {
                let file_path = PathBuf::from(&assignment.file_path);

                // Check if file exists
                if !file_path.exists() {
                    missing_files.push(assignment.file_path.clone());
                    continue;
                }

                assignments.insert(assignment.domain_code.clone(), file_path);
            }

            // If any source files are missing, show error
            if !missing_files.is_empty() {
                tracing::error!("Source files missing: {:?}", missing_files);
                let msg = format!(
                    "Cannot load project - {} source file(s) not found:\n{}",
                    missing_files.len(),
                    missing_files
                        .iter()
                        .take(3) // Show at most 3 files
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                state.error = Some(crate::error::GuiError::operation("Load project", msg));
                return Task::none();
            }

            // Convert workflow type from persistence to GUI type
            let workflow_mode = match project.study.workflow_type {
                WorkflowTypeSnapshot::Sdtm => crate::state::WorkflowMode::Sdtm,
                WorkflowTypeSnapshot::Adam => crate::state::WorkflowMode::Adam,
                WorkflowTypeSnapshot::Send => crate::state::WorkflowMode::Send,
            };

            // Store the project path for future saves
            state.project_path = Some(project_path.clone());

            // Store the project data for restoration after study loading completes
            // The StudyLoaded handler will check for this and apply the saved mappings
            state.pending_project_restore = Some((project_path, project.clone()));

            // Reset dirty tracker since we're loading a saved project
            state.dirty_tracker = tss_persistence::DirtyTracker::new();

            // Get settings for study creation
            let folder = PathBuf::from(&project.study.study_folder);
            let header_rows = state.settings.general.header_rows;
            let confidence_threshold = state.settings.general.mapping_confidence_threshold;

            // Set loading state
            state.is_loading = true;

            // Create study directly from saved assignments (skip assignment screen)
            Task::perform(
                crate::service::study::create_study_from_assignments(
                    folder,
                    assignments,
                    Vec::new(), // No metadata files - they were already incorporated
                    header_rows,
                    confidence_threshold,
                    workflow_mode,
                ),
                Message::StudyLoaded,
            )
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

            // Check if there's a pending action to perform after save
            // (from the unsaved changes dialog "Save" button)
            if let Some(pending_action) = state.pending_action_after_save.take() {
                match pending_action {
                    crate::state::PendingAction::NewProject => do_new_project(state),
                    crate::state::PendingAction::OpenProject(project_path) => {
                        handle_open_project_selected(state, Some(project_path))
                    }
                    crate::state::PendingAction::QuitApp => {
                        if let Some(main_id) = state.main_window_id {
                            iced::window::close(main_id)
                        } else {
                            Task::none()
                        }
                    }
                }
            } else {
                Task::none()
            }
        }
        Err(e) => {
            tracing::error!("Failed to save project: {}", e);
            state.dirty_tracker.save_failed();
            state.error = Some(crate::error::GuiError::operation("Save project", e));
            // Clear pending action on failure
            state.pending_action_after_save = None;
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
        // No project path - can't auto-save silently
        // User needs to do "Save As" first to establish the project file location
        // We don't prompt automatically to avoid interrupting the user's workflow
        // The user will be prompted when trying to close with unsaved changes
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

            // Add source assignment with content hash for change detection
            let source_path = domain_state.source.file_path.to_string_lossy().to_string();
            let content_hash =
                compute_file_hash(&domain_state.source.file_path).unwrap_or_else(|e| {
                    tracing::warn!("Failed to compute hash for {}: {}", source_path, e);
                    String::new()
                });
            let assignment = SourceAssignment::new(&source_path, code, content_hash, file_size);
            project.source_assignments.push(assignment);

            // Create domain snapshot
            let mut snapshot = DomainSnapshot::new(code);
            snapshot.label = domain_state.source.label.clone();

            // Create mapping snapshot
            // Note: We discard confidence scores - they're only meaningful during active mapping
            let mapping = &domain_state.mapping;
            let mapping_snapshot = MappingSnapshot {
                study_id: mapping.study_id().to_string(),
                accepted: mapping
                    .all_accepted()
                    .iter()
                    .map(|(var, (col, _conf))| (var.clone(), MappingEntry::new(col.clone())))
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

// =============================================================================
// UNSAVED CHANGES DIALOG
// =============================================================================

/// Show the unsaved changes confirmation dialog before quitting.
///
/// This is called when the user tries to close the main window
/// with unsaved changes.
pub fn show_unsaved_changes_dialog_for_quit(state: &mut AppState) -> Task<Message> {
    show_unsaved_changes_dialog(state, crate::state::PendingAction::QuitApp)
}

/// Show the unsaved changes confirmation dialog.
fn show_unsaved_changes_dialog(
    state: &mut AppState,
    pending_action: crate::state::PendingAction,
) -> Task<Message> {
    use iced::Size;
    use iced::window;

    // Don't open if already open
    if state.dialog_windows.unsaved_changes.is_some() {
        return Task::none();
    }

    // Open unsaved changes dialog in a new window
    let settings = window::Settings {
        size: Size::new(420.0, 220.0),
        resizable: false,
        decorations: true,
        level: window::Level::AlwaysOnTop,
        exit_on_close_request: false,
        ..Default::default()
    };
    let (id, task) = window::open(settings);
    state.dialog_windows.unsaved_changes = Some((id, pending_action));

    task.map(move |_| Message::DialogWindowOpened(crate::state::DialogType::UnsavedChanges, id))
}

/// Handle "Save" button clicked in unsaved changes dialog.
pub fn handle_unsaved_changes_save(state: &mut AppState) -> Task<Message> {
    // Close the dialog first and get the pending action
    let pending_action = close_unsaved_changes_dialog(state);

    // Store the pending action so ProjectSaved can continue it after save completes
    state.pending_action_after_save = pending_action;

    // Save the project - the pending action will be executed in handle_project_saved
    handle_save_project(state)
}

/// Handle "Don't Save" button clicked in unsaved changes dialog.
pub fn handle_unsaved_changes_discard(state: &mut AppState) -> Task<Message> {
    // Get the pending action and close the dialog
    let pending_action = close_unsaved_changes_dialog(state);

    // Reset dirty state since we're discarding changes
    state.dirty_tracker = tss_persistence::DirtyTracker::new();

    // Perform the pending action
    match pending_action {
        Some(crate::state::PendingAction::NewProject) => do_new_project(state),
        Some(crate::state::PendingAction::OpenProject(path)) => {
            handle_open_project_selected(state, Some(path))
        }
        Some(crate::state::PendingAction::QuitApp) => {
            // Close the main window to quit the application
            if let Some(main_id) = state.main_window_id {
                iced::window::close(main_id)
            } else {
                Task::none()
            }
        }
        None => Task::none(),
    }
}

/// Handle "Cancel" button clicked in unsaved changes dialog.
///
/// This just closes the dialog and returns to the application.
/// The user stays in the app with their project still open.
pub fn handle_unsaved_changes_cancelled(state: &mut AppState) -> Task<Message> {
    // Just close the dialog, do nothing else
    // Note: The cancel button in the view uses Message::CloseWindow(window_id)
    // which handles the window closing directly. This handler is a fallback.
    if let Some((window_id, _)) = state.dialog_windows.unsaved_changes.take() {
        iced::window::close(window_id)
    } else {
        Task::none()
    }
}

/// Close the unsaved changes dialog and return the pending action.
fn close_unsaved_changes_dialog(state: &mut AppState) -> Option<crate::state::PendingAction> {
    state
        .dialog_windows
        .unsaved_changes
        .take()
        .map(|(_, action)| action)
}

// =============================================================================
// SOURCE FILE CHANGE DETECTION
// =============================================================================

/// Check if any source files have changed since the project was saved.
///
/// Returns a list of file paths that have changed (different hash or missing).
pub fn detect_changed_source_files(project: &tss_persistence::ProjectFile) -> Vec<String> {
    let mut changed_files = Vec::new();

    for assignment in &project.source_assignments {
        // Skip if no hash was stored (legacy projects)
        if assignment.content_hash.is_empty() {
            continue;
        }

        let path = std::path::Path::new(&assignment.file_path);

        // Check if file exists
        if !path.exists() {
            changed_files.push(format!("{} (missing)", assignment.file_path));
            continue;
        }

        // Check if hash matches
        match compute_file_hash(path) {
            Ok(current_hash) => {
                if current_hash != assignment.content_hash {
                    changed_files.push(assignment.file_path.clone());
                }
            }
            Err(e) => {
                tracing::warn!("Failed to compute hash for {}: {}", assignment.file_path, e);
                // Consider it changed if we can't verify
                changed_files.push(format!("{} (unreadable)", assignment.file_path));
            }
        }
    }

    changed_files
}

// =============================================================================
// PROJECT RESTORATION
// =============================================================================

/// Restore project mappings from a loaded project file.
///
/// This is called after the study is loaded to apply saved user decisions:
/// - Accepted mappings (column → variable)
/// - Not collected variables with reasons
/// - Omitted variables
/// - Auto-generated variables
/// - SUPP configurations
pub fn restore_project_mappings(state: &mut AppState, project: &tss_persistence::ProjectFile) {
    let Some(study) = &mut state.study else {
        tracing::warn!("Cannot restore project mappings: no study loaded");
        return;
    };

    tracing::info!(
        "Restoring mappings for {} domains from project",
        project.domains.len()
    );

    // Iterate through saved domain snapshots
    for (domain_code, snapshot) in &project.domains {
        if let Some(domain_state) = study.domain_mut(domain_code) {
            // Restore mapping decisions
            let mapping = &mut domain_state.mapping;

            // Apply accepted mappings
            for (var, entry) in &snapshot.mapping.accepted {
                // Use accept_manual to apply saved mappings
                if let Err(e) = mapping.accept_manual(var, &entry.source_column) {
                    tracing::warn!(
                        "Failed to restore mapping {} -> {}: {}",
                        entry.source_column,
                        var,
                        e
                    );
                }
            }

            // Apply not collected
            for (var, reason) in &snapshot.mapping.not_collected {
                if let Err(e) = mapping.mark_not_collected(var, reason) {
                    tracing::warn!("Failed to mark {} as not collected: {}", var, e);
                }
            }

            // Apply omitted
            for var in &snapshot.mapping.omitted {
                if let Err(e) = mapping.mark_omit(var) {
                    tracing::warn!("Failed to mark {} as omitted: {}", var, e);
                }
            }

            // Apply auto-generated
            for var in &snapshot.mapping.auto_generated {
                mapping.mark_auto_generated(var);
            }

            // Restore SUPP configurations
            for (col, supp_snapshot) in &snapshot.supp_config {
                let config = crate::state::SuppColumnConfig {
                    column: col.clone(),
                    qnam: supp_snapshot.qnam.clone(),
                    qlabel: supp_snapshot.qlabel.clone(),
                    qorig: match supp_snapshot.qorig {
                        SuppOriginSnapshot::Crf => crate::state::SuppOrigin::Crf,
                        SuppOriginSnapshot::Derived => crate::state::SuppOrigin::Derived,
                        SuppOriginSnapshot::Assigned => crate::state::SuppOrigin::Assigned,
                    },
                    qeval: supp_snapshot.qeval.clone(),
                    action: match supp_snapshot.action {
                        SuppActionSnapshot::Pending => crate::state::SuppAction::Pending,
                        SuppActionSnapshot::Include => crate::state::SuppAction::Include,
                        SuppActionSnapshot::Skip => crate::state::SuppAction::Skip,
                    },
                };
                domain_state.supp_config.insert(col.clone(), config);
            }

            tracing::debug!(
                "Restored mappings for domain {}: {} accepted, {} not_collected, {} omitted, {} auto_generated, {} supp",
                domain_code,
                snapshot.mapping.accepted.len(),
                snapshot.mapping.not_collected.len(),
                snapshot.mapping.omitted.len(),
                snapshot.mapping.auto_generated.len(),
                snapshot.supp_config.len()
            );
        } else {
            tracing::warn!("Domain {} in project file not found in study", domain_code);
        }
    }

    tracing::info!("Project mappings restored successfully");
}
