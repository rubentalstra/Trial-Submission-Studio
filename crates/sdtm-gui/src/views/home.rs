//! Home screen view
//!
//! Study folder selection and domain discovery.

use crate::state::{AppState, DomainStatus};
use crate::theme::{colors, spacing};
use egui::{RichText, Ui};
use std::path::PathBuf;

/// Home screen view
pub struct HomeView;

impl HomeView {
    /// Render the home screen
    ///
    /// Returns a folder path if the user selected one to load.
    pub fn show(ui: &mut Ui, state: &mut AppState) -> Option<PathBuf> {
        let theme = colors(state.preferences.dark_mode);

        // Track which domain was clicked (if any)
        let mut clicked_domain: Option<String> = None;
        let mut go_to_export = false;
        let mut selected_folder: Option<PathBuf> = None;

        ui.vertical_centered(|ui| {
            ui.add_space(spacing::XL);

            // Title
            ui.heading(RichText::new("CDISC Transpiler").size(32.0));
            ui.add_space(spacing::SM);
            ui.label(
                RichText::new("Convert clinical trial data to SDTM format")
                    .color(theme.text_secondary),
            );

            ui.add_space(spacing::XL);

            // Open study button
            if ui
                .button(RichText::new("Open Study Folder").size(16.0))
                .clicked()
            {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    tracing::info!("Selected folder: {:?}", folder);
                    selected_folder = Some(folder);
                }
            }

            ui.add_space(spacing::LG);

            // Show loaded study if any
            if let Some(study) = &state.study {
                ui.separator();
                ui.add_space(spacing::MD);

                ui.heading(&study.study_id);
                ui.label(
                    RichText::new(study.study_folder.display().to_string())
                        .color(theme.text_muted)
                        .small(),
                );

                ui.add_space(spacing::MD);

                // Domain list
                ui.label(RichText::new("Discovered Domains").strong());
                ui.add_space(spacing::SM);

                // Collect domain info for rendering
                let domain_info: Vec<_> = study
                    .domain_codes()
                    .into_iter()
                    .filter_map(|code| {
                        study.domains.get(code).map(|domain| {
                            (
                                code.to_string(),
                                domain.status,
                                domain.row_count(),
                            )
                        })
                    })
                    .collect();

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for (code, status, row_count) in &domain_info {
                            let status_icon = status.icon();
                            let status_color = match status {
                                DomainStatus::NotStarted => theme.text_muted,
                                DomainStatus::MappingInProgress => theme.warning,
                                DomainStatus::MappingComplete => theme.accent,
                                DomainStatus::ValidationFailed => theme.error,
                                DomainStatus::ReadyForExport => theme.success,
                            };

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(status_icon).color(status_color));
                                if ui.button(code).clicked() {
                                    clicked_domain = Some(code.clone());
                                }
                                ui.label(
                                    RichText::new(format!("{} rows", row_count))
                                        .color(theme.text_muted)
                                        .small(),
                                );
                            });
                        }
                    });

                ui.add_space(spacing::MD);

                // Export button
                if ui.button("Go to Export").clicked() {
                    go_to_export = true;
                }
            }

            // Recent studies
            if !state.preferences.recent_studies.is_empty() && state.study.is_none() {
                ui.add_space(spacing::XL);
                ui.separator();
                ui.add_space(spacing::MD);

                ui.label(RichText::new("Recent Studies").strong());
                ui.add_space(spacing::SM);

                let recent_paths: Vec<_> = state.preferences.recent_studies.clone();
                for path in recent_paths {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if ui.button(name).clicked() {
                            // TODO: Load recent study
                            tracing::info!("Opening recent study: {:?}", path);
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
