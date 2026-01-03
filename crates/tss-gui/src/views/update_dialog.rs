//! Update dialog viewport.
//!
//! Displays update status, changelog, and controls for installing updates.

use egui::{Context, Vec2, ViewportBuilder, ViewportId};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::state::UpdateDialogState;

/// The viewport ID for the update dialog.
const UPDATE_VIEWPORT_ID: &str = "update_dialog";

/// Action returned from the update dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateDialogAction {
    /// No action.
    None,
    /// User clicked "Skip This Version".
    SkipVersion,
    /// User clicked "Remind Me Later".
    RemindLater,
    /// User clicked "Install and Restart".
    InstallAndRestart,
    /// User cancelled/closed the dialog.
    Cancel,
}

/// Show the update dialog as a viewport.
///
/// Returns an action if the user clicked a button.
pub fn show_update_dialog(
    ctx: &Context,
    state: &mut UpdateDialogState,
    markdown_cache: &mut CommonMarkCache,
) -> UpdateDialogAction {
    if !state.is_open() {
        return UpdateDialogAction::None;
    }

    let mut action = UpdateDialogAction::None;
    let mut should_close = false;

    ctx.show_viewport_immediate(
        ViewportId::from_hash_of(UPDATE_VIEWPORT_ID),
        ViewportBuilder::default()
            .with_title("Software Update")
            .with_inner_size(Vec2::new(500.0, 400.0))
            .with_min_inner_size(Vec2::new(400.0, 300.0))
            .with_resizable(true)
            .with_close_button(true),
        |ctx, _class| {
            // Handle window close button
            if ctx.input(|i| i.viewport().close_requested()) {
                should_close = true;
                action = UpdateDialogAction::Cancel;
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                match state {
                    UpdateDialogState::Closed => {
                        // Should not happen since we check is_open() above
                    }

                    UpdateDialogState::Checking => {
                        ui.centered_and_justified(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.spinner();
                                ui.add_space(16.0);
                                ui.label("Checking for updates...");
                            });
                        });
                    }

                    UpdateDialogState::NoUpdate => {
                        ui.centered_and_justified(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading("You're up to date!");
                                ui.add_space(16.0);
                                ui.label(format!(
                                    "Version {} is the latest version.",
                                    env!("CARGO_PKG_VERSION")
                                ));
                                ui.add_space(24.0);
                                if ui.button("Close").clicked() {
                                    action = UpdateDialogAction::Cancel;
                                    should_close = true;
                                }
                            });
                        });
                    }

                    UpdateDialogState::UpdateAvailable { version, changelog } => {
                        let (ver, cl) = (version.clone(), changelog.clone());
                        action = show_update_available(ui, &ver, &cl, markdown_cache);
                        if action == UpdateDialogAction::SkipVersion
                            || action == UpdateDialogAction::RemindLater
                        {
                            should_close = true;
                        }
                    }

                    UpdateDialogState::Installing => {
                        show_installing(ui);
                    }

                    UpdateDialogState::Error(error) => {
                        let err = error.clone();
                        action = show_error(ui, &err);
                        if action == UpdateDialogAction::Cancel {
                            should_close = true;
                        }
                    }
                }
            });
        },
    );

    if should_close {
        state.close();
    }

    action
}

/// Show the "update available" screen.
fn show_update_available(
    ui: &mut egui::Ui,
    version: &str,
    changelog: &str,
    markdown_cache: &mut CommonMarkCache,
) -> UpdateDialogAction {
    let mut action = UpdateDialogAction::None;

    ui.vertical(|ui| {
        // Header
        ui.heading("A new version is available!");
        ui.add_space(8.0);

        // Version info
        ui.horizontal(|ui| {
            ui.label("Current version:");
            ui.strong(env!("CARGO_PKG_VERSION"));
        });

        ui.horizontal(|ui| {
            ui.label("New version:");
            ui.strong(version.strip_prefix('v').unwrap_or(version));
        });

        ui.add_space(16.0);

        // Changelog
        ui.heading("What's New");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                if changelog.is_empty() {
                    ui.label("No release notes available.");
                } else {
                    CommonMarkViewer::new().show(ui, markdown_cache, changelog);
                }
            });

        ui.add_space(16.0);
        ui.separator();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("Skip This Version").clicked() {
                action = UpdateDialogAction::SkipVersion;
            }

            if ui.button("Remind Me Later").clicked() {
                action = UpdateDialogAction::RemindLater;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(egui::RichText::new("Install and Restart").strong())
                    .clicked()
                {
                    action = UpdateDialogAction::InstallAndRestart;
                }
            });
        });
    });

    action
}

/// Show the installing/downloading screen with indeterminate progress.
fn show_installing(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);

        ui.heading("Installing Update...");
        ui.add_space(16.0);

        // Indeterminate progress bar (animated, no percentage)
        ui.add(egui::ProgressBar::new(0.0).animate(true));

        ui.add_space(16.0);
        ui.label("Downloading and installing update...");
        ui.label("The application will restart automatically.");
    });
}

/// Show the error screen.
fn show_error(ui: &mut egui::Ui, error: &str) -> UpdateDialogAction {
    let mut action = UpdateDialogAction::None;

    ui.vertical_centered(|ui| {
        ui.add_space(40.0);

        ui.heading("Update Error");
        ui.add_space(16.0);

        ui.colored_label(egui::Color32::RED, error);

        ui.add_space(24.0);

        if ui.button("Close").clicked() {
            action = UpdateDialogAction::Cancel;
        }
    });

    action
}
