//! Export screen view
//!
//! Configure and execute export to various formats.

use crate::state::{AppState, DomainStatus};
use crate::theme::{colors, spacing};
use egui::{RichText, Ui};

/// Export view
pub struct ExportView;

impl ExportView {
    /// Render the export screen
    pub fn show(ui: &mut Ui, state: &mut AppState) {
        let theme = colors(state.preferences.dark_mode);

        // Top bar
        ui.horizontal(|ui| {
            if ui.button(format!("{} Back", egui_phosphor::regular::ARROW_LEFT)).clicked() {
                state.go_home();
            }
            ui.separator();
            ui.heading(format!("{} Export", egui_phosphor::regular::EXPORT));
        });

        ui.add_space(spacing::MD);

        if let Some(study) = &state.study {
            // Domain selection
            ui.label(RichText::new(format!("{} Select Domains to Export", egui_phosphor::regular::LIST_CHECKS)).strong());
            ui.add_space(spacing::SM);

            let ready_domains: Vec<_> = study
                .domain_codes()
                .into_iter()
                .filter(|code| {
                    study
                        .get_domain(code)
                        .is_some_and(|d| d.status == DomainStatus::ReadyForExport)
                })
                .collect();

            if ready_domains.is_empty() {
                ui.label(
                    RichText::new("No domains ready for export. Complete mapping and validation first.")
                        .color(theme.text_muted),
                );
            } else {
                for code in &ready_domains {
                    ui.checkbox(&mut true, *code);
                }
            }

            ui.add_space(spacing::LG);

            // Output format selection
            ui.label(RichText::new(format!("{} Output Format", egui_phosphor::regular::FILE)).strong());
            ui.add_space(spacing::SM);

            ui.horizontal(|ui| {
                ui.radio_value(&mut true, true, "XPT (SAS Transport)");
            });
            ui.horizontal(|ui| {
                ui.radio_value(&mut false, true, "Dataset-XML");
            });
            ui.horizontal(|ui| {
                ui.radio_value(&mut false, true, "Define-XML");
            });

            ui.add_space(spacing::LG);

            // Output directory
            ui.label(RichText::new(format!("{} Output Directory", egui_phosphor::regular::FOLDER)).strong());
            ui.add_space(spacing::SM);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(study.study_folder.join("output").display().to_string())
                        .color(theme.text_secondary),
                );
                if ui.button(format!("{} Change...", egui_phosphor::regular::FOLDER_OPEN)).clicked() {
                    if let Some(_folder) = rfd::FileDialog::new().pick_folder() {
                        // TODO: Set output directory
                    }
                }
            });

            ui.add_space(spacing::XL);

            // Export button
            let can_export = !ready_domains.is_empty();
            ui.add_enabled_ui(can_export, |ui| {
                if ui
                    .button(RichText::new(format!("{} Export Selected Domains", egui_phosphor::regular::EXPORT)).size(16.0))
                    .clicked()
                {
                    // TODO: Trigger export
                    tracing::info!("Starting export...");
                }
            });
        } else {
            ui.label(
                RichText::new("No study loaded. Go back and open a study folder first.")
                    .color(theme.text_muted),
            );
        }
    }
}
