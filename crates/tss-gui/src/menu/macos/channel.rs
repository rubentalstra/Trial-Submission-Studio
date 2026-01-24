//! Channel-based menu event delivery for macOS.
//!
//! This module provides a more efficient event delivery mechanism than
//! pure polling. A background thread blocks on muda's event receiver and
//! forwards events through an mpsc channel, allowing the subscription to
//! check for events without the overhead of repeated try_recv calls on
//! muda's global receiver.
//!
//! # Architecture
//!
//! ```text
//! [muda MenuEvent] --blocking--> [Forwarder Thread] --mpsc--> [Subscription]
//! ```
//!
//! The forwarder thread blocks on muda's receiver, so there's no busy-waiting.
//! Events are immediately forwarded to our channel where the subscription
//! can poll with minimal overhead.

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Mutex, OnceLock};
use std::thread;

use muda::MenuEvent;

use super::super::MenuAction;
use super::menu_bar::menu_id_to_action;

/// Thread-safe wrapper for the receiver.
///
/// The `Receiver` from `std::sync::mpsc` is not `Sync` by default because it uses
/// internal mutability. However, we wrap it in a `Mutex` which provides the necessary
/// synchronization. This makes it safe to share across threads.
struct MenuReceiver(Mutex<Receiver<MenuAction>>);

// SAFETY: The `Mutex` ensures that only one thread can access the `Receiver` at a time.
// All access to the receiver goes through `Mutex::lock()`, providing proper synchronization.
// The receiver is never moved or accessed without holding the lock.
#[allow(unsafe_code)]
unsafe impl Sync for MenuReceiver {}

/// Global sender for the menu channel.
///
/// The sender is `Sync` and can be cloned, so it's safe to store directly.
static MENU_SENDER: OnceLock<Sender<MenuAction>> = OnceLock::new();

/// Global receiver for the menu channel.
///
/// The receiver is wrapped in a Mutex for thread-safety.
static MENU_RECEIVER: OnceLock<MenuReceiver> = OnceLock::new();

/// Flag to track if the forwarder thread has been started.
static FORWARDER_STARTED: OnceLock<()> = OnceLock::new();

/// Initialize the menu event channel and start the forwarder thread.
///
/// This should be called once during app initialization, after the menu
/// is created. It's safe to call multiple times - subsequent calls are no-ops.
pub fn init_menu_channel() {
    // Initialize the channel (sender and receiver separately)
    let (sender, receiver) = mpsc::channel();

    MENU_SENDER.get_or_init(|| sender);
    MENU_RECEIVER.get_or_init(|| MenuReceiver(Mutex::new(receiver)));

    // Start the forwarder thread (only once)
    // Graceful degradation: menu events may not work if spawn fails (#147)
    FORWARDER_STARTED.get_or_init(|| {
        match thread::Builder::new()
            .name("menu-event-forwarder".into())
            .spawn(forwarder_thread)
        {
            Ok(_) => tracing::debug!("Menu event forwarder thread started"),
            Err(e) => tracing::error!(
                error = %e,
                "Failed to spawn menu forwarder thread - menu events may not work"
            ),
        }
    });
}

/// Background thread that forwards muda events to our channel.
///
/// This thread blocks on muda's receiver, converting events to MenuActions
/// and forwarding them through our mpsc channel. This design means:
///
/// 1. No busy-waiting - we block on the muda receiver
/// 2. Immediate forwarding - events are sent as soon as they arrive
/// 3. Minimal subscription overhead - polling our channel is cheap
fn forwarder_thread() {
    let receiver = MenuEvent::receiver();

    // Get our sender
    let Some(sender) = MENU_SENDER.get() else {
        eprintln!("Menu channel not initialized before forwarder started");
        return;
    };

    // Block on muda's receiver and forward events
    // When the muda channel disconnects (app shutting down), the loop exits
    while let Ok(event) = receiver.recv() {
        let id = event.id().0.as_str();
        if let Some(action) = menu_id_to_action(id) {
            // Send to our channel, ignore errors (receiver dropped = app shutting down)
            let _ = sender.send(action);
        }
    }
}

/// Try to receive a menu action from the channel.
///
/// This is a non-blocking operation that returns immediately.
/// Used by the subscription to poll for events.
pub fn try_recv_menu_action() -> Option<MenuAction> {
    MENU_RECEIVER.get().and_then(|wrapper| {
        // Lock the mutex briefly to try_recv
        let receiver = wrapper.0.lock().ok()?;
        match receiver.try_recv() {
            Ok(action) => Some(action),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    })
}

/// Send a menu action directly to the channel.
///
/// This can be used for programmatic menu triggers or testing.
/// Returns true if the action was sent successfully.
#[allow(dead_code)]
pub fn send_menu_action(action: MenuAction) -> bool {
    MENU_SENDER
        .get()
        .map(|sender| sender.send(action).is_ok())
        .unwrap_or(false)
}
