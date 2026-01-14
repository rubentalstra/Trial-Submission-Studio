//! Keyboard shortcut message handlers.
//!
//! Handles:
//! - Cmd/Ctrl+O (Open study)
//! - Cmd/Ctrl+W (Close study)
//! - Cmd/Ctrl+E (Export)
//! - Escape (Close dialog)

use iced::Task;
use iced::keyboard;
use iced::keyboard::key::Named;

use crate::app::App;
use crate::message::{HomeMessage, Message};
use crate::state::ViewState;

impl App {
    /// Handle keyboard shortcuts.
    pub fn handle_key_press(
        &mut self,
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    ) -> Task<Message> {
        match key.as_ref() {
            // Cmd/Ctrl+O: Open study
            keyboard::Key::Character("o") if modifiers.command() => {
                Task::done(Message::Home(HomeMessage::OpenStudyClicked))
            }

            // Cmd/Ctrl+W: Close study
            keyboard::Key::Character("w") if modifiers.command() => {
                if self.state.has_study() {
                    Task::done(Message::Home(HomeMessage::CloseStudyClicked))
                } else {
                    Task::none()
                }
            }

            // Cmd/Ctrl+E: Export
            keyboard::Key::Character("e") if modifiers.command() => {
                if self.state.has_study() {
                    Task::done(Message::Navigate(ViewState::export()))
                } else {
                    Task::none()
                }
            }

            // Escape: Go home or close dialogs
            keyboard::Key::Named(Named::Escape) => match &self.state.view {
                ViewState::DomainEditor { .. } | ViewState::Export(_) => {
                    Task::done(Message::Navigate(ViewState::home()))
                }
                _ => Task::none(),
            },

            _ => Task::none(),
        }
    }
}
