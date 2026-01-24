//! Keyboard shortcut message handlers.
//!
//! Handles:
//! - Cmd/Ctrl+N (New project)
//! - Cmd/Ctrl+O (Open project)
//! - Cmd/Ctrl+S (Save project)
//! - Cmd/Ctrl+Shift+S (Save project as)
//! - Cmd/Ctrl+W (Close project)
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
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_key_press(
        &mut self,
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    ) -> Task<Message> {
        match key.as_ref() {
            // Cmd/Ctrl+N: New project
            keyboard::Key::Character("n") if modifiers.command() && !modifiers.shift() => {
                Task::done(Message::NewProject)
            }

            // Cmd/Ctrl+O: Open project (.tss file)
            keyboard::Key::Character("o") if modifiers.command() => {
                Task::done(Message::OpenProject)
            }

            // Cmd/Ctrl+S: Save project
            keyboard::Key::Character("s") if modifiers.command() && !modifiers.shift() => {
                if self.state.has_study() {
                    Task::done(Message::SaveProject)
                } else {
                    Task::none()
                }
            }

            // Cmd/Ctrl+Shift+S: Save project as
            keyboard::Key::Character("s") if modifiers.command() && modifiers.shift() => {
                if self.state.has_study() {
                    Task::done(Message::SaveProjectAs)
                } else {
                    Task::none()
                }
            }

            // Cmd/Ctrl+W: Close project
            keyboard::Key::Character("w") if modifiers.command() => {
                if self.state.has_study() {
                    Task::done(Message::Home(HomeMessage::CloseProjectClicked))
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
                ViewState::DomainEditor(_) | ViewState::Export(_) => {
                    Task::done(Message::Navigate(ViewState::home()))
                }
                _ => Task::none(),
            },

            // Preview tab navigation (only when Preview tab is active)
            keyboard::Key::Named(Named::ArrowRight) | keyboard::Key::Named(Named::PageDown) => {
                if let ViewState::DomainEditor(editor) = &self.state.view {
                    if editor.tab == EditorTab::Preview {
                        return Task::done(Message::DomainEditor(DomainEditorMessage::Preview(
                            PreviewMessage::NextPage,
                        )));
                    }
                }
                Task::none()
            }

            keyboard::Key::Named(Named::ArrowLeft) | keyboard::Key::Named(Named::PageUp) => {
                if let ViewState::DomainEditor(editor) = &self.state.view {
                    if editor.tab == EditorTab::Preview {
                        return Task::done(Message::DomainEditor(DomainEditorMessage::Preview(
                            PreviewMessage::PreviousPage,
                        )));
                    }
                }
                Task::none()
            }

            keyboard::Key::Named(Named::Home) => {
                if let ViewState::DomainEditor(editor) = &self.state.view {
                    if editor.tab == EditorTab::Preview {
                        return Task::done(Message::DomainEditor(DomainEditorMessage::Preview(
                            PreviewMessage::GoToPage(0),
                        )));
                    }
                }
                Task::none()
            }

            keyboard::Key::Named(Named::End) => {
                if let ViewState::DomainEditor(editor) = &self.state.view {
                    if editor.tab == EditorTab::Preview {
                        let total_rows = editor
                            .preview_cache
                            .as_ref()
                            .map(polars::prelude::DataFrame::height)
                            .unwrap_or(0);
                        let page_size = editor.preview_ui.rows_per_page;
                        let last_page = total_rows.saturating_sub(1) / page_size;
                        return Task::done(Message::DomainEditor(DomainEditorMessage::Preview(
                            PreviewMessage::GoToPage(last_page),
                        )));
                    }
                }
                Task::none()
            }

            _ => Task::none(),
        }
    }
}
