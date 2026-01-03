//! About dialog viewport.
//!
//! Displays application information, version, and links.

use chrono::Datelike;
use egui::{Context, RichText, Vec2, ViewportBuilder, ViewportId};

use crate::state::AboutUiState;

/// The viewport ID for the about dialog.
const ABOUT_VIEWPORT_ID: &str = "about_dialog";

/// Show the about dialog as a viewport.
pub fn show_about_dialog(ctx: &Context, state: &mut AboutUiState) {
    if !state.open {
        return;
    }

    let mut should_close = false;

    ctx.show_viewport_immediate(
        ViewportId::from_hash_of(ABOUT_VIEWPORT_ID),
        ViewportBuilder::default()
            .with_title("About Trial Submission Studio")
            .with_inner_size(Vec2::new(400.0, 300.0))
            .with_min_inner_size(Vec2::new(350.0, 250.0))
            .with_resizable(false)
            .with_close_button(true),
        |ctx, _class| {
            // Handle window close button
            if ctx.input(|i| i.viewport().close_requested()) {
                should_close = true;
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    // App name
                    ui.heading(RichText::new("Trial Submission Studio").size(24.0).strong());

                    ui.add_space(8.0);

                    // TSS abbreviation with styling
                    ui.label(RichText::new("TSS").size(18.0).color(egui::Color32::from_rgb(40, 79, 119)));

                    ui.add_space(16.0);

                    // Version
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));

                    ui.add_space(8.0);

                    // Description
                    ui.label("A professional CDISC data transformation tool");
                    ui.label("for clinical trial submissions.");

                    ui.add_space(20.0);

                    ui.separator();

                    ui.add_space(12.0);

                    // Copyright
                    ui.label(format!("Copyright {} Ruben Talstra", current_year()));

                    ui.add_space(8.0);

                    // License
                    ui.label("Licensed under the MIT License");

                    ui.add_space(16.0);

                    // Links
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 16.0;

                        if ui.link("GitHub").clicked() {
                            let _ = open::that("https://github.com/rubentalstra/Trial-Submission-Studio");
                        }

                        if ui.link("Documentation").clicked() {
                            let _ = open::that("https://github.com/rubentalstra/Trial-Submission-Studio#readme");
                        }

                        if ui.link("Report Issue").clicked() {
                            let _ = open::that("https://github.com/rubentalstra/Trial-Submission-Studio/issues");
                        }
                    });

                    ui.add_space(20.0);

                    // Close button
                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            });
        },
    );

    if should_close {
        state.close();
    }
}

/// Get the current year for copyright.
fn current_year() -> i32 {
    chrono::Utc::now().year()
}
