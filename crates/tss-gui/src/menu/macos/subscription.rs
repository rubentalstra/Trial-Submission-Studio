//! Iced subscription for macOS menu events.
//!
//! Uses a polling approach with the global menu event receiver.
//! The polling interval is optimized for responsiveness while minimizing
//! CPU overhead.

use iced::Subscription;
use std::time::Duration;

use super::super::MenuAction;
use super::menu_bar::poll_menu_event;

/// Create an Iced subscription that polls for native menu events.
///
/// Uses a 50ms polling interval which provides good responsiveness
/// while keeping CPU usage minimal (the poll is a non-blocking try_recv).
///
/// # Design Notes
///
/// While a blocking receiver approach would be more efficient, Iced's
/// subscription model works best with the polling pattern. The overhead
/// of 20 polls/second is negligible compared to typical UI rendering.
pub fn menu_subscription() -> Subscription<Option<MenuAction>> {
    iced::time::every(Duration::from_millis(50)).map(|_| poll_menu_event())
}
