//! Trial Submission Studio - Desktop GUI Application
//!
//! A desktop application for converting clinical trial source data into
//! CDISC SDTM formats (XPT, Dataset-XML, Define-XML).

mod app;
mod export;
mod menu;
mod services;
mod settings;
mod state;
mod theme;
mod views;

use eframe::egui;

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create native menu bar
    // Note: On macOS, we must disable eframe's default menu to use our custom muda menu
    let menu = menu::create_menu();
    let menu_receiver = menu::menu_event_receiver();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Trial Submission Studio")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1024.0, 600.0]),
        #[cfg(target_os = "macos")]
        event_loop_builder: Some(Box::new(|builder| {
            use winit::platform::macos::EventLoopBuilderExtMacOS;
            builder.with_default_menu(false);
        })),
        #[cfg(not(target_os = "macos"))]
        event_loop_builder: None,
        ..Default::default()
    };

    eframe::run_native(
        "Trial Submission Studio",
        options,
        Box::new(move |cc| {
            // Initialize the menu for macOS NSApp now that the event loop is running
            menu::init_menu_for_nsapp(&menu);

            // Create app, passing menu ownership to keep it alive
            Ok(Box::new(app::CdiscApp::new(cc, menu_receiver, menu)))
        }),
    )
}
