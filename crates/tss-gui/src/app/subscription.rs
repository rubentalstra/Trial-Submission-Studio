//! Application subscriptions.
//!
//! This module centralizes all Iced subscriptions for the application.
//! Subscriptions are reactive event sources that run alongside the app.
//!
//! # Subscription Overview
//!
//! | Subscription | Interval | Condition | Purpose |
//! |--------------|----------|-----------|---------|
//! | Keyboard | Continuous | Always | Global keyboard shortcuts |
//! | System Theme | Continuous | Always | Track OS theme changes |
//! | Menu (macOS) | 100ms poll | Always | Native menu bar events |
//! | Window Close | Continuous | Always | Dialog window cleanup |
//! | Toast Dismiss | 5 seconds | Toast visible | Auto-dismiss notifications |
//! | Auto-Save | 500ms poll | Study loaded + enabled | Debounced project saves |
//!
//! # Architecture
//!
//! Subscriptions are batched together in `create_subscription()` and run
//! concurrently. Conditional subscriptions (toast, auto-save) return
//! `Subscription::none()` when their condition is not met, avoiding
//! unnecessary polling.

use std::time::Duration;

use iced::Subscription;
use iced::keyboard;
use iced::window;
use iced::{system, time};

use crate::message::Message;
use crate::state::AppState;

/// Create all application subscriptions.
///
/// This batches together all event subscriptions:
/// - Keyboard events for shortcuts
/// - System theme changes for automatic theme switching
/// - Native menu events (macOS only)
/// - Window close events for dialog cleanup
/// - Toast auto-dismiss timer (conditional)
/// - Auto-save timer (conditional)
pub fn create_subscription(state: &AppState) -> Subscription<Message> {
    Subscription::batch([
        keyboard_subscription(),
        system_theme_subscription(),
        menu_subscription(),
        window_close_subscription(),
        toast_subscription(state),
        auto_save_subscription(state),
    ])
}

/// Keyboard event subscription.
///
/// Listens for all key press events to handle global shortcuts.
/// Runs continuously without polling.
fn keyboard_subscription() -> Subscription<Message> {
    keyboard::listen().map(|event| match event {
        keyboard::Event::KeyPressed { key, modifiers, .. } => Message::KeyPressed(key, modifiers),
        _ => Message::Noop,
    })
}

/// System theme change subscription.
///
/// Monitors OS theme changes (light/dark) for ThemeMode::System.
/// Runs continuously without polling.
fn system_theme_subscription() -> Subscription<Message> {
    system::theme_changes().map(Message::SystemThemeChanged)
}

/// Native menu event subscription (macOS only).
///
/// Polls the menu event channel every 100ms. This interval balances
/// responsiveness with efficiency:
/// - Menu clicks are human-initiated (~200-300ms reaction time)
/// - A background forwarder thread captures events immediately
/// - We're just polling a local mpsc channel, not the native event system
///
/// On non-macOS platforms, returns an empty subscription.
fn menu_subscription() -> Subscription<Message> {
    #[cfg(target_os = "macos")]
    {
        crate::menu::menu_subscription().map(|action| match action {
            Some(a) => Message::MenuAction(a),
            None => Message::Noop,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Subscription::none()
    }
}

/// Window close event subscription.
///
/// Listens for window close requests to clean up dialog window state.
/// Runs continuously without polling.
fn window_close_subscription() -> Subscription<Message> {
    window::close_requests().map(Message::DialogWindowClosed)
}

/// Toast auto-dismiss subscription.
///
/// When a toast notification is visible, polls every 5 seconds to
/// trigger auto-dismissal. Returns no subscription when no toast exists.
///
/// # Conditional Behavior
/// - Active: When `state.toast.is_some()`
/// - Inactive: When no toast is displayed
fn toast_subscription(state: &AppState) -> Subscription<Message> {
    if state.toast.is_some() {
        time::every(Duration::from_secs(5))
            .map(|_| Message::Toast(crate::message::ToastMessage::Dismiss))
    } else {
        Subscription::none()
    }
}

/// Auto-save subscription.
///
/// Polls every 500ms to check if an auto-save should trigger. The actual
/// save only occurs if the dirty tracker indicates changes need saving.
///
/// # Conditional Behavior
/// - Active: When auto-save is enabled AND a study is loaded
/// - Inactive: When auto-save is disabled OR no study is loaded
///
/// The 500ms interval provides a balance between:
/// - Responsiveness (saves happen within 500ms of idle threshold)
/// - Efficiency (only 2 checks per second)
fn auto_save_subscription(state: &AppState) -> Subscription<Message> {
    if state.auto_save_config.enabled && state.study.is_some() {
        time::every(Duration::from_millis(500)).map(|_| Message::AutoSaveTick)
    } else {
        Subscription::none()
    }
}

#[cfg(test)]
mod tests {
    // Note: Subscription testing requires an Iced runtime, which is not
    // available in unit tests. Integration tests should verify subscription
    // behavior through the full application.
}
