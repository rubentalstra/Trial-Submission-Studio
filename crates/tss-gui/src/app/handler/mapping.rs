//! Mapping tab message handlers.
//!
//! Handles:
//! - Accept/reject mapping suggestions
//! - Manual column mapping
//! - Clear mappings
//! - Mark variable as not collected

use iced::Task;

use crate::app::App;
use crate::message::Message;
use crate::message::domain_editor::MappingMessage;
use crate::state::{NotCollectedEdit, ViewState};

impl App {
    /// Handle mapping tab messages.
    pub fn handle_mapping_message(&mut self, msg: MappingMessage) -> Task<Message> {
        // Get current domain code
        let domain_code = match &self.state.view {
            ViewState::DomainEditor { domain, .. } => domain.clone(),
            _ => return Task::none(),
        };

        match msg {
            MappingMessage::VariableSelected(idx) => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.selected_variable = Some(idx);
                }
                Task::none()
            }

            MappingMessage::SearchChanged(text) => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.search_filter = text;
                }
                Task::none()
            }

            MappingMessage::SearchCleared => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.search_filter.clear();
                }
                Task::none()
            }

            MappingMessage::AcceptSuggestion(variable) => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    let _ = domain.mapping.accept_suggestion(&variable);
                    domain.invalidate_validation();
                }
                // Invalidate preview cache
                if let ViewState::DomainEditor { preview_cache, .. } = &mut self.state.view {
                    *preview_cache = None;
                }
                Task::none()
            }

            MappingMessage::ClearMapping(variable) => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    domain.mapping.clear_assignment(&variable);
                    domain.invalidate_validation();
                }
                // Invalidate preview cache
                if let ViewState::DomainEditor { preview_cache, .. } = &mut self.state.view {
                    *preview_cache = None;
                }
                Task::none()
            }

            MappingMessage::ManualMap { variable, column } => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    let _ = domain.mapping.accept_manual(&variable, &column);
                    domain.invalidate_validation();
                }
                // Invalidate preview cache
                if let ViewState::DomainEditor { preview_cache, .. } = &mut self.state.view {
                    *preview_cache = None;
                }
                Task::none()
            }

            MappingMessage::MarkNotCollected { variable } => {
                // Start inline editing for new "Not Collected" marking
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.not_collected_edit = Some(NotCollectedEdit {
                        variable,
                        reason: String::new(),
                    });
                }
                Task::none()
            }

            MappingMessage::NotCollectedReasonChanged(reason) => {
                // Update the reason text while editing
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view
                    && let Some(edit) = &mut mapping_ui.not_collected_edit
                {
                    edit.reason = reason;
                }
                Task::none()
            }

            MappingMessage::NotCollectedSave { variable, reason } => {
                // Validate reason is not empty
                if reason.trim().is_empty() {
                    return Task::none();
                }

                // Save the "Not Collected" status with reason
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    let _ = domain.mapping.mark_not_collected(&variable, &reason);
                    domain.invalidate_validation();
                }
                // Clear edit state and invalidate preview cache
                if let ViewState::DomainEditor {
                    mapping_ui,
                    preview_cache,
                    ..
                } = &mut self.state.view
                {
                    mapping_ui.not_collected_edit = None;
                    *preview_cache = None;
                }
                Task::none()
            }

            MappingMessage::NotCollectedCancel => {
                // Cancel inline editing
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.not_collected_edit = None;
                }
                Task::none()
            }

            MappingMessage::EditNotCollectedReason {
                variable,
                current_reason,
            } => {
                // Start editing an existing "Not Collected" reason
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.not_collected_edit = Some(NotCollectedEdit {
                        variable,
                        reason: current_reason,
                    });
                }
                Task::none()
            }

            MappingMessage::ClearNotCollected(variable) => {
                // Revert "Not Collected" back to unmapped
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    domain.mapping.clear_assignment(&variable);
                    domain.invalidate_validation();
                }
                // Invalidate preview cache
                if let ViewState::DomainEditor { preview_cache, .. } = &mut self.state.view {
                    *preview_cache = None;
                }
                Task::none()
            }

            MappingMessage::MarkOmitted(variable) => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    let _ = domain.mapping.mark_omit(&variable);
                    domain.invalidate_validation();
                }
                // Invalidate preview cache
                if let ViewState::DomainEditor { preview_cache, .. } = &mut self.state.view {
                    *preview_cache = None;
                }
                Task::none()
            }

            MappingMessage::ClearOmitted(variable) => {
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    domain.mapping.clear_assignment(&variable);
                    domain.invalidate_validation();
                }
                // Invalidate preview cache
                if let ViewState::DomainEditor { preview_cache, .. } = &mut self.state.view {
                    *preview_cache = None;
                }
                Task::none()
            }

            MappingMessage::FilterUnmappedToggled(enabled) => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.filter_unmapped = enabled;
                }
                Task::none()
            }

            MappingMessage::FilterRequiredToggled(enabled) => {
                if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                    mapping_ui.filter_required = enabled;
                }
                Task::none()
            }
        }
    }
}
