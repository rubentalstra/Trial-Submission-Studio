//! About dialog viewport.
//!
//! Displays application information, version, build info, and links.
//! Styled similar to RustRover/IntelliJ IDEs.

use chrono::Datelike;
use egui::{Context, RichText, Vec2, ViewportBuilder, ViewportId};

use crate::state::AboutUiState;

/// The viewport ID for the about dialog.
const ABOUT_VIEWPORT_ID: &str = "about_dialog";

/// Application version from Cargo.toml.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Rust version captured at build time.
const RUST_VERSION: &str = env!("RUST_VERSION");

/// Build target triple captured at build time.
const BUILD_TARGET: &str = env!("BUILD_TARGET");

/// Build date captured at build time.
const BUILD_DATE: &str = env!("BUILD_DATE");

/// Build number captured at build time (TSS-X.Y for CI, LOCAL.Y for local).
const BUILD_NUMBER: &str = env!("BUILD_NUMBER");

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
            .with_inner_size(Vec2::new(480.0, 250.0))
            .with_min_inner_size(Vec2::new(480.0, 250.0))
            .with_resizable(false)
            .with_close_button(true),
        |ctx, _class| {
            // Handle window close button
            if ctx.input(|i| i.viewport().close_requested()) {
                should_close = true;
            }

            // Bottom panel for buttons (renders first, takes space from bottom)
            egui::TopBottomPanel::bottom("about_buttons")
                .show_separator_line(false)
                .show(ctx, |ui| {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                should_close = true;
                            }
                            if ui.button("Copy and Close").clicked() {
                                let info = format!(
                                    "Trial Submission Studio {VERSION}\n\
                                     Build: {BUILD_NUMBER} ({BUILD_DATE})\n\
                                     Runtime: {RUST_VERSION}\n\
                                     Target: {BUILD_TARGET}"
                                );
                                ui.ctx().copy_text(info);
                                should_close = true;
                            }
                        });
                    });
                    ui.add_space(6.0);
                });

            // Main content panel
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    ui.add_space(16.0);

                    // App icon
                    ui.add(
                        egui::Image::new(egui::include_image!("../../assets/icon.svg"))
                            .fit_to_exact_size(Vec2::splat(80.0)),
                    );

                    ui.add_space(16.0);

                    // All info in one vertical column, left-aligned
                    ui.vertical(|ui| {
                        // App name (large, bold)
                        ui.label(RichText::new("Trial Submission Studio").size(18.0).strong());

                        ui.add_space(4.0);

                        // Version + Build on same conceptual level
                        ui.label(format!("Version {VERSION}"));
                        ui.label(
                            RichText::new(format!("Build {BUILD_NUMBER} ({BUILD_DATE})"))
                                .size(12.0)
                                .color(ui.visuals().weak_text_color()),
                        );

                        ui.add_space(8.0);

                        // Runtime info
                        ui.label(
                            RichText::new(format!("Runtime: {RUST_VERSION}"))
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                        ui.label(
                            RichText::new(BUILD_TARGET)
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );

                        ui.add_space(8.0);

                        // Links and copyright - left aligned
                        if ui
                            .link(RichText::new("View on GitHub").size(12.0))
                            .clicked()
                        {
                            let _ = open::that(
                                "https://github.com/rubentalstra/Trial-Submission-Studio",
                            );
                        }

                        ui.add_space(4.0);

                        ui.label(
                            RichText::new(format!(
                                "Copyright © 2024–{} Ruben Talstra",
                                current_year()
                            ))
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                        );
                        ui.label(
                            RichText::new("Licensed under the MIT License")
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
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
