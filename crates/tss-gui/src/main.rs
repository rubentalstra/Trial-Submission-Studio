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
use iced::Size;
use iced::window;

// Import Lucide font bytes for loading
use component::LUCIDE_FONT_BYTES;

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

    // Note: Native menu initialization for macOS happens in App::new()
    // after the Iced runtime has started

    // Run the Iced application using the builder pattern
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .font(LUCIDE_FONT_BYTES)
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
