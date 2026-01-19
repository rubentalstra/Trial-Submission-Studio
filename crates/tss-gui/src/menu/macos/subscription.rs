//! Iced subscription for macOS menu events.
//!
//! Uses a polling approach with the global menu event receiver since
//! Iced subscriptions don't easily support passing owned receivers.

use iced::Subscription;
use std::time::Duration;

use super::super::MenuAction;
use super::menu_bar::poll_menu_event;

/// Create an Iced subscription that polls for native menu events.
///
/// Polls every 50ms (reduced from 200ms) for better responsiveness.
/// This is still efficient as the poll is a non-blocking try_recv.
pub fn menu_subscription() -> Subscription<Option<MenuAction>> {
    iced::time::every(Duration::from_millis(50)).map(|_| poll_menu_event())
}
