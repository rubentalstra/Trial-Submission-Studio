//! Export screen view
//!
//! Clean two-column layout: domain selection (left) and configuration (right).

use crate::settings::ExportFormat;
use crate::state::{AppState, DomainStatus, ExportDomainStep, ExportProgress};
use crate::theme::spacing;
use egui::{Color32, RichText, Ui};

/// Export view
pub struct ExportView;

impl ExportView {
    /// Render the export screen
    pub fn show(ui: &mut Ui, state: &mut AppState) {
        // Check if export is in progress
        if let Some(progress) = &state.export_state.progress {
            show_export_progress(ui, state, progress.clone());
            return;
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

/// Main two-column export layout
fn show_export_layout(ui: &mut Ui, state: &mut AppState) {
    let mut start_export = false;

    // Get study data
    let (study_folder, study_id, all_domains) = {
        let study = state.study.as_ref().unwrap();
        let domains: Vec<_> = study
            .domain_codes()
            .into_iter()
            .map(|code| {
                let domain = study.get_domain(code).unwrap();
                let display_name = domain.display_name(code);
                (
                    code.to_string(),
                    display_name,
                    domain.status,
                    domain.row_count(),
                    domain.validation.as_ref().map(|v| v.error_count(None)),
                )
            })
            .collect();
        (study.study_folder.clone(), study.study_id.clone(), domains)
    };

    let selected_count = state.export_state.selected_domains.len();
    let ready_count = all_domains
        .iter()
        .filter(|(_, _, status, _, _)| {
            can_export_domain(*status, state.settings.developer.allow_export_with_errors)
        })
        .count();

    let output_dir = state
        .export_state
        .output_dir_override
        .clone()
        .or_else(|| state.settings.export.default_output_dir.clone())
        .unwrap_or_else(|| study_folder.join("output"));

    let current_format = state
        .export_state
        .format_override
        .unwrap_or(state.settings.export.default_format);

    let available_height = ui.available_height();

    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(280.0))
        .size(egui_extras::Size::exact(1.0))
        .size(egui_extras::Size::remainder())
        .horizontal(|mut strip| {
            // =================================================================
            // LEFT: Domain Selection
            // =================================================================
            strip.cell(|ui| {
                // Header with back button
                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new(egui_phosphor::regular::ARROW_LEFT))
                        .clicked()
                    {
                        state.export_state.progress = None;
                        state.go_home();
                    }
                    ui.heading("Export");
                });

                ui.add_space(spacing::MD);

                // Domain list header
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Domains").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("None").clicked() {
                            state.export_state.selected_domains.clear();
                        }
                        if ui.small_button("All").clicked() {
                            for (code, _, status, _, _) in &all_domains {
                                if can_export_domain(
                                    *status,
                                    state.settings.developer.allow_export_with_errors,
                                ) {
                                    state.export_state.selected_domains.insert(code.clone());
                                }
                            }
                        }
                    });
                });

                ui.label(
                    RichText::new(format!("{} of {} selected", selected_count, ready_count))
                        .weak()
                        .small(),
                );
                ui.add_space(spacing::SM);

                // Domain list
                egui::ScrollArea::vertical()
                    .max_height(available_height - 120.0)
                    .show(ui, |ui| {
                        for (code, display_name, status, row_count, error_count) in &all_domains {
                            show_domain_row(
                                ui,
                                state,
                                code,
                                display_name,
                                *status,
                                *row_count,
                                *error_count,
                            );
                        }
                    });
            });

            strip.cell(|ui| {
                ui.separator();
            });

            // =================================================================
            // RIGHT: Configuration
            // =================================================================
            strip.cell(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Study Info Header
                    ui.add_space(spacing::SM);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(egui_phosphor::regular::FLASK)
                                .size(20.0)
                                .color(ui.visuals().hyperlink_color),
                        );
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&study_id).size(18.0).strong());
                            ui.label(
                                RichText::new(study_folder.display().to_string())
                                    .weak()
                                    .small(),
                            );
                        });
                    });

                    ui.add_space(spacing::LG);
                    ui.separator();
                    ui.add_space(spacing::MD);

                    // Format Selection
                    ui.label(RichText::new("Output Format").strong());
                    ui.add_space(spacing::SM);

                    for format in ExportFormat::all() {
                        let is_selected = current_format == *format;
                        ui.horizontal(|ui| {
                            if ui.radio(is_selected, "").clicked() {
                                state.export_state.format_override = Some(*format);
                            }
                            ui.vertical(|ui| {
                                ui.label(RichText::new(format.display_name()).strong());
                                ui.label(RichText::new(format.description()).weak().small());
                            });
                        });
                        ui.add_space(spacing::XS);
                    }

                    ui.add_space(spacing::SM);

                    let mut generate_define = state.settings.export.generate_define_xml;
                    if ui
                        .checkbox(&mut generate_define, "Include Define-XML metadata")
                        .changed()
                    {
                        state.settings.export.generate_define_xml = generate_define;
                    }

                    ui.add_space(spacing::LG);
                    ui.separator();
                    ui.add_space(spacing::MD);

                    // Output Location
                    ui.label(RichText::new("Output Location").strong());
                    ui.add_space(spacing::SM);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(egui_phosphor::regular::FOLDER)
                                .color(ui.visuals().hyperlink_color),
                        );
                        ui.label(
                            RichText::new(output_dir.display().to_string())
                                .monospace()
                                .small(),
                        );
                    });

                    ui.add_space(spacing::SM);

                    ui.horizontal(|ui| {
                        if ui
                            .button(format!("{} Browse", egui_phosphor::regular::FOLDER_OPEN))
                            .clicked()
                        {
                            if let Some(folder) = rfd::FileDialog::new()
                                .set_directory(&output_dir)
                                .pick_folder()
                            {
                                state.export_state.output_dir_override = Some(folder);
                            }
                        }
                        if state.export_state.output_dir_override.is_some() {
                            if ui.small_button("Reset").clicked() {
                                state.export_state.output_dir_override = None;
                            }
                        }
                    });

                    ui.add_space(spacing::SM);

                    let mut overwrite = state.settings.export.overwrite_without_prompt;
                    if ui
                        .checkbox(&mut overwrite, "Overwrite existing files")
                        .changed()
                    {
                        state.settings.export.overwrite_without_prompt = overwrite;
                    }

                    ui.add_space(spacing::XL);

                    // Export Button
                    let can_export = selected_count > 0;

                    if can_export {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(egui_phosphor::regular::CHECK_CIRCLE)
                                    .color(Color32::GREEN),
                            );
                            let format_name = current_format.display_name();
                            let define_suffix = if state.settings.export.generate_define_xml {
                                " + Define-XML"
                            } else {
                                ""
                            };
                            ui.label(
                                RichText::new(format!(
                                    "{} domain{} → {}{}",
                                    selected_count,
                                    if selected_count == 1 { "" } else { "s" },
                                    format_name,
                                    define_suffix
                                ))
                                .weak(),
                            );
                        });
                        ui.add_space(spacing::SM);
                    }

                    ui.add_enabled_ui(can_export, |ui| {
                        let text = if can_export {
                            format!("{} Export", egui_phosphor::regular::EXPORT)
                        } else {
                            format!(
                                "{} Select domains to export",
                                egui_phosphor::regular::EXPORT
                            )
                        };
                        let button = egui::Button::new(RichText::new(text).size(15.0))
                            .min_size(egui::vec2(ui.available_width().min(300.0), 40.0));
                        if ui.add(button).clicked() {
                            start_export = true;
                        }
                    });
                });
            });
        });

    if start_export {
        let total = state.export_state.selected_domains.len();
        state.export_state.progress = Some(ExportProgress::new(total));
        tracing::info!("Starting export of {} domains", total);
    }
}

/// Domain row in selection list
fn show_domain_row(
    ui: &mut Ui,
    state: &mut AppState,
    code: &str,
    display_name: &str,
    status: DomainStatus,
    row_count: usize,
    error_count: Option<usize>,
) {
    let can_export = can_export_domain(status, state.settings.developer.allow_export_with_errors);
    let is_selected = state.export_state.selected_domains.contains(code);
    let (status_icon, status_color) = domain_status_icon(status);

    ui.horizontal(|ui| {
        // Checkbox
        let mut selected = is_selected;
        ui.add_enabled_ui(can_export, |ui| {
            if ui.checkbox(&mut selected, "").changed() {
                if selected {
                    state.export_state.selected_domains.insert(code.to_string());
                } else {
                    state.export_state.selected_domains.remove(code);
                }
            }
        });

        // Status icon
        ui.label(RichText::new(status_icon).color(status_color));

        // Domain name with label
        let name_text = if can_export {
            RichText::new(display_name).strong()
        } else {
            RichText::new(display_name).weak()
        };
        ui.label(name_text);

        // Info on the right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(format!("{}", row_count)).weak().small());

            if let Some(errors) = error_count {
                if errors > 0 {
                    ui.label(
                        RichText::new(format!("{}", errors))
                            .color(ui.visuals().error_fg_color)
                            .small(),
                    );
                    ui.label(
                        RichText::new(egui_phosphor::regular::WARNING)
                            .color(ui.visuals().error_fg_color)
                            .small(),
                    );
                }
            }
        });
    });
}

/// Export progress screen
fn show_export_progress(ui: &mut Ui, state: &mut AppState, progress: ExportProgress) {
    ui.vertical_centered(|ui| {
        ui.add_space(spacing::XL);

        // Status icon
        if progress.completed {
            ui.label(
                RichText::new(egui_phosphor::regular::CHECK_CIRCLE)
                    .size(56.0)
                    .color(Color32::GREEN),
            );
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Export Complete").size(22.0).strong());
        } else if progress.error.is_some() {
            ui.label(
                RichText::new(egui_phosphor::regular::X_CIRCLE)
                    .size(56.0)
                    .color(ui.visuals().error_fg_color),
            );
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Export Failed").size(22.0).strong());
        } else {
            ui.spinner();
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Exporting...").size(22.0).strong());
        }

        ui.add_space(spacing::LG);

        // Progress bar
        ui.set_max_width(360.0);
        ui.add(
            egui::ProgressBar::new(progress.fraction())
                .show_percentage()
                .animate(!progress.completed && progress.error.is_none()),
        );

        ui.add_space(spacing::MD);

        // Current operation details
        if let Some(domain) = &progress.current_domain {
            ui.label(
                RichText::new(format!(
                    "Processing {} ({}/{})",
                    domain,
                    progress.completed_domains + 1,
                    progress.total_domains
                ))
                .weak(),
            );

            ui.add_space(spacing::SM);

            // Step list
            let steps = [
                (ExportDomainStep::ApplyingMappings, "Apply mappings"),
                (ExportDomainStep::NormalizingCT, "Normalize terminology"),
                (ExportDomainStep::GeneratingVariables, "Generate variables"),
                (ExportDomainStep::ValidatingOutput, "Validate"),
                (ExportDomainStep::WritingXpt, "Write XPT"),
                (ExportDomainStep::WritingDefineXml, "Define-XML"),
            ];

            ui.horizontal(|ui| {
                for (i, (step, _label)) in steps.iter().enumerate() {
                    let (icon, color) = if progress.domain_step == *step {
                        (
                            egui_phosphor::regular::CIRCLE_NOTCH,
                            ui.visuals().hyperlink_color,
                        )
                    } else if (progress.domain_step as u8) > (*step as u8) {
                        (egui_phosphor::regular::CHECK, Color32::GREEN)
                    } else {
                        (
                            egui_phosphor::regular::CIRCLE,
                            ui.visuals().weak_text_color(),
                        )
                    };
                    ui.label(RichText::new(icon).color(color).small());
                    if i < steps.len() - 1 {
                        ui.label(RichText::new("—").weak().small());
                    }
                }
            });

            ui.label(RichText::new(progress.domain_step.label()).small());
        } else {
            ui.label(RichText::new(&progress.current_step).weak());
        }

        // Error
        if let Some(error) = &progress.error {
            ui.add_space(spacing::MD);
            ui.label(RichText::new(error).color(ui.visuals().error_fg_color));
        }

        // Output files
        if progress.completed && !progress.output_files.is_empty() {
            ui.add_space(spacing::LG);
            ui.label(RichText::new("Created files:").weak().small());
            for file in &progress.output_files {
                ui.label(
                    RichText::new(file.file_name().unwrap_or_default().to_string_lossy())
                        .monospace()
                        .small(),
                );
            }
        }

        ui.add_space(spacing::LG);

        // Buttons
        if progress.completed || progress.error.is_some() {
            ui.horizontal(|ui| {
                if ui
                    .button(format!("{} Done", egui_phosphor::regular::CHECK))
                    .clicked()
                {
                    state.export_state.progress = None;
                }
                if progress.completed {
                    if let Some(first) = progress.output_files.first() {
                        if let Some(parent) = first.parent() {
                            let folder = parent.to_path_buf();
                            if ui
                                .button(format!(
                                    "{} Open Folder",
                                    egui_phosphor::regular::FOLDER_OPEN
                                ))
                                .clicked()
                            {
                                #[cfg(target_os = "macos")]
                                {
                                    let _ = std::process::Command::new("open").arg(&folder).spawn();
                                }
                                #[cfg(target_os = "windows")]
                                {
                                    let _ =
                                        std::process::Command::new("explorer").arg(&folder).spawn();
                                }
                                #[cfg(target_os = "linux")]
                                {
                                    let _ =
                                        std::process::Command::new("xdg-open").arg(&folder).spawn();
                                }
                            }
                        }
                    }
                }
            });
        }
    });
}

fn can_export_domain(status: DomainStatus, allow_with_errors: bool) -> bool {
    match status {
        DomainStatus::ReadyForExport => true,
        DomainStatus::ValidationFailed => allow_with_errors,
        DomainStatus::MappingComplete => true,
        _ => false,
    }
}

fn domain_status_icon(status: DomainStatus) -> (&'static str, egui::Color32) {
    match status {
        DomainStatus::ReadyForExport => (egui_phosphor::regular::CHECK_CIRCLE, Color32::GREEN),
        DomainStatus::ValidationFailed => (
            egui_phosphor::regular::WARNING,
            egui::Color32::from_rgb(239, 68, 68),
        ),
        DomainStatus::MappingComplete => (
            egui_phosphor::regular::CHECK,
            egui::Color32::from_rgb(59, 130, 246),
        ),
        DomainStatus::MappingInProgress => (
            egui_phosphor::regular::ARROWS_LEFT_RIGHT,
            egui::Color32::from_rgb(234, 179, 8),
        ),
        DomainStatus::Loading => (
            egui_phosphor::regular::SPINNER,
            egui::Color32::from_rgb(156, 163, 175),
        ),
        DomainStatus::NotStarted => (
            egui_phosphor::regular::MINUS,
            egui::Color32::from_rgb(156, 163, 175),
        ),
    }
}
