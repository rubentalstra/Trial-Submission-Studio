//! Keyboard handlers as impl App methods.
//!
//! Keyboard handling remains as `impl App` since it's tightly coupled
//! with the App's keyboard shortcut state.
//!
//! All other handlers have been moved to the top-level `handler/` module
//! and implement the `MessageHandler` trait.

mod keyboard;

// Keyboard handler is implemented as `impl App` block.
