//! Preview tab message handlers.
//!
//! Handles:
//! - Pagination (next/prev page)
//! - Preview rebuild trigger
//! - Preview computation callback

use iced::Task;

use crate::app::App;
use crate::message::Message;
use crate::message::domain_editor::PreviewMessage;
use crate::service::preview::{PreviewInput, compute_preview};
use crate::state::ViewState;

impl App {
    /// Handle preview tab messages.
    pub fn handle_preview_message(&mut self, msg: PreviewMessage) -> Task<Message> {
        // Get current domain code
        let domain_code = match &self.state.view {
            ViewState::DomainEditor { domain, .. } => domain.clone(),
            _ => return Task::none(),
        };

        match msg {
            PreviewMessage::RebuildPreview => {
                // Get domain data for preview
                let domain = match self
                    .state
                    .study
                    .as_ref()
                    .and_then(|s| s.domain(&domain_code))
                {
                    Some(d) => d,
                    None => return Task::none(),
                };

                // Mark as rebuilding
                if let ViewState::DomainEditor { preview_ui, .. } = &mut self.state.view {
                    preview_ui.is_rebuilding = true;
                    preview_ui.error = None;
                }

                // Build preview input
                let input = PreviewInput {
                    source_df: domain.source.data.clone(),
                    mapping: domain.mapping.clone(),
                    ct_registry: self.state.terminology.clone(),
                };

                let domain_for_result = domain_code.clone();

                // Start async preview computation
                Task::perform(compute_preview(input), move |result| {
                    Message::PreviewReady {
                        domain: domain_for_result,
                        result: result.map_err(|e| e.to_string()),
                    }
                })
            }

            PreviewMessage::GoToPage(page) => {
                if let ViewState::DomainEditor { preview_ui, .. } = &mut self.state.view {
                    preview_ui.current_page = page;
                }
                Task::none()
            }

            PreviewMessage::NextPage => {
                if let ViewState::DomainEditor { preview_ui, .. } = &mut self.state.view {
                    preview_ui.current_page = preview_ui.current_page.saturating_add(1);
                }
                Task::none()
            }

            PreviewMessage::PreviousPage => {
                if let ViewState::DomainEditor { preview_ui, .. } = &mut self.state.view {
                    preview_ui.current_page = preview_ui.current_page.saturating_sub(1);
                }
                Task::none()
            }

            PreviewMessage::RowsPerPageChanged(rows) => {
                if let ViewState::DomainEditor { preview_ui, .. } = &mut self.state.view {
                    preview_ui.rows_per_page = rows;
                    preview_ui.current_page = 0; // Reset to first page
                }
                Task::none()
            }
        }
    }
}
