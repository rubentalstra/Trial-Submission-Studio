//! Source assignment view message handlers.
//!
//! Handles all interactions for the source-to-domain assignment screen.

use iced::Task;

use crate::app::App;
use crate::message::{Message, SourceAssignmentMessage};
use crate::state::{SourceFileStatus, ViewState};

impl App {
    /// Handle source assignment view messages.
    pub fn handle_source_assignment_message(
        &mut self,
        msg: SourceAssignmentMessage,
    ) -> Task<Message> {
        match msg {
            // =================================================================
            // Drag and Drop
            // =================================================================
            SourceAssignmentMessage::DragStarted { file_index } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view {
                    assignment_ui.dragging_file = Some(file_index);
                    assignment_ui.selected_file = None; // Clear selection when dragging
                }
                Task::none()
            }

            SourceAssignmentMessage::DragOverDomain { domain_code } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view {
                    assignment_ui.hover_domain = domain_code;
                }
                Task::none()
            }

            SourceAssignmentMessage::DroppedOnDomain {
                file_index,
                domain_code,
            } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view {
                    // Assign the file to the domain
                    if let Some(file) = assignment_ui.source_files.get_mut(file_index) {
                        file.assigned_domain = Some(domain_code);
                        file.status = SourceFileStatus::Unassigned; // Reset status
                    }
                    // Clear drag state
                    assignment_ui.dragging_file = None;
                    assignment_ui.hover_domain = None;
                }
                Task::none()
            }

            SourceAssignmentMessage::DragCancelled => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view {
                    assignment_ui.dragging_file = None;
                    assignment_ui.hover_domain = None;
                }
                Task::none()
            }

            // =================================================================
            // Click-to-Assign
            // =================================================================
            SourceAssignmentMessage::FileClicked { file_index } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view {
                    // Toggle selection or select new file
                    if assignment_ui.selected_file == Some(file_index) {
                        assignment_ui.selected_file = None;
                    } else {
                        assignment_ui.selected_file = Some(file_index);
                    }
                    assignment_ui.dragging_file = None; // Clear drag when clicking
                }
                Task::none()
            }

            SourceAssignmentMessage::DomainClicked { domain_code } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view {
                    // If a file is selected, assign it to this domain
                    if let Some(file_index) = assignment_ui.selected_file {
                        if let Some(file) = assignment_ui.source_files.get_mut(file_index) {
                            file.assigned_domain = Some(domain_code);
                            file.status = SourceFileStatus::Unassigned;
                        }
                        assignment_ui.selected_file = None; // Clear selection after assignment
                    }
                }
                Task::none()
            }

            // =================================================================
            // Context Menu Actions
            // =================================================================
            SourceAssignmentMessage::MarkAsMetadata { file_index } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view
                    && let Some(file) = assignment_ui.source_files.get_mut(file_index)
                {
                    file.status = SourceFileStatus::Metadata;
                    file.assigned_domain = None; // Unassign if was assigned
                }
                Task::none()
            }

            SourceAssignmentMessage::MarkAsSkipped { file_index } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view
                    && let Some(file) = assignment_ui.source_files.get_mut(file_index)
                {
                    file.status = SourceFileStatus::Skipped;
                    file.assigned_domain = None;
                }
                Task::none()
            }

            SourceAssignmentMessage::UnmarkFile { file_index } => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view
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
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view
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
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view {
                    assignment_ui.source_search = search;
                }
                Task::none()
            }

            SourceAssignmentMessage::DomainSearchChanged(search) => {
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view {
                    assignment_ui.domain_search = search;
                }
                Task::none()
            }

            // =================================================================
            // Navigation
            // =================================================================
            SourceAssignmentMessage::BackClicked => {
                // Go back to home, preserving the workflow mode
                let mode = self.state.view.workflow_mode();
                self.state.view = ViewState::home_with_mode(mode);
                Task::none()
            }

            SourceAssignmentMessage::BackConfirmed => {
                // Go back to home, preserving the workflow mode
                let mode = self.state.view.workflow_mode();
                self.state.view = ViewState::home_with_mode(mode);
                Task::none()
            }

            SourceAssignmentMessage::BackCancelled => {
                // Just close the confirmation dialog (if we had one)
                Task::none()
            }

            SourceAssignmentMessage::ContinueClicked => {
                // Create study from assignments
                if let ViewState::SourceAssignment {
                    workflow_mode,
                    assignment_ui,
                } = &mut self.state.view
                {
                    // Verify all files are categorized
                    if !assignment_ui.all_categorized() {
                        // Show error toast or message
                        self.state.error =
                            Some("Please categorize all files before continuing.".to_string());
                        return Task::none();
                    }

                    // Set loading state
                    assignment_ui.is_creating_study = true;

                    // Get assignments and metadata
                    let assignments = assignment_ui.get_assignments();
                    let metadata_files = assignment_ui.get_metadata_files();
                    let folder = assignment_ui.folder.clone();
                    let header_rows = self.state.settings.general.header_rows;
                    let confidence_threshold =
                        self.state.settings.general.mapping_confidence_threshold;
                    let mode = *workflow_mode;

                    // Spawn async task to create study
                    return Task::perform(
                        async move {
                            crate::app::util::create_study_from_assignments(
                                folder,
                                assignments,
                                metadata_files,
                                header_rows,
                                confidence_threshold,
                                mode,
                            )
                            .await
                        },
                        |result| {
                            Message::SourceAssignment(SourceAssignmentMessage::StudyCreated(result))
                        },
                    );
                }
                Task::none()
            }

            SourceAssignmentMessage::StudyCreated(result) => {
                // Clear loading state
                if let ViewState::SourceAssignment { assignment_ui, .. } = &mut self.state.view {
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
                        let workflow_type = match self.state.view.workflow_mode() {
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
                        self.state.settings.general.add_recent_study(recent_study);
                        let _ = self.state.settings.save();

                        // Update native menu on macOS
                        #[cfg(target_os = "macos")]
                        {
                            let studies: Vec<_> = self
                                .state
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
                        self.state.study = Some(study);
                        self.state.terminology = Some(terminology);
                        self.state.view = ViewState::home();
                    }
                    Err(e) => {
                        tracing::error!("Failed to create study: {}", e);
                        self.state.error = Some(e);
                    }
                }
                Task::none()
            }
        }
    }
}
