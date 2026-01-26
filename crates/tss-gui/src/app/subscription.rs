//! Application subscriptions.
//!
//! This module centralizes all Iced subscriptions for the application.
//! Subscriptions are reactive event sources that run alongside the app.
//!
//! # Subscription Overview
//!
//! | Subscription | Type | Condition | Purpose |
//! |--------------|------|-----------|---------|
//! | Keyboard | Event-driven | Always | Global keyboard shortcuts |
//! | System Theme | Event-driven | Always | Track OS theme changes |
//! | Menu (macOS) | 100ms poll | Always | Native menu bar events |
//! | Window Close | Event-driven | Always | Dialog window cleanup |
//! | Auto-Save | Event-driven | Study loaded + enabled | Debounced project saves |
//!
//! # Event-Driven Patterns (Issue #187, #193)
//!
//! Toast notifications and auto-save use one-shot Task::perform timers instead
//! of polling subscriptions:
//! - Toast: 5-second timer started when toast is shown
//! - Auto-save: 2-second debounce timer started when changes are made
//!
//! # Architecture
//!
//! Subscriptions are batched together in `create_subscription()` and run
//! concurrently.

use iced::keyboard;
use iced::window;
use iced::{Subscription, system};

use crate::message::Message;
use crate::state::AppState;

/// Create all application subscriptions.
///
/// This batches together all event subscriptions:
/// - Keyboard events for shortcuts
/// - System theme changes for automatic theme switching
/// - Native menu events (macOS only)
/// - Window close events for dialog cleanup
///
/// Note: Toast auto-dismiss and auto-save use Task::perform instead of
/// polling subscriptions (see #187, #193).
pub fn create_subscription(_state: &AppState) -> Subscription<Message> {
    Subscription::batch([
        keyboard_subscription(),
        system_theme_subscription(),
        menu_subscription(),
        window_close_subscription(),
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

#[cfg(test)]
mod tests {
    // Note: Subscription testing requires an Iced runtime, which is not
    // available in unit tests. Integration tests should verify subscription
    // behavior through the full application.
}
