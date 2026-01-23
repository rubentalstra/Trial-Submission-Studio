//! Source assignment view message handler.
//!
//! Handles all interactions for the source-to-domain assignment screen.

use iced::Task;

use super::MessageHandler;
use crate::message::{Message, SourceAssignmentMessage};
use crate::state::{AppState, SourceFileStatus, ViewState};

/// Handler for source assignment view messages.
pub struct SourceAssignmentHandler;

impl MessageHandler<SourceAssignmentMessage> for SourceAssignmentHandler {
    fn handle(&self, state: &mut AppState, msg: SourceAssignmentMessage) -> Task<Message> {
        match msg {
            // =================================================================
            // Drag and Drop
            // =================================================================
            SourceAssignmentMessage::DragStarted { file_index } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view {
                    assignment_ui.dragging_file = Some(file_index);
                    assignment_ui.selected_file = None;
                }
                Task::none()
            }

            SourceAssignmentMessage::DragOverDomain { domain_code } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view {
                    assignment_ui.hover_domain = domain_code;
                }
                Task::none()
            }

            SourceAssignmentMessage::DroppedOnDomain {
                file_index,
                domain_code,
            } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view {
                    if let Some(file) = assignment_ui.source_files.get_mut(file_index) {
                        file.assigned_domain = Some(domain_code);
                        file.status = SourceFileStatus::Unassigned;
                    }
                    assignment_ui.dragging_file = None;
                    assignment_ui.hover_domain = None;
                }
                Task::none()
            }

            SourceAssignmentMessage::DragCancelled => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view {
                    assignment_ui.dragging_file = None;
                    assignment_ui.hover_domain = None;
                }
                Task::none()
            }

            // =================================================================
            // Click-to-Assign
            // =================================================================
            SourceAssignmentMessage::FileClicked { file_index } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view {
                    if assignment_ui.selected_file == Some(file_index) {
                        assignment_ui.selected_file = None;
                    } else {
                        assignment_ui.selected_file = Some(file_index);
                    }
                    assignment_ui.dragging_file = None;
                }
                Task::none()
            }

            SourceAssignmentMessage::DomainClicked { domain_code } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view
                    && let Some(file_index) = assignment_ui.selected_file
                {
                    if let Some(file) = assignment_ui.source_files.get_mut(file_index) {
                        file.assigned_domain = Some(domain_code);
                        file.status = SourceFileStatus::Unassigned;
                    }
                    assignment_ui.selected_file = None;
                }
                Task::none()
            }

            // =================================================================
            // Context Menu Actions
            // =================================================================
            SourceAssignmentMessage::MarkAsMetadata { file_index } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view
                    && let Some(file) = assignment_ui.source_files.get_mut(file_index)
                {
                    file.status = SourceFileStatus::Metadata;
                    file.assigned_domain = None;
                }
                Task::none()
            }

            SourceAssignmentMessage::MarkAsSkipped { file_index } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view
                    && let Some(file) = assignment_ui.source_files.get_mut(file_index)
                {
                    file.status = SourceFileStatus::Skipped;
                    file.assigned_domain = None;
                }
                Task::none()
            }

            SourceAssignmentMessage::UnmarkFile { file_index } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view
                    && let Some(file) = assignment_ui.source_files.get_mut(file_index)
                {
                    file.status = SourceFileStatus::Unassigned;
                }
                Task::none()
            }

            SourceAssignmentMessage::UnassignFile {
                file_index,
                domain_code: _,
            } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view
                    && let Some(file) = assignment_ui.source_files.get_mut(file_index)
                {
                    file.assigned_domain = None;
                }
                Task::none()
            }

            // =================================================================
            // Search & Filter
            // =================================================================
            SourceAssignmentMessage::SourceSearchChanged(search) => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view {
                    assignment_ui.source_search = search;
                }
                Task::none()
            }

            SourceAssignmentMessage::DomainSearchChanged(search) => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view {
                    assignment_ui.domain_search = search;
                }
                Task::none()
            }

            // =================================================================
            // Navigation
            // =================================================================
            SourceAssignmentMessage::BackClicked => {
                let mode = state.view.workflow_mode();
                state.view = ViewState::home_with_mode(mode);
                Task::none()
            }

            SourceAssignmentMessage::BackConfirmed => {
                let mode = state.view.workflow_mode();
                state.view = ViewState::home_with_mode(mode);
                Task::none()
            }

            SourceAssignmentMessage::BackCancelled => {
                // Just close the confirmation dialog (if we had one)
                Task::none()
            }

            SourceAssignmentMessage::ContinueClicked => handle_continue_clicked(state),

            SourceAssignmentMessage::StudyCreated(result) => {
                // Clear loading state
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut state.view {
                    assignment_ui.is_creating_study = false;
                }

                match result {
                    Ok((study, terminology)) => {
                        tracing::info!(
                            "Study created: {} with {} domains",
                            study.study_id,
                            study.domain_count()
                        );

                        // Add to recent studies
                        let workflow_type = match state.view.workflow_mode() {
                            crate::state::WorkflowMode::Sdtm => crate::state::WorkflowType::Sdtm,
                            crate::state::WorkflowMode::Adam => crate::state::WorkflowType::Adam,
                            crate::state::WorkflowMode::Send => crate::state::WorkflowType::Send,
                        };
                        let total_rows = study.total_rows();
                        let recent_study = crate::state::RecentStudy::new(
                            study.study_folder.clone(),
                            study.study_id.clone(),
                            workflow_type,
                            study.domain_count(),
                            total_rows,
                        );
                        state.settings.general.add_recent_study(recent_study);
                        let _ = state.settings.save();

                        // Update native menu on macOS
                        #[cfg(target_os = "macos")]
                        {
                            let studies: Vec<_> = state
                                .settings
                                .general
                                .recent_sorted()
                                .iter()
                                .map(|s| {
                                    crate::menu::RecentStudyInfo::new(s.id, s.display_name.clone())
                                })
                                .collect();
                            crate::menu::update_recent_studies_menu(&studies);
                        }

                        // Store study and navigate to home
                        state.study = Some(study);
                        state.terminology = Some(terminology);
                        state.view = ViewState::home();
                    }
                    Err(e) => {
                        tracing::error!("Failed to create study: {}", e);
                        state.error = Some(crate::error::GuiError::study_load(e));
                    }
                }
                Task::none()
            }
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Handle the "Continue" button click - creates study from assigned files.
fn handle_continue_clicked(state: &mut AppState) -> Task<Message> {
    let ViewState::SourceAssignment {
        workflow_mode,
        assignment_ui,
    } = &mut state.view
    else {
        return Task::none();
    };

    // Verify all files are categorized
    if !assignment_ui.all_categorized() {
        state.error = Some(crate::error::GuiError::Operation {
            operation: "Source Assignment".to_string(),
            reason: "Please categorize all files before continuing.".to_string(),
        });
        return Task::none();
    }

    // Set loading state
    assignment_ui.is_creating_study = true;

    // Get assignments and metadata
    let assignments = assignment_ui.get_assignments();
    let metadata_files = assignment_ui.get_metadata_files();
    let folder = assignment_ui.folder.clone();
    let header_rows = state.settings.general.header_rows;
    let confidence_threshold = state.settings.general.mapping_confidence_threshold;
    let mode = *workflow_mode;

    // Spawn async task to create study
    Task::perform(
        async move {
            crate::service::study::create_study_from_assignments(
                folder,
                assignments,
                metadata_files,
                header_rows,
                confidence_threshold,
                mode,
            )
            .await
        },
        |result| Message::SourceAssignment(SourceAssignmentMessage::StudyCreated(result)),
    )
}
