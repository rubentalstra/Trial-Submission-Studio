//! Main application module for Trial Submission Studio.
//!
//! This module implements the Iced 0.14.0 application using the builder pattern.
//! The architecture follows the Elm pattern: State → Message → Update → View.

use iced::keyboard;
use iced::widget::{column, container, text};
use iced::{Element, Subscription, Task, Theme};

use crate::message::Message;
use crate::state::navigation::View;
use crate::theme::clinical_light;

// =============================================================================
// APPLICATION STATE
// =============================================================================

/// Main application state.
///
/// This is the root state container for Trial Submission Studio.
/// It holds all UI and domain state, organized by concern.
pub struct App {
    /// Current view/route
    view: View,
    // TODO: Add full state once services are ported
    // pub study: Option<StudyState>,
    // pub settings: Settings,
    // pub ui: UiState,
}

// =============================================================================
// APPLICATION IMPLEMENTATION
// =============================================================================

impl App {
    /// Create a new application instance.
    ///
    /// This is called once at startup. Returns the initial state and any
    /// startup tasks (e.g., loading settings, checking for updates).
    pub fn new() -> (Self, Task<Message>) {
        let app = Self { view: View::Home };

        // TODO: Add startup tasks
        // - Load settings from disk
        // - Check for updates (if enabled)
        // - Load recent studies list
        let startup_tasks = Task::none();

        (app, startup_tasks)
    }

    /// Update application state in response to a message.
    ///
    /// This is the core of the Elm architecture - all state changes happen here.
    /// Returns any follow-up tasks to execute.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation
            Message::Navigate(view) => {
                self.view = view;
                Task::none()
            }

            // Workflow mode change
            Message::SetWorkflowMode(_mode) => {
                // TODO: Store workflow mode in state
                Task::none()
            }

            // Home view messages
            Message::Home(home_msg) => self.handle_home_message(home_msg),

            // Domain editor messages
            Message::DomainEditor(editor_msg) => self.handle_domain_editor_message(editor_msg),

            // Export messages
            Message::Export(export_msg) => self.handle_export_message(export_msg),

            // Dialog messages
            Message::Dialog(dialog_msg) => self.handle_dialog_message(dialog_msg),

            // Menu messages
            Message::Menu(menu_msg) => self.handle_menu_message(menu_msg),

            // Background task results
            Message::StudyLoaded(result) => self.handle_study_loaded(result),

            Message::PreviewReady { domain, result } => self.handle_preview_ready(&domain, result),

            Message::ValidationComplete { domain, report } => {
                self.handle_validation_complete(&domain, report)
            }

            Message::UpdateCheckComplete(result) => self.handle_update_check_complete(result),

            // Global events
            Message::KeyPressed(key, modifiers) => self.handle_key_press(key, modifiers),

            Message::Tick => {
                // Periodic tick for animations or polling
                Task::none()
            }

            // File dialog result
            Message::FolderSelected(path) => {
                if let Some(_folder) = path {
                    // TODO: Load study from folder
                }
                Task::none()
            }

            // No-op placeholder
            Message::Noop => Task::none(),
        }
    }

    /// Render the current view.
    ///
    /// This is a pure function that produces the UI based on current state.
    /// No side effects should happen here.
    pub fn view(&self) -> Element<'_, Message> {
        // Main content based on current view
        let content: Element<'_, Message> = match &self.view {
            View::Home => self.view_home(),
            View::DomainEditor { domain, tab } => self.view_domain_editor(domain, *tab),
            View::Export => self.view_export(),
        };

        // Wrap in main container
        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    /// Get the window title.
    ///
    /// This can change based on current state (e.g., show study name).
    pub fn title(&self) -> String {
        match &self.view {
            View::Home => "Trial Submission Studio".to_string(),
            View::DomainEditor { domain, .. } => {
                format!("{} - Trial Submission Studio", domain)
            }
            View::Export => "Export - Trial Submission Studio".to_string(),
        }
    }

    /// Get the current theme.
    ///
    /// Returns the Professional Clinical light theme.
    pub fn theme(&self) -> Theme {
        clinical_light()
    }

    /// Subscribe to runtime events.
    ///
    /// This sets up event listeners for keyboard shortcuts, timers, etc.
    pub fn subscription(&self) -> Subscription<Message> {
        // Keyboard event subscription using Iced 0.14.0 API
        keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Message::KeyPressed(key, modifiers)
            }
            _ => Message::Noop,
        })
    }
}

// =============================================================================
// VIEW IMPLEMENTATIONS (Placeholder)
// =============================================================================

impl App {
    /// Render the home view.
    fn view_home(&self) -> Element<'_, Message> {
        // Placeholder - will be implemented in Phase 3
        column![
            text("Trial Submission Studio").size(32),
            text("Welcome! Open a study folder to begin.").size(16),
        ]
        .spacing(16)
        .padding(32)
        .into()
    }

    /// Render the domain editor view.
    fn view_domain_editor(
        &self,
        domain: &str,
        tab: crate::state::navigation::EditorTab,
    ) -> Element<'_, Message> {
        // Placeholder - will be implemented in Phase 4
        column![
            text(format!("Domain: {}", domain)).size(24),
            text(format!("Tab: {}", tab.name())).size(16),
        ]
        .spacing(16)
        .padding(32)
        .into()
    }

    /// Render the export view.
    fn view_export(&self) -> Element<'_, Message> {
        // Placeholder - will be implemented in Phase 5
        column![
            text("Export").size(24),
            text("Configure and export your domains.").size(16),
        ]
        .spacing(16)
        .padding(32)
        .into()
    }
}

// =============================================================================
// MESSAGE HANDLERS (Placeholder)
// =============================================================================

impl App {
    fn handle_home_message(&mut self, _msg: crate::message::HomeMessage) -> Task<Message> {
        // TODO: Implement in Phase 3
        Task::none()
    }

    fn handle_domain_editor_message(
        &mut self,
        _msg: crate::message::DomainEditorMessage,
    ) -> Task<Message> {
        // TODO: Implement in Phase 4
        Task::none()
    }

    fn handle_export_message(&mut self, _msg: crate::message::ExportMessage) -> Task<Message> {
        // TODO: Implement in Phase 5
        Task::none()
    }

    fn handle_dialog_message(&mut self, _msg: crate::message::DialogMessage) -> Task<Message> {
        // TODO: Implement in Phase 5
        Task::none()
    }

    fn handle_menu_message(&mut self, _msg: crate::message::MenuMessage) -> Task<Message> {
        // TODO: Implement in Phase 6
        Task::none()
    }

    fn handle_study_loaded(
        &mut self,
        _result: Result<crate::state::StudyState, String>,
    ) -> Task<Message> {
        // TODO: Implement in Phase 3
        Task::none()
    }

    fn handle_preview_ready(
        &mut self,
        _domain: &str,
        _result: Result<polars::prelude::DataFrame, String>,
    ) -> Task<Message> {
        // TODO: Implement in Phase 4
        Task::none()
    }

    fn handle_validation_complete(
        &mut self,
        _domain: &str,
        _report: tss_validate::ValidationReport,
    ) -> Task<Message> {
        // TODO: Implement in Phase 4
        Task::none()
    }

    fn handle_update_check_complete(
        &mut self,
        _result: Result<Option<crate::message::UpdateInfo>, String>,
    ) -> Task<Message> {
        // TODO: Implement in Phase 5
        Task::none()
    }

    fn handle_key_press(
        &mut self,
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    ) -> Task<Message> {
        // Global keyboard shortcuts
        use keyboard::key::Named;

        match key.as_ref() {
            // Cmd/Ctrl+O: Open study
            keyboard::Key::Character("o") if modifiers.command() => {
                Task::done(Message::Menu(crate::message::MenuMessage::OpenStudy))
            }

            // Cmd/Ctrl+W: Close study
            keyboard::Key::Character("w") if modifiers.command() => {
                Task::done(Message::Menu(crate::message::MenuMessage::CloseStudy))
            }

            // Cmd/Ctrl+,: Settings
            keyboard::Key::Character(",") if modifiers.command() => {
                Task::done(Message::Menu(crate::message::MenuMessage::Settings))
            }

            // Cmd/Ctrl+E: Export
            keyboard::Key::Character("e") if modifiers.command() => {
                Task::done(Message::Navigate(View::Export))
            }

            // Escape: Go home or close dialog
            keyboard::Key::Named(Named::Escape) => {
                if self.view.is_domain_editor() || self.view.is_export() {
                    Task::done(Message::Navigate(View::Home))
                } else {
                    Task::none()
                }
            }

            _ => Task::none(),
        }
    }
}
