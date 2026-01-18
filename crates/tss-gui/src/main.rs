//! Trial Submission Studio - Desktop GUI Application
//!
//! A desktop application for converting clinical trial source data into
//! CDISC SDTM formats (XPT, Dataset-XML, Define-XML).
//!
//! Built with Iced 0.14.0 using the Elm architecture (State, Message, Update, View).

// Module declarations
mod app;
mod component;
mod menu;
mod message;
mod service;
mod state;
mod theme;
mod view;

use app::App;

// Import Lucide font bytes for loading
use component::LUCIDE_FONT_BYTES;

/// Application entry point.
///
/// Initializes the Iced application with the Professional Clinical theme
/// and default window settings.
///
/// Uses `daemon()` builder for multi-window support (dialog windows).
pub fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting Trial Submission Studio");

    // Run the Iced application using daemon builder for multi-window support
    // daemon() allows multiple windows with window::Id-based view/title
    iced::daemon(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .font(LUCIDE_FONT_BYTES)
        .run()
}
