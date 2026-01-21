//! Thread-local theme context for zero-parameter color access.
//!
//! This module provides a thread-local storage pattern for theme colors,
//! eliminating the need to pass `&ThemeConfig` through the entire call hierarchy.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::theme::{set_theme, colors, is_dark};
//!
//! // Initialize in App::new() and when settings change
//! set_theme(config);
//!
//! // Access colors anywhere in the UI code
//! let c = colors();
//! let bg = c.background_primary;
//!
//! // Check dark mode
//! if is_dark() { /* dark mode styling */ }
//! ```

use std::cell::RefCell;

use super::ThemeConfig;
use super::resolved::ResolvedColors;

/// Thread-local theme context holding configuration and resolved colors.
struct ThemeContext {
    config: ThemeConfig,
    colors: ResolvedColors,
}

thread_local! {
    /// Global theme context, initialized lazily with default theme.
    static THEME_CONTEXT: RefCell<ThemeContext> = RefCell::new(ThemeContext {
        config: ThemeConfig::default(),
        colors: ResolvedColors::default(),
    });
}

/// Initialize or update the theme context.
///
/// Call this in:
/// - `App::new()` after loading settings
/// - Theme mode change handler (light/dark toggle)
/// - Accessibility mode change handler
///
/// The colors are resolved once and cached until the next call.
pub fn set_theme(config: ThemeConfig) {
    THEME_CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        ctx.config = config;
        ctx.colors = ResolvedColors::from_config(&config);
    });
}

/// Get the current resolved colors.
///
/// This is the main API for accessing theme colors throughout the UI.
/// Returns a copy of the pre-resolved colors struct.
///
/// # Panics
///
/// Will not panic - the context is always initialized with defaults.
///
/// # Example
///
/// ```rust,ignore
/// let c = colors();
/// text("Hello").color(c.text_primary)
/// ```
pub fn colors() -> ResolvedColors {
    THEME_CONTEXT.with(|ctx| ctx.borrow().colors)
}

/// Check if the current theme is in dark mode.
///
/// Useful for conditional styling that differs between light/dark modes.
pub fn is_dark() -> bool {
    THEME_CONTEXT.with(|ctx| ctx.borrow().config.is_dark())
}

/// Get the current theme configuration.
///
/// Useful when you need to create the Iced Theme or access the config directly.
pub fn current_config() -> ThemeConfig {
    THEME_CONTEXT.with(|ctx| ctx.borrow().config)
}
