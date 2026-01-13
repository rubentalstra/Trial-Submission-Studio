//! Menu module for Trial Submission Studio.
//!
//! This module provides both native and in-app menu support:
//!
//! - **macOS**: Uses native menu bar via `muda` crate
//! - **Windows/Linux**: Uses in-app menu bar rendered with Iced
//!
//! The menu system converts platform-specific events into unified `MenuMessage` variants.

pub mod in_app;
pub mod native;

pub use in_app::MenuBarState;
pub use native::{create_menu, ids, init_menu_for_nsapp, menu_event_receiver};

// Re-export MenuBarMenuId from message module
pub use crate::message::MenuBarMenuId;

use crate::message::MenuMessage;

/// Convert a muda menu event ID to a MenuMessage.
///
/// Returns `None` for events that don't map to application actions
/// (like predefined system events).
pub fn menu_event_to_message(event_id: &str) -> Option<MenuMessage> {
    match event_id {
        // File menu
        ids::OPEN_STUDY => Some(MenuMessage::OpenStudy),
        ids::SETTINGS => Some(MenuMessage::Settings),
        ids::EXIT => Some(MenuMessage::Quit),

        // Help menu
        ids::DOCUMENTATION => Some(MenuMessage::Documentation),
        ids::RELEASE_NOTES => Some(MenuMessage::ReleaseNotes),
        ids::VIEW_ON_GITHUB => Some(MenuMessage::ViewOnGitHub),
        ids::REPORT_ISSUE => Some(MenuMessage::ReportIssue),
        ids::VIEW_LICENSE => Some(MenuMessage::ViewLicense),
        ids::THIRD_PARTY_LICENSES => Some(MenuMessage::ThirdPartyLicenses),
        ids::CHECK_UPDATES => Some(MenuMessage::CheckUpdates),
        ids::ABOUT => Some(MenuMessage::About),

        // Unknown or predefined items (handled by system)
        _ => None,
    }
}

/// Poll for native menu events and convert them to messages.
///
/// This should be called in the application's subscription handler.
/// Returns `None` if no events are pending.
pub fn poll_native_menu_event() -> Option<MenuMessage> {
    let receiver = menu_event_receiver();

    // Try to receive an event without blocking
    match receiver.try_recv() {
        Ok(event) => {
            let id = event.id().0.as_str();
            menu_event_to_message(id)
        }
        Err(_) => None,
    }
}

/// Menu state that tracks both native menu and in-app menu state.
#[derive(Debug, Clone, Default)]
pub struct MenuState {
    /// In-app menu bar state (Windows/Linux only).
    pub bar: MenuBarState,

    /// Whether the native menu has been initialized.
    pub native_initialized: bool,
}

impl MenuState {
    /// Create a new menu state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the native menu.
    ///
    /// On macOS, this creates the native menu bar.
    /// On other platforms, this is a no-op.
    #[cfg(target_os = "macos")]
    pub fn init_native(&mut self) -> muda::Menu {
        let menu = create_menu();
        init_menu_for_nsapp(&menu);
        self.native_initialized = true;
        menu
    }

    /// Initialize the native menu (no-op on non-macOS).
    #[cfg(not(target_os = "macos"))]
    pub fn init_native(&mut self) -> muda::Menu {
        self.native_initialized = true;
        create_menu()
    }

    /// Toggle an in-app menu.
    pub fn toggle_menu(&mut self, menu_id: MenuBarMenuId) {
        self.bar.toggle(menu_id);
    }

    /// Close all in-app menus.
    pub fn close_menus(&mut self) {
        self.bar.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_event_to_message() {
        assert!(matches!(
            menu_event_to_message(ids::OPEN_STUDY),
            Some(MenuMessage::OpenStudy)
        ));
        assert!(matches!(
            menu_event_to_message(ids::SETTINGS),
            Some(MenuMessage::Settings)
        ));
        assert!(matches!(
            menu_event_to_message(ids::ABOUT),
            Some(MenuMessage::About)
        ));
        assert!(menu_event_to_message("unknown_id").is_none());
    }

    #[test]
    fn test_menu_bar_state() {
        let mut state = MenuBarState::new();
        assert!(state.open_menu.is_none());

        state.toggle(MenuBarMenuId::File);
        assert!(state.is_open(MenuBarMenuId::File));
        assert!(!state.is_open(MenuBarMenuId::Edit));

        state.toggle(MenuBarMenuId::File);
        assert!(!state.is_open(MenuBarMenuId::File));

        state.toggle(MenuBarMenuId::Help);
        assert!(state.is_open(MenuBarMenuId::Help));

        state.close();
        assert!(state.open_menu.is_none());
    }
}
