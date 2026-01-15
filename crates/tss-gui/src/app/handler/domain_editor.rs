//! Domain editor message handlers.
//!
//! Handles:
//! - Tab navigation (Map, Normalize, Validate, Preview, SUPP)
//! - Delegates to specific tab handlers

use iced::Task;

use crate::app::App;
use crate::message::{DomainEditorMessage, Message};
use crate::state::ViewState;

impl App {
    /// Handle domain editor messages.
    pub fn handle_domain_editor_message(&mut self, msg: DomainEditorMessage) -> Task<Message> {
        match msg {
            DomainEditorMessage::TabSelected(tab) => {
                if let ViewState::DomainEditor {
                    tab: current_tab, ..
                } = &mut self.state.view
                {
                    *current_tab = tab;
                }
                Task::none()
            }

            DomainEditorMessage::BackClicked => {
                self.state.view = ViewState::home();
                Task::none()
            }

            DomainEditorMessage::Mapping(mapping_msg) => self.handle_mapping_message(mapping_msg),

            DomainEditorMessage::Normalization(norm_msg) => {
                self.handle_normalization_message(norm_msg)
            }

            DomainEditorMessage::Validation(validation_msg) => {
                self.handle_validation_message(validation_msg)
            }

            DomainEditorMessage::Preview(preview_msg) => self.handle_preview_message(preview_msg),

            DomainEditorMessage::Supp(supp_msg) => self.handle_supp_message(supp_msg),
        }
    }
}
