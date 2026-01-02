//! Export screen view
//!
//! Clean two-column layout: domain selection (left) and configuration (right).

use crate::settings::ExportFormat;
use crate::state::AppState;
use crate::theme::spacing;
use egui::{Color32, RichText, Ui};

/// Export view
pub struct ExportView;

impl ExportView {
    /// Render the export screen
    pub fn show(ui: &mut Ui, state: &mut AppState) {
        // Check if export is in progress
        if let Some(ref progress) = state.ui.export.progress {
            if progress.is_in_progress() {
                show_export_progress(ui, state);
                return;
            }
        }

        if state.study.is_some() {
            show_export_layout(ui, state);
        } else {
            show_no_study(ui, state);
        }
    }
}

/// Show message when no study is loaded
fn show_no_study(ui: &mut Ui, state: &mut AppState) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 3.0);
        ui.label(
            RichText::new(egui_phosphor::regular::FOLDER_NOTCH_OPEN)
                .size(48.0)
                .weak(),
        );
        ui.add_space(spacing::MD);
        ui.label(RichText::new("No Study Loaded").size(20.0).strong());
        ui.add_space(spacing::SM);
        ui.label(RichText::new("Open a study folder to export domains").weak());
        ui.add_space(spacing::LG);
        if ui
            .button(format!("{} Go Back", egui_phosphor::regular::ARROW_LEFT))
            .clicked()
        {
            state.go_home();
        }
    });
}

/// Show the two-column export layout
fn show_export_layout(ui: &mut Ui, state: &mut AppState) {
    // Header
    ui.horizontal(|ui| {
        if ui
            .button(format!("{} Back", egui_phosphor::regular::ARROW_LEFT))
            .clicked()
        {
            state.go_home();
        }
        ui.separator();
        ui.heading("Export");
    });

    ui.add_space(spacing::MD);
    ui.separator();
    ui.add_space(spacing::MD);

    // Two-column layout
    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::relative(0.4).at_least(200.0)) // Left column
        .size(egui_extras::Size::exact(1.0)) // Separator
        .size(egui_extras::Size::remainder()) // Right column
        .horizontal(|mut strip| {
            // Left: Domain selection
            strip.cell(|ui| {
                show_domain_selection(ui, state);
            });

            // Separator
            strip.cell(|ui| {
                ui.separator();
            });

            // Right: Configuration
            strip.cell(|ui| {
                show_export_config(ui, state);
            });
        });
}

/// Show domain selection list
fn show_domain_selection(ui: &mut Ui, state: &mut AppState) {
    ui.label(
        RichText::new(format!(
            "{} Select Domains",
            egui_phosphor::regular::DATABASE
        ))
        .strong(),
    );
    ui.add_space(spacing::SM);

    // Extract domain info before mutable borrows
    // Tuple: (is_complete, is_touched, row_count)
    let domain_info: Vec<_> = {
        let Some(study) = state.study() else {
            return;
        };

        study
            .domain_codes()
            .iter()
            .map(|code| {
                let code = code.to_string();
                let info = study
                    .get_domain(&code)
                    .map(|d| (d.is_mapping_complete(), d.is_touched(), d.row_count()));
                (code, info)
            })
            .collect()
    };

    let domain_codes: Vec<String> = domain_info.iter().map(|(c, _)| c.clone()).collect();

    // Select all / deselect all
    ui.horizontal(|ui| {
        if ui.small_button("Select All").clicked() {
            state.ui.export.select_all(domain_codes.iter());
        }
        if ui.small_button("Deselect All").clicked() {
            state.ui.export.deselect_all();
        }
    });

    ui.add_space(spacing::SM);
    ui.separator();
    ui.add_space(spacing::SM);

    // Track which domain to toggle (if any)
    let mut toggle_domain: Option<String> = None;

    // Domain list
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (code, info) in &domain_info {
            let is_selected = state.ui.export.selected_domains.contains(code);
            let is_accessible = state.is_domain_accessible(code);

            ui.horizontal(|ui| {
                // Checkbox (disabled if not accessible)
                let mut checked = is_selected;
                if ui
                    .add_enabled(is_accessible, egui::Checkbox::without_text(&mut checked))
                    .changed()
                {
                    toggle_domain = Some(code.clone());
                }

                // Domain code
                let code_text = if is_accessible {
                    RichText::new(code)
                } else {
                    RichText::new(code).weak()
                };
                ui.label(code_text);

                // Status icon (4 states: locked, ready, in progress, complete)
                if let Some((is_complete, is_touched, row_count)) = info {
                    if !is_accessible {
                        // State 1: Locked
                        ui.label(
                            RichText::new(egui_phosphor::regular::LOCK)
                                .color(ui.visuals().weak_text_color()),
                        );
                    } else if *is_complete {
                        // State 2: Complete
                        ui.label(
                            RichText::new(egui_phosphor::regular::CHECK_CIRCLE)
                                .color(Color32::GREEN),
                        );
                    } else if *is_touched {
                        // State 3: In Progress
                        ui.label(
                            RichText::new(egui_phosphor::regular::PENCIL)
                                .color(ui.visuals().warn_fg_color),
                        );
                    } else {
                        // State 4: Ready (unlocked but not started)
                        ui.label(
                            RichText::new(egui_phosphor::regular::CIRCLE)
                                .color(ui.visuals().text_color()),
                        );
                    }

                    ui.label(RichText::new(format!("{} rows", row_count)).weak().small());
                }
            });
        }
    });

    // Apply toggle after ScrollArea
    if let Some(code) = toggle_domain {
        state.ui.export.toggle_domain(&code);
    }
}

/// Show export configuration panel
fn show_export_config(ui: &mut Ui, state: &mut AppState) {
    ui.label(RichText::new(format!("{} Export Settings", egui_phosphor::regular::GEAR)).strong());
    ui.add_space(spacing::MD);

    // Output directory
    ui.label(RichText::new("Output Directory").weak());
    let output_dir = state
        .ui
        .export
        .output_dir_override
        .clone()
        .or_else(|| state.study().map(|s| s.study_folder.join("export")));

    if let Some(dir) = &output_dir {
        ui.horizontal(|ui| {
            ui.label(RichText::new(dir.display().to_string()).monospace().small());
            if ui.small_button("Change").clicked() {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    state.ui.export.output_dir_override = Some(folder);
                }
            }
        });
    }

    ui.add_space(spacing::MD);

    // Format selection
    ui.label(RichText::new("Export Format").weak());
    let current_format = state
        .ui
        .export
        .format_override
        .unwrap_or(state.settings.export.default_format);

    let formats = [
        (ExportFormat::Xpt, "XPT (SAS Transport v5)"),
        (ExportFormat::DatasetXml, "Dataset-XML (CDISC)"),
    ];

    for (format, label) in formats {
        if ui
            .selectable_label(current_format == format, label)
            .clicked()
        {
            state.ui.export.format_override = Some(format);
        }
    }

    ui.add_space(spacing::LG);
    ui.separator();
    ui.add_space(spacing::MD);

    // Export button
    let selected_count = state.ui.export.selection_count();
    let can_export = selected_count > 0;

    ui.add_enabled_ui(can_export, |ui| {
        if ui
            .button(
                RichText::new(format!(
                    "{} Export {} Domain{}",
                    egui_phosphor::regular::EXPORT,
                    selected_count,
                    if selected_count == 1 { "" } else { "s" }
                ))
                .size(16.0),
            )
            .clicked()
        {
            // TODO: Start export
            tracing::info!("Export clicked with {} domains", selected_count);
        }
    });

    if !can_export {
        ui.label(
            RichText::new("Select at least one domain to export")
                .weak()
                .small(),
        );
    }
}

/// Show export progress
fn show_export_progress(ui: &mut Ui, state: &mut AppState) {
    // Extract progress info before rendering
    let (completed, error, current_step, fraction) = {
        let Some(ref progress) = state.ui.export.progress else {
            return;
        };
        (
            progress.completed,
            progress.error.clone(),
            progress.current_step.clone(),
            progress.fraction(),
        )
    };

    let mut close_clicked = false;

    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 3.0);

        if completed {
            // Show success
            ui.label(
                RichText::new(egui_phosphor::regular::CHECK_CIRCLE)
                    .size(48.0)
                    .color(Color32::GREEN),
            );
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Export Complete").size(20.0).strong());
        } else if let Some(ref err) = error {
            // Show error
            ui.label(
                RichText::new(egui_phosphor::regular::WARNING)
                    .size(48.0)
                    .color(ui.visuals().error_fg_color),
            );
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Export Failed").size(20.0).strong());
            ui.add_space(spacing::SM);
            ui.label(RichText::new(err).color(ui.visuals().error_fg_color));
        } else {
            // Show progress
            ui.spinner();
            ui.add_space(spacing::MD);
            ui.label(RichText::new(&current_step).size(16.0));
            ui.add_space(spacing::SM);
            ui.add(egui::ProgressBar::new(fraction));
        }

        ui.add_space(spacing::LG);
        if ui.button("Close").clicked() {
            close_clicked = true;
        }
    });

    // Apply state change after rendering
    if close_clicked {
        state.ui.export.progress = None;
    }
}
