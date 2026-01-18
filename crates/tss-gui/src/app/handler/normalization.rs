//! Normalization tab message handlers.
//!
//! Handles:
//! - Rule selection
//! - Rule toggling (future)

use iced::Task;

use crate::app::App;
use crate::message::Message;
use crate::message::domain_editor::NormalizationMessage;
use crate::state::ViewState;

impl App {
    /// Handle normalization tab messages.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_normalization_message(&mut self, msg: NormalizationMessage) -> Task<Message> {
        match msg {
            NormalizationMessage::RuleSelected(index) => {
                if let ViewState::DomainEditor {
                    normalization_ui, ..
                } = &mut self.state.view
                {
                    normalization_ui.selected_rule = Some(index);
                }
                Task::none()
            }
        }
    }
}
