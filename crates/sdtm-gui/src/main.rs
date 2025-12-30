//! CDISC SDTM Transpiler - Desktop GUI Application
//!
//! A desktop application for converting clinical trial source data into
//! CDISC SDTM formats (XPT, Dataset-XML, Define-XML).

mod app;
mod state;
mod theme;
mod views;

use eframe::egui;

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("CDISC Transpiler")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1024.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "CDISC Transpiler",
        options,
        Box::new(|cc| Ok(Box::new(app::CdiscApp::new(cc)))),
    )
}
