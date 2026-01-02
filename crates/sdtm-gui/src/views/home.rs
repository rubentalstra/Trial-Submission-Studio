//! Home screen view
//!
//! Study folder selection and domain discovery with DM dependency display.

use crate::state::AppState;
use crate::theme::spacing;
use egui::{Color32, RichText, Ui};
use std::path::PathBuf;

/// Home screen view
pub struct HomeView;

impl HomeView {
    /// Render the home screen
    ///
    /// Returns a folder path if the user selected one to load.
    pub fn show(ui: &mut Ui, state: &mut AppState) -> Option<PathBuf> {
        // Track which domain was clicked (if any)
        let mut clicked_domain: Option<String> = None;
        let mut go_to_export = false;
        let mut selected_folder: Option<PathBuf> = None;

        ui.vertical_centered(|ui| {
            ui.add_space(spacing::XL);

            // Title
            ui.heading(RichText::new("CDISC Transpiler").size(32.0));
            ui.add_space(spacing::SM);
            ui.label(RichText::new("Convert clinical trial data to SDTM format").weak());

            ui.add_space(spacing::XL);

            // Open study button
            if ui
                .button(
                    RichText::new(format!(
                        "{} Open Study Folder",
                        egui_phosphor::regular::FOLDER_OPEN
                    ))
                    .size(16.0),
                )
                .clicked()
            {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    tracing::info!("Selected folder: {:?}", folder);
                    selected_folder = Some(folder);
                }
            }

            ui.add_space(spacing::LG);

            // Show loaded study if any
            if let Some(study) = state.study() {
                ui.separator();
                ui.add_space(spacing::MD);

                ui.heading(&study.study_id);
                ui.label(
                    RichText::new(study.study_folder.display().to_string())
                        .weak()
                        .small(),
                );

                ui.add_space(spacing::MD);

                // DM dependency notice if DM exists but not ready
                if study.has_dm_domain() && !study.is_dm_ready() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} Complete DM domain first to unlock other domains",
                                egui_phosphor::regular::INFO
                            ))
                            .color(ui.visuals().warn_fg_color),
                        );
                    });
                    ui.add_space(spacing::SM);
                }

                // Domain list
                ui.label(
                    RichText::new(format!(
                        "{} Discovered Domains",
                        egui_phosphor::regular::DATABASE
                    ))
                    .strong(),
                );
                ui.add_space(spacing::SM);

                // Get domain codes with DM first
                let domain_codes = study.domain_codes_dm_first();

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for code in domain_codes {
                            // Get domain info (bypassing DM check since we're just displaying)
                            let Some(domain) = study.get_domain(code) else {
                                continue;
                            };

                            let display_name = domain.display_name(code);
                            let row_count = domain.row_count();
                            let is_accessible = state.is_domain_accessible(code);
                            let lock_reason = state.domain_lock_reason(code);
                            let is_mapping_complete = domain.is_mapping_complete();

                            // Determine status icon and color
                            let (status_icon, status_color) = if !is_accessible {
                                // Locked - requires DM
                                (egui_phosphor::regular::LOCK, ui.visuals().weak_text_color())
                            } else if is_mapping_complete {
                                // Mapping complete
                                (egui_phosphor::regular::CHECK_CIRCLE, Color32::GREEN)
                            } else {
                                // In progress
                                (
                                    egui_phosphor::regular::CIRCLE_DASHED,
                                    ui.visuals().warn_fg_color,
                                )
                            };

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(status_icon).color(status_color));

                                let button =
                                    ui.add_enabled(is_accessible, egui::Button::new(&display_name));

                                if button.clicked() && is_accessible {
                                    clicked_domain = Some(code.to_string());
                                }

                                // Show tooltip for locked domains
                                if let Some(reason) = lock_reason {
                                    button.on_hover_text(reason);
                                }

                                ui.label(
                                    RichText::new(format!("{} rows", row_count)).weak().small(),
                                );

                                // Show lock badge for non-DM domains when DM not ready
                                if !is_accessible {
                                    ui.label(
                                        RichText::new("Requires DM")
                                            .small()
                                            .color(ui.visuals().warn_fg_color),
                                    );
                                }
                            });
                        }
                    });

                ui.add_space(spacing::MD);

                // Export button
                if ui
                    .button(format!("{} Go to Export", egui_phosphor::regular::EXPORT))
                    .clicked()
                {
                    go_to_export = true;
                }
            }

            // Recent studies
            if !state.settings.recent_studies.is_empty() && state.study.is_none() {
                ui.add_space(spacing::XL);
                ui.separator();
                ui.add_space(spacing::MD);

                ui.label(
                    RichText::new(format!(
                        "{} Recent Studies",
                        egui_phosphor::regular::CLOCK_COUNTER_CLOCKWISE
                    ))
                    .strong(),
                );
                ui.add_space(spacing::SM);

                let recent_paths: Vec<_> = state.settings.recent_studies.clone();
                for path in recent_paths {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if ui
                            .button(format!("{} {}", egui_phosphor::regular::FOLDER, name))
                            .clicked()
                        {
                            selected_folder = Some(path);
                        }
                    }
                }
            }
        });

        // Handle navigation after borrowing ends
        if let Some(domain) = clicked_domain {
            state.open_domain(domain);
        }
        if go_to_export {
            state.go_export();
        }

        selected_folder
    }
}
