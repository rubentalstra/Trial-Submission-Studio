//! Export screen view
//!
//! Master-detail layout: domain selection (left) and configuration (right).
//! Uses modals for progress and completion.

use crate::export::{
    ExportBypasses, ExportConfig, ExportPhase, ExportStep, can_export_domain,
    count_bypassed_issues, count_supp_columns, estimate_supp_row_count, get_domain_status,
    spawn_export,
};
use crate::settings::ExportFormat;
use crate::state::AppState;
use crate::theme::spacing;
use egui::{Color32, RichText, Ui};

/// Export view
pub struct ExportView;

impl ExportView {
    /// Render the export screen
    pub fn show(ui: &mut Ui, state: &mut AppState) {
        // Poll for export updates
        state.poll_export_updates();

        // Handle modals based on phase
        match state.ui.export.phase {
            ExportPhase::Exporting => {
                show_export_layout(ui, state);
                show_progress_modal(ui, state);
            }
            ExportPhase::Complete => {
                show_export_layout(ui, state);
                show_completion_modal(ui, state);
            }
            ExportPhase::Idle => {
                if state.study.is_some() {
                    show_export_layout(ui, state);
                } else {
                    show_no_study(ui, state);
                }
            }
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

/// Build export bypasses from settings
fn build_bypasses(state: &AppState) -> ExportBypasses {
    let dev = &state.settings.developer;
    ExportBypasses {
        developer_mode: dev.enabled,
        allow_errors: dev.allow_export_with_errors,
        allow_incomplete_mappings: dev.allow_incomplete_mappings,
        bypassed_categories: dev.bypassed_categories.clone(),
        bypassed_rule_ids: dev.bypassed_rule_ids.clone(),
    }
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
        .size(egui_extras::Size::relative(0.45).at_least(250.0)) // Left column
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

/// Show domain selection list with status icons
fn show_domain_selection(ui: &mut Ui, state: &mut AppState) {
    ui.label(
        RichText::new(format!(
            "{} Select Domains",
            egui_phosphor::regular::DATABASE
        ))
        .strong(),
    );
    ui.add_space(spacing::SM);

    let bypasses = build_bypasses(state);

    // Build domain info list
    let domain_info: Vec<_> = {
        let Some(study) = state.study() else {
            return;
        };

        study
            .domain_codes()
            .iter()
            .map(|code| {
                let code = code.to_string();
                if let Some(domain) = study.get_domain(&code) {
                    let status = get_domain_status(domain, &bypasses);
                    let row_count = domain.row_count();
                    let supp_count = count_supp_columns(domain);
                    let supp_rows = if supp_count > 0 {
                        estimate_supp_row_count(domain)
                    } else {
                        0
                    };
                    (code, Some((status, row_count, supp_count, supp_rows)))
                } else {
                    (code, None)
                }
            })
            .collect()
    };

    let domain_codes: Vec<String> = domain_info.iter().map(|(c, _)| c.clone()).collect();

    // Collect exportable domain codes first (to avoid borrow checker issues)
    let exportable_codes: Vec<String> = {
        let Some(study) = state.study() else {
            return;
        };
        domain_codes
            .iter()
            .filter(|code| {
                study
                    .get_domain(code)
                    .map(|d| can_export_domain(d, &bypasses))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    };

    // Select all / deselect all (only exportable domains)
    ui.horizontal(|ui| {
        if ui.small_button("Select All").clicked() {
            for code in &exportable_codes {
                state.ui.export.selected_domains.insert(code.clone());
            }
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

            if let Some((status, row_count, supp_count, supp_rows)) = info {
                let can_export = status.can_export();

                ui.horizontal(|ui| {
                    // Checkbox (disabled if blocked/incomplete)
                    let mut checked = is_selected;
                    if ui
                        .add_enabled(can_export, egui::Checkbox::without_text(&mut checked))
                        .changed()
                    {
                        toggle_domain = Some(code.clone());
                    }

                    // Domain code
                    let code_text = if can_export {
                        RichText::new(code)
                    } else {
                        RichText::new(code).weak()
                    };
                    ui.label(code_text);

                    // Status icon with tooltip
                    let icon_response =
                        ui.label(RichText::new(status.icon()).color(status.color()));
                    icon_response.on_hover_text(status.tooltip());

                    // Row count
                    ui.label(RichText::new(format!("{} rows", row_count)).weak().small());
                });

                // Show SUPP info if configured (indented, non-selectable)
                if *supp_count > 0 {
                    ui.horizontal(|ui| {
                        ui.add_space(24.0); // Indent
                        ui.label(
                            RichText::new(format!(
                                "  {} SUPP{}",
                                egui_phosphor::regular::ARROW_ELBOW_DOWN_RIGHT,
                                code.to_lowercase()
                            ))
                            .weak()
                            .small(),
                        );
                        ui.label(RichText::new(format!("{} rows", supp_rows)).weak().small());
                    });
                }
            } else {
                // Domain not found in study
                ui.horizontal(|ui| {
                    ui.add_enabled(false, egui::Checkbox::without_text(&mut false));
                    ui.label(RichText::new(code).weak());
                    ui.label(
                        RichText::new(egui_phosphor::regular::WARNING)
                            .color(ui.visuals().warn_fg_color),
                    );
                });
            }
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
    let study_folder = state.study().map(|s| s.study_folder.clone());
    let output_dir = state
        .ui
        .export
        .output_dir
        .clone()
        .or_else(|| study_folder.as_ref().map(|f| f.join("export")));

    if let Some(dir) = &output_dir {
        ui.horizontal(|ui| {
            ui.label(RichText::new(dir.display().to_string()).monospace().small());
            if ui.small_button("Change").clicked() {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    state.ui.export.output_dir = Some(folder);
                }
            }
        });
    }

    ui.add_space(spacing::MD);

    // Format selection
    ui.label(RichText::new("Data Format").weak());
    let current_format = state
        .ui
        .export
        .format
        .unwrap_or(state.settings.export.default_format);

    for format in ExportFormat::all() {
        if ui
            .selectable_label(current_format == *format, format.display_name())
            .clicked()
        {
            state.ui.export.format = Some(*format);
        }
        ui.label(RichText::new(format.description()).weak().small());
        ui.add_space(2.0);
    }

    ui.add_space(spacing::SM);
    ui.label(
        RichText::new("Define-XML is always generated with exports")
            .weak()
            .small(),
    );

    ui.add_space(spacing::LG);
    ui.separator();
    ui.add_space(spacing::MD);

    // Show bypass info if developer mode is on
    let bypasses = build_bypasses(state);
    if bypasses.developer_mode {
        let bypassed_count = count_bypassed_issues_total(state, &bypasses);
        if bypassed_count > 0 {
            ui.label(
                RichText::new(format!(
                    "{} Dev: {} issue{} bypassed",
                    egui_phosphor::regular::WARNING_CIRCLE,
                    bypassed_count,
                    if bypassed_count == 1 { "" } else { "s" }
                ))
                .color(ui.visuals().warn_fg_color)
                .small(),
            );
            ui.add_space(spacing::SM);
        }
    }

    // Export button
    let selected_count = state.ui.export.selection_count();
    let can_export = selected_count > 0;

    ui.add_enabled_ui(can_export, |ui| {
        if ui
            .button(
                RichText::new(format!(
                    "Export {} Domain{} {}",
                    selected_count,
                    if selected_count == 1 { "" } else { "s" },
                    egui_phosphor::regular::ARROW_RIGHT
                ))
                .size(16.0),
            )
            .clicked()
        {
            start_export(state);
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

/// Count bypassed issues across all selected domains
fn count_bypassed_issues_total(state: &AppState, bypasses: &ExportBypasses) -> usize {
    let Some(study) = state.study() else {
        return 0;
    };

    let mut total = 0;
    for code in &state.ui.export.selected_domains {
        if let Some(domain) = study.get_domain(code) {
            total += count_bypassed_issues(domain, bypasses);
        }
    }
    total
}

/// Start the export process
fn start_export(state: &mut AppState) {
    let Some(study) = state.study.as_ref() else {
        return;
    };

    let bypasses = build_bypasses(state);
    let study_folder = study.study_folder.clone();

    // Build config
    let config = ExportConfig {
        output_dir: state
            .ui
            .export
            .output_dir
            .clone()
            .unwrap_or_else(|| study_folder.join("export")),
        format: state
            .ui
            .export
            .format
            .unwrap_or(state.settings.export.default_format),
        selected_domains: state.ui.export.selected_domains.clone(),
        bypasses,
    };

    // Calculate expected file count (domain + SUPP per domain with SUPP + define.xml)
    let mut expected_files = 1; // define.xml
    for code in &config.selected_domains {
        expected_files += 1; // domain file
        if let Some(domain) = study.get_domain(code) {
            if count_supp_columns(domain) > 0 {
                expected_files += 1; // SUPP file
            }
        }
    }

    // Set up UI state
    state.ui.export.phase = ExportPhase::Exporting;
    state.ui.export.current_domain = None;
    state.ui.export.current_step = ExportStep::Preparing;
    state.ui.export.written_files.clear();
    state.ui.export.total_expected_files = expected_files;
    state.ui.export.result = None;

    // Spawn export thread
    let handle = spawn_export(config, study.clone(), state.export_sender.clone());
    state.ui.export.cancel_handle = Some(handle);
}

/// Show progress modal
fn show_progress_modal(ui: &mut Ui, state: &mut AppState) {
    let modal_id = ui.id().with("export_progress_modal");

    egui::Window::new("Exporting...")
        .id(modal_id)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.set_min_width(300.0);
            ui.vertical_centered(|ui| {
                ui.add_space(spacing::MD);
                ui.spinner();
                ui.add_space(spacing::MD);

                // Current step
                if let Some(ref domain) = state.ui.export.current_domain {
                    ui.label(RichText::new(domain).strong());
                }
                ui.label(state.ui.export.current_step.label());

                ui.add_space(spacing::SM);

                // Progress bar
                let fraction = state.ui.export.progress_fraction();
                ui.add(egui::ProgressBar::new(fraction).show_percentage());

                ui.add_space(spacing::SM);
                ui.label(
                    RichText::new(format!(
                        "{} / {} files",
                        state.ui.export.written_files.len(),
                        state.ui.export.total_expected_files
                    ))
                    .weak()
                    .small(),
                );

                ui.add_space(spacing::MD);

                // Cancel button
                if ui.button("Cancel").clicked() {
                    state.ui.export.request_cancel();
                }
            });
        });
}

/// Show completion modal
fn show_completion_modal(ui: &mut Ui, state: &mut AppState) {
    // Clone result data to avoid borrow conflicts
    let result_clone = state.ui.export.result.clone();

    let modal_id = ui.id().with("export_complete_modal");
    let mut should_reset = false;
    let mut should_retry = false;
    let mut open_folder: Option<std::path::PathBuf> = None;

    egui::Window::new("Export Complete")
        .id(modal_id)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.set_min_width(350.0);
            ui.vertical_centered(|ui| {
                ui.add_space(spacing::MD);

                match &result_clone {
                    Some(Ok(result)) => {
                        // Success
                        ui.label(
                            RichText::new(egui_phosphor::regular::CHECK_CIRCLE)
                                .size(48.0)
                                .color(Color32::from_rgb(34, 197, 94)),
                        );
                        ui.add_space(spacing::MD);
                        ui.label(RichText::new("Export Complete!").size(18.0).strong());
                        ui.add_space(spacing::SM);
                        ui.label(format!("{} files written", result.written_files.len()));
                        ui.label(
                            RichText::new(format!("Completed in {}ms", result.elapsed_ms))
                                .weak()
                                .small(),
                        );

                        ui.add_space(spacing::MD);

                        // Show output path
                        ui.label(RichText::new("Output:").weak());
                        ui.label(
                            RichText::new(result.output_dir.display().to_string())
                                .monospace()
                                .small(),
                        );

                        ui.add_space(spacing::LG);

                        ui.horizontal(|ui| {
                            if ui.button("Show in Folder").clicked() {
                                open_folder = Some(result.output_dir.clone());
                            }

                            if ui.button("Done").clicked() {
                                should_reset = true;
                            }
                        });
                    }
                    Some(Err(error)) => {
                        // Error
                        ui.label(
                            RichText::new(egui_phosphor::regular::X_CIRCLE)
                                .size(48.0)
                                .color(Color32::from_rgb(239, 68, 68)),
                        );
                        ui.add_space(spacing::MD);
                        ui.label(RichText::new("Export Failed").size(18.0).strong());
                        ui.add_space(spacing::SM);

                        if let Some(ref domain) = error.domain {
                            ui.label(format!("Domain: {}", domain));
                        }
                        ui.label(RichText::new(&error.message).color(ui.visuals().error_fg_color));

                        if let Some(ref details) = error.details {
                            ui.label(RichText::new(details).weak().small());
                        }

                        ui.add_space(spacing::LG);

                        ui.horizontal(|ui| {
                            if ui.button("Retry").clicked() {
                                should_retry = true;
                            }
                            if ui.button("Close").clicked() {
                                should_reset = true;
                            }
                        });
                    }
                    None => {
                        // Cancelled or unexpected state
                        ui.label("Export was cancelled.");
                        ui.add_space(spacing::MD);
                        if ui.button("Close").clicked() {
                            should_reset = true;
                        }
                    }
                }
            });
        });

    // Handle actions after the window (to avoid borrow issues)
    if let Some(folder) = open_folder {
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&folder).spawn();

        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("explorer").arg(&folder).spawn();

        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&folder).spawn();
    }

    if should_retry {
        state.ui.export.reset();
        start_export(state);
    } else if should_reset {
        state.ui.export.reset();
    }
}
