//! Transform tab
//!
//! Displays SDTM transformations derived from accepted mappings.
//! Transform list is built from mapping state; display data fetched on-the-fly.

use crate::services::MappingService;
use crate::state::{AppState, AutoTransform, TransformState};
use crate::theme::{colors, spacing};
use egui::{RichText, Ui};

pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    let theme = colors(state.preferences.dark_mode);

    // Rebuild transforms from current mapping state (cheap operation)
    rebuild_transforms_if_needed(state, domain_code);

    // Get data for display
    let Some(study) = &state.study else {
        ui.label("No study loaded");
        return;
    };
    let Some(domain) = study.get_domain(domain_code) else {
        ui.label("Domain not found");
        return;
    };
    let Some(ts) = &domain.transform_state else {
        ui.label("Loading...");
        return;
    };

    // Header
    ui.label(
        RichText::new(format!(
            "{} Automatic Transformations",
            egui_phosphor::regular::SHUFFLE
        ))
        .strong(),
    );
    ui.add_space(spacing::SM);
    ui.separator();

    if ts.transforms.is_empty() {
        ui.add_space(spacing::LG);
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} No transformations - accept mappings first",
                    egui_phosphor::regular::INFO
                ))
                .color(theme.text_muted),
            );
        });
        return;
    }

    // Clone what we need for layout
    let transforms: Vec<_> = ts.transforms.iter().cloned().collect();
    let selected_idx = ts.selected_idx;

    let available_height = ui.available_height();

    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(300.0))
        .size(egui_extras::Size::exact(1.0))
        .size(egui_extras::Size::remainder())
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        show_transform_list(ui, state, domain_code, &transforms, selected_idx, &theme);
                    });
            });

            strip.cell(|ui| {
                ui.separator();
            });

            strip.cell(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        show_transform_detail(ui, state, domain_code, &transforms, selected_idx, &theme);
                    });
            });
        });
}

/// Rebuild transforms from current mapping state
fn rebuild_transforms_if_needed(state: &mut AppState, domain_code: &str) {
    let mut transforms = Vec::new();

    if let Some(study) = &state.study {
        if let Some(domain) = study.get_domain(domain_code) {
            if let Some(ms) = &domain.mapping_state {
                // USUBJID prefix - if USUBJID has an accepted mapping
                if ms.get_accepted_for("USUBJID").is_some() {
                    transforms.push(AutoTransform::UsUbjIdPrefix);
                }

                // --SEQ is always needed
                let seq_column = format!("{}SEQ", domain_code);
                transforms.push(AutoTransform::SequenceNumbers { seq_column });

                // CT normalization for each mapped variable with a codelist
                for variable in &ms.sdtm_domain.variables {
                    if let Some(codelist_code) = &variable.codelist_code {
                        if ms.get_accepted_for(&variable.name).is_some() {
                            let code = codelist_code.split(';').next().unwrap_or("").trim();
                            if !code.is_empty() {
                                transforms.push(AutoTransform::CtNormalization {
                                    variable: variable.name.clone(),
                                    codelist_code: code.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Store transforms
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            let selected_idx = domain.transform_state.as_ref().and_then(|ts| ts.selected_idx);
            domain.transform_state = Some(TransformState {
                transforms,
                selected_idx,
            });
        }
    }
}

fn show_transform_list(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    transforms: &[AutoTransform],
    selected_idx: Option<usize>,
    theme: &crate::theme::ThemeColors,
) {
    let mut new_selection: Option<usize> = None;

    // Group by category
    let identifiers: Vec<_> = transforms
        .iter()
        .enumerate()
        .filter(|(_, t)| t.is_identifier())
        .collect();

    let ct: Vec<_> = transforms
        .iter()
        .enumerate()
        .filter(|(_, t)| matches!(t, AutoTransform::CtNormalization { .. }))
        .collect();

    if !identifiers.is_empty() {
        ui.label(
            RichText::new(format!(
                "{} Identifiers",
                egui_phosphor::regular::IDENTIFICATION_BADGE
            ))
            .strong()
            .color(theme.text_muted),
        );
        ui.add_space(spacing::SM);

        for (idx, t) in &identifiers {
            if render_row(ui, *idx, t, selected_idx == Some(*idx), theme) {
                new_selection = Some(*idx);
            }
        }
    }

    if !ct.is_empty() {
        if !identifiers.is_empty() {
            ui.add_space(spacing::MD);
            ui.separator();
            ui.add_space(spacing::SM);
        }

        ui.label(
            RichText::new(format!(
                "{} Controlled Terminology",
                egui_phosphor::regular::LIST_CHECKS
            ))
            .strong()
            .color(theme.text_muted),
        );
        ui.add_space(spacing::SM);

        for (idx, t) in &ct {
            if render_row(ui, *idx, t, selected_idx == Some(*idx), theme) {
                new_selection = Some(*idx);
            }
        }
    }

    if let Some(idx) = new_selection {
        if let Some(study) = &mut state.study {
            if let Some(domain) = study.get_domain_mut(domain_code) {
                if let Some(ts) = &mut domain.transform_state {
                    ts.selected_idx = Some(idx);
                }
            }
        }
    }
}

fn render_row(
    ui: &mut Ui,
    _idx: usize,
    transform: &AutoTransform,
    is_selected: bool,
    theme: &crate::theme::ThemeColors,
) -> bool {
    let mut clicked = false;

    ui.horizontal(|ui| {
        ui.label(RichText::new(transform.icon()).color(theme.accent));

        let text = if is_selected {
            RichText::new(transform.target_variable()).strong()
        } else {
            RichText::new(transform.target_variable())
        };

        if ui.selectable_label(is_selected, text).clicked() {
            clicked = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(transform.display_name())
                    .color(theme.text_muted)
                    .small(),
            );
        });
    });

    clicked
}

fn show_transform_detail(
    ui: &mut Ui,
    state: &AppState,
    domain_code: &str,
    transforms: &[AutoTransform],
    selected_idx: Option<usize>,
    theme: &crate::theme::ThemeColors,
) {
    let Some(idx) = selected_idx else {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} Select a transform",
                    egui_phosphor::regular::INFO
                ))
                .color(theme.text_muted),
            );
        });
        return;
    };

    let Some(transform) = transforms.get(idx) else {
        return;
    };

    // Get live data from state
    let (study_id, mapping_state, source_data) = {
        let Some(study) = &state.study else { return };
        let Some(domain) = study.get_domain(domain_code) else { return };
        let Some(ms) = &domain.mapping_state else { return };
        (study.study_id.clone(), ms, &domain.source_data)
    };

    // Header
    ui.horizontal(|ui| {
        ui.label(RichText::new(transform.icon()).size(24.0).color(theme.accent));
        ui.heading(transform.display_name());
    });

    ui.add_space(spacing::MD);
    ui.separator();
    ui.add_space(spacing::SM);

    match transform {
        AutoTransform::UsUbjIdPrefix => {
            // Get source column from mapping
            if let Some((source_col, _)) = mapping_state.get_accepted_for("USUBJID") {
                let samples = MappingService::get_sample_values(source_data, source_col, 3);

                ui.label(RichText::new("Mapping").strong().color(theme.text_muted));
                ui.add_space(spacing::SM);

                egui::Grid::new("usubjid_detail")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Source").color(theme.text_muted));
                        ui.label(source_col);
                        ui.end_row();

                        ui.label(RichText::new("Target").color(theme.text_muted));
                        ui.label("USUBJID");
                        ui.end_row();

                        ui.label(RichText::new("Prefix").color(theme.text_muted));
                        ui.label(RichText::new(&study_id).color(theme.accent));
                        ui.end_row();
                    });

                if !samples.is_empty() {
                    ui.add_space(spacing::MD);
                    ui.label(RichText::new("Sample Values").strong().color(theme.text_muted));
                    ui.add_space(spacing::SM);

                    for val in &samples {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(val).code());
                            ui.label(RichText::new("→").color(theme.text_muted));
                            ui.label(RichText::new(format!("{}-{}", study_id, val)).code().color(theme.accent));
                        });
                    }
                }
            }
        }

        AutoTransform::SequenceNumbers { seq_column } => {
            ui.label(RichText::new("Configuration").strong().color(theme.text_muted));
            ui.add_space(spacing::SM);

            egui::Grid::new("seq_detail")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Column").color(theme.text_muted));
                    ui.label(RichText::new(seq_column.as_str()).strong());
                    ui.end_row();

                    ui.label(RichText::new("Group By").color(theme.text_muted));
                    ui.label("USUBJID");
                    ui.end_row();

                    ui.label(RichText::new("Values").color(theme.text_muted));
                    ui.label("1, 2, 3... per subject");
                    ui.end_row();
                });
        }

        AutoTransform::CtNormalization { variable, codelist_code } => {
            // Get source column from mapping
            if let Some((source_col, _)) = mapping_state.get_accepted_for(variable) {
                let samples = MappingService::get_sample_values(source_data, source_col, 5);

                ui.label(RichText::new("Mapping").strong().color(theme.text_muted));
                ui.add_space(spacing::SM);

                egui::Grid::new("ct_detail")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Source").color(theme.text_muted));
                        ui.label(source_col);
                        ui.end_row();

                        ui.label(RichText::new("Target").color(theme.text_muted));
                        ui.label(variable);
                        ui.end_row();

                        ui.label(RichText::new("Codelist").color(theme.text_muted));
                        ui.label(RichText::new(codelist_code.as_str()).color(theme.accent));
                        ui.end_row();
                    });

                if !samples.is_empty() {
                    ui.add_space(spacing::MD);
                    ui.label(RichText::new("Source Values").strong().color(theme.text_muted));
                    ui.add_space(spacing::SM);

                    for val in &samples {
                        ui.label(RichText::new(format!("• {}", val)).code());
                    }
                }
            }
        }
    }
}
