//! Iced subscription for macOS menu events.
//!
//! Uses a channel-based approach with a background forwarder thread.
//! The forwarder blocks on muda's event receiver and forwards events
//! through an mpsc channel, providing efficient event delivery with
//! minimal polling overhead.

use iced::Subscription;
use iced::futures::stream;
use std::time::Duration;

use super::super::MenuAction;
use super::channel::try_recv_menu_action;

/// Create an Iced subscription that receives native menu events.
///
/// This subscription uses a hybrid approach:
/// 1. A background thread blocks on muda's receiver (no busy-waiting)
/// 2. Events are forwarded through an mpsc channel
/// 3. The subscription polls the channel at 100ms intervals using async sleep
///
/// The 100ms interval is sufficient because:
/// - Menu clicks are human-initiated (reaction time ~200-300ms)
/// - The forwarder thread captures events immediately
/// - We're just polling a local mpsc channel, not the native event system
///
/// This is 2x less frequent than the previous 50ms polling, reducing
/// subscription overhead while maintaining good responsiveness.
pub fn menu_subscription() -> Subscription<Option<MenuAction>> {
    Subscription::run(menu_event_stream)
}

/// Create an async stream that polls the menu event channel.
///
/// Uses `tokio::time::sleep` for efficient async waiting between polls.
fn menu_event_stream() -> impl iced::futures::Stream<Item = Option<MenuAction>> {
    stream::unfold((), |()| async {
        // Sleep for the polling interval
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Try to receive a menu action
        let action = try_recv_menu_action();
        Some((action, ()))
    })
}
