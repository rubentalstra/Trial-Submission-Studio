//! Keyboard shortcut message handlers.
//!
//! Handles:
//! - Cmd/Ctrl+O (Open study)
//! - Cmd/Ctrl+W (Close study)
//! - Cmd/Ctrl+E (Export)
//! - Escape (Close dialog)
//! - Arrow keys, Page Up/Down, Home/End (Preview navigation)

use iced::Task;
use iced::keyboard;
use iced::keyboard::key::Named;

use crate::app::App;
use crate::message::domain_editor::PreviewMessage;
use crate::message::{DomainEditorMessage, HomeMessage, Message};
use crate::state::{EditorTab, ViewState};

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

            // Preview tab navigation (only when Preview tab is active)
            keyboard::Key::Named(Named::ArrowRight) | keyboard::Key::Named(Named::PageDown) => {
                if let ViewState::DomainEditor {
                    tab: EditorTab::Preview,
                    ..
                } = &self.state.view
                {
                    Task::done(Message::DomainEditor(DomainEditorMessage::Preview(
                        PreviewMessage::NextPage,
                    )))
                } else {
                    Task::none()
                }
            }

            keyboard::Key::Named(Named::ArrowLeft) | keyboard::Key::Named(Named::PageUp) => {
                if let ViewState::DomainEditor {
                    tab: EditorTab::Preview,
                    ..
                } = &self.state.view
                {
                    Task::done(Message::DomainEditor(DomainEditorMessage::Preview(
                        PreviewMessage::PreviousPage,
                    )))
                } else {
                    Task::none()
                }
            }

            keyboard::Key::Named(Named::Home) => {
                if let ViewState::DomainEditor {
                    tab: EditorTab::Preview,
                    ..
                } = &self.state.view
                {
                    Task::done(Message::DomainEditor(DomainEditorMessage::Preview(
                        PreviewMessage::GoToPage(0),
                    )))
                } else {
                    Task::none()
                }
            }

            keyboard::Key::Named(Named::End) => {
                if let ViewState::DomainEditor {
                    tab: EditorTab::Preview,
                    preview_cache,
                    preview_ui,
                    ..
                } = &self.state.view
                {
                    let total_rows = preview_cache.as_ref().map(|df| df.height()).unwrap_or(0);
                    let page_size = preview_ui.rows_per_page;
                    let last_page = total_rows.saturating_sub(1) / page_size;
                    Task::done(Message::DomainEditor(DomainEditorMessage::Preview(
                        PreviewMessage::GoToPage(last_page),
                    )))
                } else {
                    Task::none()
                }
            }

            _ => Task::none(),
        }
    }
}
