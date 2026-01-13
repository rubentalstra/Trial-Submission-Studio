//! Trial Submission Studio - Desktop GUI Application
//!
//! A desktop application for converting clinical trial source data into
//! CDISC SDTM formats (XPT, Dataset-XML, Define-XML).
//!
//! Built with Iced 0.14.0 using the Elm architecture (State, Message, Update, View).

// Module declarations
mod app;
mod component;
mod message;
mod service;
mod state;
mod theme;
mod view;

// These modules will be uncommented as they're ported:
// mod menu;
// mod settings;

use app::App;
use iced::Size;
use iced::window;

/// Application entry point.
///
/// Initializes the Iced application with the Professional Clinical theme
/// and default window settings.
pub fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting Trial Submission Studio");

    // Run the Iced application using the builder pattern
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window(window::Settings {
            size: Size::new(1280.0, 800.0),
            min_size: Some(Size::new(1024.0, 600.0)),
            icon: load_icon(),
            ..Default::default()
        })
        .run()
}

/// Load the application icon from embedded PNG data.
fn load_icon() -> Option<window::Icon> {
    let icon_data = include_bytes!("../assets/icon.png");
    // Use Iced 0.14.0 API: from_file_data takes raw bytes and optional format
    window::icon::from_file_data(icon_data, Some(image::ImageFormat::Png)).ok()
}
