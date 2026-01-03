//! Home screen view
//!
//! Study folder selection with CDISC standard selection.
//! Currently focused on SDTM. ADaM and SEND are not available.

use crate::state::{AppState, WorkflowMode};
use crate::theme::spacing;
use egui::{Color32, RichText, Ui, Vec2};
use std::path::PathBuf;

/// Action returned from the home view.
pub enum HomeAction {
    /// No action
    None,
    /// Load a new study from folder
    LoadStudy(PathBuf),
    /// Close the current study
    CloseStudy,
}

/// Home screen view
pub struct HomeView;

impl HomeView {
    /// Render the home screen
    ///
    /// Returns an action if user triggered something.
    pub fn show(ui: &mut Ui, state: &mut AppState) -> HomeAction {
        let mut clicked_domain: Option<String> = None;
        let mut go_to_export = false;
        let mut selected_folder: Option<PathBuf> = None;
        let mut new_workflow_mode: Option<WorkflowMode> = None;
        let mut close_study = false;

        ui.vertical_centered(|ui| {
            ui.add_space(spacing::XL);

            // Title
            ui.heading(RichText::new("CDISC Data Transpiler").size(32.0));
            ui.add_space(spacing::SM);
            ui.label(
                RichText::new("Transform clinical data to regulatory formats")
                    .weak()
                    .size(14.0),
            );

            ui.add_space(spacing::XL);

            // Show study info if loaded, otherwise show standard selector
            if state.study.is_some() {
                Self::show_loaded_study(ui, state, &mut clicked_domain, &mut go_to_export);
            } else {
                Self::show_standard_selector(
                    ui,
                    state,
                    &mut new_workflow_mode,
                    &mut selected_folder,
                );

                // Recent studies
                if !state.settings.recent_studies.is_empty() {
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
            }
        });

        // Show close study confirmation modal
        if state.ui.close_study_confirm {
            close_study = Self::show_close_confirm_modal(ui, state);
        }

        // Handle workflow mode change
        if let Some(mode) = new_workflow_mode {
            state.set_workflow_mode(mode);
        }

        // Handle navigation after borrowing ends
        if let Some(domain) = clicked_domain {
            state.open_domain(domain);
        }
        if go_to_export {
            state.go_export();
        }

        // Return appropriate action
        if close_study {
            HomeAction::CloseStudy
        } else if let Some(folder) = selected_folder {
            HomeAction::LoadStudy(folder)
        } else {
            HomeAction::None
        }
    }

    /// Show the close study confirmation modal.
    /// Returns true if user confirmed closing.
    fn show_close_confirm_modal(ui: &mut Ui, state: &mut AppState) -> bool {
        let mut confirmed = false;

        egui::Window::new("Close Study")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.set_min_width(350.0);

                ui.vertical_centered(|ui| {
                    ui.add_space(spacing::MD);

                    ui.label(
                        RichText::new(egui_phosphor::regular::WARNING)
                            .size(48.0)
                            .color(ui.visuals().warn_fg_color),
                    );

                    ui.add_space(spacing::MD);

                    ui.label(
                        RichText::new("Are you sure you want to close this study?")
                            .strong()
                            .size(16.0),
                    );

                    ui.add_space(spacing::SM);

                    ui.label(
                        RichText::new("All unsaved mapping progress will be lost.")
                            .weak()
                            .color(ui.visuals().warn_fg_color),
                    );

                    ui.add_space(spacing::LG);

                    ui.horizontal(|ui| {
                        ui.add_space(40.0);

                        if ui
                            .button(format!("{} Cancel", egui_phosphor::regular::X))
                            .clicked()
                        {
                            state.ui.close_study_confirm = false;
                        }

                        ui.add_space(spacing::MD);

                        let close_btn = ui.add(
                            egui::Button::new(RichText::new(format!(
                                "{} Close Study",
                                egui_phosphor::regular::TRASH
                            )))
                            .fill(Color32::from_rgb(192, 57, 43)),
                        );

                        if close_btn.clicked() {
                            state.ui.close_study_confirm = false;
                            confirmed = true;
                        }

                        ui.add_space(40.0);
                    });

                    ui.add_space(spacing::MD);
                });
            });

        confirmed
    }

    /// Show the standard selector with dropdown and open button
    fn show_standard_selector(
        ui: &mut Ui,
        state: &AppState,
        new_mode: &mut Option<WorkflowMode>,
        selected_folder: &mut Option<PathBuf>,
    ) {
        // Container frame for the selector
        egui::Frame::new()
            .fill(ui.visuals().widgets.noninteractive.bg_fill)
            .corner_radius(8.0)
            .inner_margin(spacing::LG)
            .show(ui, |ui| {
                ui.set_min_width(400.0);

                let mut selected = state.workflow_mode;

                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(RichText::new("Select a CDISC Standard").strong().size(16.0));
                    ui.add_space(spacing::MD);

                    // Standard dropdown - centered
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                        egui::ComboBox::from_id_salt("standard_selector")
                            .width(300.0)
                            .selected_text(format!(
                                "{} - {}",
                                selected.display_name(),
                                selected.tagline()
                            ))
                            .show_ui(ui, |ui| {
                                // SDTM - fully supported
                                let sdtm_label = format!(
                                    "{} - {}",
                                    WorkflowMode::Sdtm.display_name(),
                                    WorkflowMode::Sdtm.tagline()
                                );
                                if ui
                                    .selectable_value(&mut selected, WorkflowMode::Sdtm, sdtm_label)
                                    .changed()
                                {
                                    *new_mode = Some(WorkflowMode::Sdtm);
                                }

                                // ADaM - not yet available
                                let adam_label = format!(
                                    "{} - {} (Not Available)",
                                    WorkflowMode::Adam.display_name(),
                                    WorkflowMode::Adam.tagline()
                                );
                                ui.add_enabled(
                                    false,
                                    egui::Button::new(RichText::new(adam_label).weak())
                                        .frame(false),
                                );

                                // SEND - not yet available
                                let send_label = format!(
                                    "{} - {} (Not Available)",
                                    WorkflowMode::Send.display_name(),
                                    WorkflowMode::Send.tagline()
                                );
                                ui.add_enabled(
                                    false,
                                    egui::Button::new(RichText::new(send_label).weak())
                                        .frame(false),
                                );
                            });
                    });

                    ui.add_space(spacing::SM);

                    // Description for selected standard
                    ui.label(RichText::new(selected.description()).weak().small());

                    ui.add_space(spacing::LG);

                    // Open folder button
                    let button = ui.add_sized(
                        Vec2::new(200.0, 40.0),
                        egui::Button::new(RichText::new(format!(
                            "{} Open Study Folder",
                            egui_phosphor::regular::FOLDER_OPEN
                        ))),
                    );

                    if button.clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            tracing::info!("Selected folder: {:?}", folder);
                            *selected_folder = Some(folder);
                        }
                    }
                });
            });
    }

    /// Show loaded study information
    fn show_loaded_study(
        ui: &mut Ui,
        state: &mut AppState,
        clicked_domain: &mut Option<String>,
        go_to_export: &mut bool,
    ) {
        let study = state.study.as_ref().unwrap();

        ui.separator();
        ui.add_space(spacing::MD);

        // Study header with mode badge and close button
        ui.horizontal(|ui| {
            ui.heading(&study.study_id);
            ui.add_space(spacing::SM);

            // Mode badge
            let badge_color = match state.workflow_mode {
                WorkflowMode::Sdtm => Color32::from_rgb(52, 152, 219),
                WorkflowMode::Adam => Color32::from_rgb(155, 89, 182),
                WorkflowMode::Send => Color32::from_rgb(46, 204, 113),
            };
            ui.label(
                RichText::new(state.workflow_mode.display_name())
                    .color(badge_color)
                    .strong(),
            );

            // Push close button to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(format!("{} Close Study", egui_phosphor::regular::X))
                    .clicked()
                {
                    state.ui.close_study_confirm = true;
                }
            });
        });

        let study = state.study.as_ref().unwrap();
        ui.label(
            RichText::new(study.study_folder.display().to_string())
                .weak()
                .small(),
        );

        ui.add_space(spacing::MD);

        // Extract data needed for rendering before borrowing state mutably
        let domain_codes: Vec<String> = study
            .domain_codes_dm_first()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        // Domain list
        ui.label(
            RichText::new(format!(
                "{} Discovered Domains",
                egui_phosphor::regular::DATABASE
            ))
            .strong(),
        );
        ui.add_space(spacing::SM);

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for code in &domain_codes {
                    let Some(study) = state.study.as_ref() else {
                        continue;
                    };
                    let Some(domain) = study.get_domain(code) else {
                        continue;
                    };

                    let display_name = domain.display_name(code);
                    let row_count = domain.row_count();
                    let is_mapping_complete = domain.is_mapping_complete();

                    let (status_icon, status_color) = if is_mapping_complete {
                        (egui_phosphor::regular::CHECK_CIRCLE, Color32::GREEN)
                    } else if domain.is_touched() {
                        (egui_phosphor::regular::PENCIL, ui.visuals().warn_fg_color)
                    } else {
                        (egui_phosphor::regular::CIRCLE, ui.visuals().text_color())
                    };

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(status_icon).color(status_color));

                        let button = ui.button(&display_name);

                        if button.clicked() {
                            *clicked_domain = Some(code.to_string());
                        }

                        ui.label(RichText::new(format!("{} rows", row_count)).weak().small());
                    });
                }
            });

        ui.add_space(spacing::MD);

        // Export button
        if ui
            .button(format!("{} Go to Export", egui_phosphor::regular::EXPORT))
            .clicked()
        {
            *go_to_export = true;
        }
    }
}
