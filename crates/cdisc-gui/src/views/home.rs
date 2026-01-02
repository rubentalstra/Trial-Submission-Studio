//! Home screen view
//!
//! Study folder selection with CDISC standard selection (SDTM, ADaM, SEND).

use crate::state::{AppState, WorkflowMode};
use crate::theme::spacing;
use egui::{Color32, RichText, Ui, Vec2};
use std::path::PathBuf;

/// Home screen view
pub struct HomeView;

impl HomeView {
    /// Render the home screen
    ///
    /// Returns a folder path if the user selected one to load.
    pub fn show(ui: &mut Ui, state: &mut AppState) -> Option<PathBuf> {
        let mut clicked_domain: Option<String> = None;
        let mut go_to_export = false;
        let mut selected_folder: Option<PathBuf> = None;
        let mut new_workflow_mode: Option<WorkflowMode> = None;

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
            if let Some(study) = state.study() {
                Self::show_loaded_study(
                    ui,
                    state,
                    study,
                    &mut clicked_domain,
                    &mut go_to_export,
                    &mut selected_folder,
                );
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

        selected_folder
    }

    /// Show the standard selector cards (SDTM, ADaM, SEND)
    fn show_standard_selector(
        ui: &mut Ui,
        state: &AppState,
        new_mode: &mut Option<WorkflowMode>,
        selected_folder: &mut Option<PathBuf>,
    ) {
        ui.label(RichText::new("Select a CDISC Standard").strong().size(16.0));
        ui.add_space(spacing::MD);

        // Standard cards in horizontal layout
        ui.horizontal(|ui| {
            ui.add_space(spacing::MD);

            // SDTM Card
            Self::show_standard_card(
                ui,
                WorkflowMode::Sdtm,
                state.workflow_mode,
                "63 domains",
                "DM, AE, LB, VS...",
                new_mode,
                selected_folder,
            );

            ui.add_space(spacing::MD);

            // ADaM Card
            Self::show_standard_card(
                ui,
                WorkflowMode::Adam,
                state.workflow_mode,
                "3 structures",
                "ADSL, BDS, OCCDS",
                new_mode,
                selected_folder,
            );

            ui.add_space(spacing::MD);

            // SEND Card
            Self::show_standard_card(
                ui,
                WorkflowMode::Send,
                state.workflow_mode,
                "30+ domains",
                "EX, BW, LB, MI...",
                new_mode,
                selected_folder,
            );

            ui.add_space(spacing::MD);
        });
    }

    /// Show a single standard card
    fn show_standard_card(
        ui: &mut Ui,
        mode: WorkflowMode,
        current_mode: WorkflowMode,
        stat_line: &str,
        examples: &str,
        new_mode: &mut Option<WorkflowMode>,
        selected_folder: &mut Option<PathBuf>,
    ) {
        let is_selected = mode == current_mode;
        let card_color = if is_selected {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().widgets.noninteractive.bg_fill
        };

        egui::Frame::none()
            .fill(card_color)
            .corner_radius(8.0)
            .inner_margin(spacing::MD)
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(180.0, 200.0));
                ui.vertical_centered(|ui| {
                    // Standard name
                    ui.heading(RichText::new(mode.display_name()).size(24.0));
                    ui.add_space(spacing::XS);

                    // Tagline
                    ui.label(RichText::new(mode.tagline()).weak());
                    ui.add_space(spacing::SM);

                    // Statistics
                    ui.label(RichText::new(stat_line).strong());
                    ui.label(RichText::new(examples).weak().small());

                    ui.add_space(spacing::MD);

                    // Select/Open button
                    let button_text = if is_selected {
                        format!("{} Open Study...", egui_phosphor::regular::FOLDER_OPEN)
                    } else {
                        format!("{} Select", egui_phosphor::regular::CHECK)
                    };

                    if ui.button(button_text).clicked() {
                        if is_selected {
                            // Open file dialog
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                tracing::info!("Selected folder: {:?}", folder);
                                *selected_folder = Some(folder);
                            }
                        } else {
                            // Select this mode
                            *new_mode = Some(mode);
                        }
                    }
                });
            });
    }

    /// Show loaded study information
    fn show_loaded_study(
        ui: &mut Ui,
        state: &AppState,
        study: &crate::state::StudyState,
        clicked_domain: &mut Option<String>,
        go_to_export: &mut bool,
        selected_folder: &mut Option<PathBuf>,
    ) {
        ui.separator();
        ui.add_space(spacing::MD);

        // Study header with mode badge
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
        });

        ui.label(
            RichText::new(study.study_folder.display().to_string())
                .weak()
                .small(),
        );

        ui.add_space(spacing::MD);

        // Back to home button
        if ui
            .button(format!(
                "{} Change Standard",
                egui_phosphor::regular::ARROW_LEFT
            ))
            .clicked()
        {
            // This will be handled by clearing study
        }

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
                    let Some(domain) = study.get_domain(code) else {
                        continue;
                    };

                    let display_name = domain.display_name(code);
                    let row_count = domain.row_count();
                    let is_accessible = state.is_domain_accessible(code);
                    let lock_reason = state.domain_lock_reason(code);
                    let is_mapping_complete = domain.is_mapping_complete();

                    let (status_icon, status_color) = if !is_accessible {
                        (egui_phosphor::regular::LOCK, ui.visuals().weak_text_color())
                    } else if is_mapping_complete {
                        (egui_phosphor::regular::CHECK_CIRCLE, Color32::GREEN)
                    } else if domain.is_touched() {
                        (egui_phosphor::regular::PENCIL, ui.visuals().warn_fg_color)
                    } else {
                        (egui_phosphor::regular::CIRCLE, ui.visuals().text_color())
                    };

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(status_icon).color(status_color));

                        let button =
                            ui.add_enabled(is_accessible, egui::Button::new(&display_name));

                        if button.clicked() && is_accessible {
                            *clicked_domain = Some(code.to_string());
                        }

                        if let Some(reason) = lock_reason {
                            button.on_hover_text(reason);
                        }

                        ui.label(RichText::new(format!("{} rows", row_count)).weak().small());

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
            *go_to_export = true;
        }
    }
}
