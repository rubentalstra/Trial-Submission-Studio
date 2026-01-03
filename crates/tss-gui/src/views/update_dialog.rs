//! Update dialog viewport.
//!
//! Displays update status, changelog, and controls for downloading/installing updates.

use egui::{Context, Vec2, ViewportBuilder, ViewportId};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::state::{UpdatePhase, UpdateUiState};

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
    /// User clicked "Download Update".
    Download,
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
    state: &mut UpdateUiState,
    markdown_cache: &mut CommonMarkCache,
) -> UpdateDialogAction {
    if !state.open {
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
                match state.phase {
                    UpdatePhase::Idle => {
                        // Should not normally be shown in idle state
                        ui.centered_and_justified(|ui| {
                            ui.label("No update information available.");
                        });
                    }

                    UpdatePhase::Checking => {
                        ui.centered_and_justified(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.spinner();
                                ui.add_space(16.0);
                                ui.label("Checking for updates...");
                            });
                        });
                    }

                    UpdatePhase::UpdateAvailable => {
                        action = show_update_available(ui, state, markdown_cache);
                        if action != UpdateDialogAction::None {
                            should_close = action == UpdateDialogAction::SkipVersion
                                || action == UpdateDialogAction::RemindLater;
                        }
                    }

                    UpdatePhase::Downloading => {
                        show_downloading(ui, state);
                    }

                    UpdatePhase::Downloaded | UpdatePhase::ReadyToInstall => {
                        action = show_ready_to_install(ui, state);
                        if action == UpdateDialogAction::Cancel {
                            should_close = true;
                        }
                    }

                    UpdatePhase::Error => {
                        action = show_error(ui, state);
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
    state: &UpdateUiState,
    markdown_cache: &mut CommonMarkCache,
) -> UpdateDialogAction {
    let mut action = UpdateDialogAction::None;

    ui.vertical(|ui| {
        // Header
        ui.horizontal(|ui| {
            ui.heading("A new version is available!");
        });

        ui.add_space(8.0);

        // Version info
        ui.horizontal(|ui| {
            ui.label("Current version:");
            ui.strong(env!("CARGO_PKG_VERSION"));
        });

        if let Some(ref version) = state.available_version {
            ui.horizontal(|ui| {
                ui.label("New version:");
                ui.strong(version);
            });
        }

        ui.add_space(16.0);

        // Changelog
        ui.heading("What's New");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                if let Some(ref changelog) = state.changelog {
                    CommonMarkViewer::new().show(ui, markdown_cache, changelog);
                } else {
                    ui.label("No release notes available.");
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
                    .button(egui::RichText::new("Download Update").strong())
                    .clicked()
                {
                    action = UpdateDialogAction::Download;
                }
            });
        });
    });

    action
}

/// Show the downloading progress screen.
fn show_downloading(ui: &mut egui::Ui, state: &UpdateUiState) {
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);

        ui.heading("Downloading Update...");
        ui.add_space(16.0);

        // Progress bar
        let progress = state.download_progress;
        ui.add(
            egui::ProgressBar::new(progress)
                .show_percentage()
                .animate(true),
        );

        ui.add_space(8.0);

        // Speed info
        if state.download_speed > 0 {
            let speed = format_speed(state.download_speed);
            ui.label(format!("Speed: {speed}"));
        }

        ui.add_space(16.0);
        ui.label("Please wait while the update is being downloaded...");
    });
}

/// Show the "ready to install" screen.
fn show_ready_to_install(ui: &mut egui::Ui, state: &UpdateUiState) -> UpdateDialogAction {
    let mut action = UpdateDialogAction::None;

    ui.vertical_centered(|ui| {
        ui.add_space(40.0);

        ui.heading("Download Complete!");
        ui.add_space(16.0);

        if let Some(ref version) = state.available_version {
            ui.label(format!("Version {version} is ready to install."));
        }

        ui.add_space(8.0);
        ui.label("The application will restart after the update is installed.");

        ui.add_space(24.0);

        ui.horizontal(|ui| {
            if ui.button("Later").clicked() {
                action = UpdateDialogAction::Cancel;
            }

            ui.add_space(16.0);

            if ui
                .button(egui::RichText::new("Install and Restart").strong())
                .clicked()
            {
                action = UpdateDialogAction::InstallAndRestart;
            }
        });
    });

    action
}

/// Show the error screen.
fn show_error(ui: &mut egui::Ui, state: &UpdateUiState) -> UpdateDialogAction {
    let mut action = UpdateDialogAction::None;

    ui.vertical_centered(|ui| {
        ui.add_space(40.0);

        ui.heading("Update Error");
        ui.add_space(16.0);

        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
        } else {
            ui.label("An unknown error occurred while checking for updates.");
        }

        ui.add_space(24.0);

        if ui.button("Close").clicked() {
            action = UpdateDialogAction::Cancel;
        }
    });

    action
}

/// Format download speed as human-readable string.
fn format_speed(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes_per_sec >= MB {
        format!("{:.2} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.2} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

