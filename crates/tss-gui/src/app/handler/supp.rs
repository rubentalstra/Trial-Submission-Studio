//! SUPP qualifier tab message handlers.
//!
//! Handles:
//! - SUPP column configuration
//! - Field editing (QLABEL, QORIG, QEVAL)
//! - Save/skip actions
//! - Filter mode changes

use iced::Task;

use crate::app::App;
use crate::message::Message;
use crate::message::domain_editor::SuppMessage;
use crate::state::{SuppAction, SuppColumnConfig, SuppEditDraft, ViewState};

impl App {
    /// Handle SUPP tab messages.
    ///
    /// # Message Flow
    ///
    /// - **Pending columns**: Field edits update `supp_config` directly
    /// - **Included columns (editing)**: Field edits update `edit_draft`, committed on Save
    /// - **Actions**: AddToSupp, Skip, UndoAction change `supp_config.action`
    pub fn handle_supp_message(&mut self, msg: SuppMessage) -> Task<Message> {
        // Get current domain code
        let domain_code = match &self.state.view {
            ViewState::DomainEditor { domain, .. } => domain.clone(),
            _ => return Task::none(),
        };

        match msg {
            // =================================================================
            // NAVIGATION & FILTERING
            // =================================================================
            SuppMessage::ColumnSelected(col_name) => {
                // Clear any edit draft when changing selection
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.selected_column = Some(col_name.clone());
                    supp_ui.edit_draft = None;
                }
                // Initialize config if not exists
                if let Some(domain) = self
                    .state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                {
                    domain
                        .supp_config
                        .entry(col_name.clone())
                        .or_insert_with(|| SuppColumnConfig::from_column(&col_name));
                }
                Task::none()
            }

            SuppMessage::SearchChanged(text) => {
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.search_filter = text;
                }
                Task::none()
            }

            SuppMessage::FilterModeChanged(mode) => {
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.filter_mode = mode;
                }
                Task::none()
            }

            // =================================================================
            // FIELD EDITING
            // =================================================================
            SuppMessage::QnamChanged(value) => {
                // Enforce max 8 chars, uppercase
                let value = value.chars().take(8).collect::<String>().to_uppercase();
                self.update_supp_field(&domain_code, |config, draft| {
                    if let Some(d) = draft {
                        d.qnam = value;
                    } else {
                        config.qnam = value;
                    }
                });
                Task::none()
            }

            SuppMessage::QlabelChanged(value) => {
                // Enforce max 40 chars
                let value: String = value.chars().take(40).collect();
                self.update_supp_field(&domain_code, |config, draft| {
                    if let Some(d) = draft {
                        d.qlabel = value;
                    } else {
                        config.qlabel = value;
                    }
                });
                Task::none()
            }

            SuppMessage::QorigChanged(value) => {
                self.update_supp_field(&domain_code, |config, draft| {
                    if let Some(d) = draft {
                        d.qorig = value;
                    } else {
                        config.qorig = value;
                    }
                });
                Task::none()
            }

            SuppMessage::QevalChanged(value) => {
                self.update_supp_field(&domain_code, |config, draft| {
                    if let Some(d) = draft {
                        d.qeval = value.clone();
                    } else {
                        config.qeval = if value.is_empty() { None } else { Some(value) };
                    }
                });
                Task::none()
            }

            // =================================================================
            // ACTIONS
            // =================================================================
            SuppMessage::AddToSupp => {
                // Get selected column
                let col = match &self.state.view {
                    ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                    _ => None,
                };

                if let Some(col_name) = col {
                    if let Some(domain) = self
                        .state
                        .study
                        .as_mut()
                        .and_then(|s| s.domain_mut(&domain_code))
                    {
                        if let Some(config) = domain.supp_config.get_mut(&col_name) {
                            // Validate required fields before adding
                            if config.qnam.trim().is_empty() || config.qlabel.trim().is_empty() {
                                // Don't add - QNAM and QLABEL are required
                                return Task::none();
                            }
                            config.action = SuppAction::Include;
                        }
                    }
                }
                // Clear draft after action
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.edit_draft = None;
                }
                Task::none()
            }

            SuppMessage::Skip => {
                let col = match &self.state.view {
                    ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                    _ => None,
                };

                if let Some(col_name) = col {
                    if let Some(domain) = self
                        .state
                        .study
                        .as_mut()
                        .and_then(|s| s.domain_mut(&domain_code))
                    {
                        if let Some(config) = domain.supp_config.get_mut(&col_name) {
                            config.action = SuppAction::Skip;
                        }
                    }
                }
                // Clear draft after action
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.edit_draft = None;
                }
                Task::none()
            }

            SuppMessage::UndoAction => {
                let col = match &self.state.view {
                    ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                    _ => None,
                };

                if let Some(col_name) = col {
                    if let Some(domain) = self
                        .state
                        .study
                        .as_mut()
                        .and_then(|s| s.domain_mut(&domain_code))
                    {
                        if let Some(config) = domain.supp_config.get_mut(&col_name) {
                            config.action = SuppAction::Pending;
                        }
                    }
                }
                // Clear draft after action
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.edit_draft = None;
                }
                Task::none()
            }

            // =================================================================
            // EDIT MODE (for included columns)
            // =================================================================
            SuppMessage::StartEdit => {
                // Get selected column and create draft from its config
                let col = match &self.state.view {
                    ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                    _ => None,
                };

                if let Some(col_name) = &col {
                    if let Some(domain) = self
                        .state
                        .study
                        .as_ref()
                        .and_then(|s| s.domain(&domain_code))
                    {
                        if let Some(config) = domain.supp_config.get(col_name) {
                            let draft = SuppEditDraft::from_config(config);
                            if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                                supp_ui.edit_draft = Some(draft);
                            }
                        }
                    }
                }
                Task::none()
            }

            SuppMessage::SaveEdit => {
                // Apply draft to config
                let (col, draft) = match &self.state.view {
                    ViewState::DomainEditor { supp_ui, .. } => {
                        (supp_ui.selected_column.clone(), supp_ui.edit_draft.clone())
                    }
                    _ => (None, None),
                };

                if let (Some(col_name), Some(draft)) = (col, draft) {
                    // Validate required fields before saving
                    if draft.qnam.trim().is_empty() || draft.qlabel.trim().is_empty() {
                        // Don't save - QNAM and QLABEL are required
                        return Task::none();
                    }

                    if let Some(domain) = self
                        .state
                        .study
                        .as_mut()
                        .and_then(|s| s.domain_mut(&domain_code))
                    {
                        if let Some(config) = domain.supp_config.get_mut(&col_name) {
                            config.qnam = draft.qnam;
                            config.qlabel = draft.qlabel;
                            config.qorig = draft.qorig;
                            config.qeval = if draft.qeval.is_empty() {
                                None
                            } else {
                                Some(draft.qeval)
                            };
                        }
                    }
                }
                // Clear draft
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.edit_draft = None;
                }
                Task::none()
            }

            SuppMessage::CancelEdit => {
                // Just discard the draft
                if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                    supp_ui.edit_draft = None;
                }
                Task::none()
            }
        }
    }

    /// Helper to update a SUPP field, routing to draft or config as appropriate.
    pub fn update_supp_field<F>(&mut self, domain_code: &str, update: F)
    where
        F: FnOnce(&mut SuppColumnConfig, Option<&mut SuppEditDraft>),
    {
        // Get selected column
        let col = match &self.state.view {
            ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
            _ => return,
        };

        let Some(col_name) = col else { return };

        // Check if we're in edit mode (have a draft)
        let is_editing = match &self.state.view {
            ViewState::DomainEditor { supp_ui, .. } => supp_ui.edit_draft.is_some(),
            _ => false,
        };

        if is_editing {
            // Update the draft
            if let ViewState::DomainEditor { supp_ui, .. } = &mut self.state.view {
                if let Some(draft) = &mut supp_ui.edit_draft {
                    // Get a dummy config to satisfy the closure signature
                    let mut dummy = SuppColumnConfig::from_column("");
                    update(&mut dummy, Some(draft));
                }
            }
        } else {
            // Update the config directly
            if let Some(domain) = self
                .state
                .study
                .as_mut()
                .and_then(|s| s.domain_mut(domain_code))
            {
                if let Some(config) = domain.supp_config.get_mut(&col_name) {
                    update(config, None);
                }
            }
        }
    }
}
