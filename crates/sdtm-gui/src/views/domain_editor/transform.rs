//! Transform tab
//!
//! Displays SDTM transformations derived from mappings and domain metadata.
//! Transform list is built from mapping state; display data fetched on-the-fly.

use crate::services::{MappingService, MappingState};
use crate::state::{AppState, DomainStatus, TransformRule, TransformState};
use crate::theme::{colors, spacing};
use egui::{RichText, Ui};

use super::mapping::{initialize_mapping, show_loading_indicator};

pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    let theme = colors(state.preferences.dark_mode);

    // Ensure mapping state is initialized so transforms can be derived accurately.
    let (has_mapping_state, status) = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .map(|d| (d.mapping_state.is_some(), d.status))
        .unwrap_or((false, DomainStatus::NotStarted));

    match (has_mapping_state, status) {
        (false, DomainStatus::NotStarted) => {
            if let Some(study) = &mut state.study {
                if let Some(domain) = study.get_domain_mut(domain_code) {
                    domain.status = DomainStatus::Loading;
                }
            }
            show_loading_indicator(ui, &theme);
            ui.ctx().request_repaint();
            return;
        }
        (false, DomainStatus::Loading) => {
            show_loading_indicator(ui, &theme);
            initialize_mapping(state, domain_code);
            ui.ctx().request_repaint();
            return;
        }
        (false, _) => {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} Failed to initialize mapping",
                        egui_phosphor::regular::WARNING
                    ))
                    .color(theme.error),
                );
            });
            return;
        }
        (true, _) => {}
    }

    // Rebuild transforms from current mapping state (cheap operation)
    rebuild_transforms_if_needed(state, domain_code);

    let mut new_selection: Option<usize> = None;

    {
        // Get data for display
        let (transforms, selected_idx, generated_count, ct_count, has_subject_id_mapping) = {
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

            let generated_count = ts.transforms.iter().filter(|t| t.is_generated()).count();
            let ct_count = ts.transforms.len().saturating_sub(generated_count);
            let has_subject_id_mapping = domain
                .mapping_state
                .as_ref()
                .map(|ms| subject_id_mapping(ms).is_some())
                .unwrap_or(false);

            (
                ts.transforms.as_slice(),
                ts.selected_idx,
                generated_count,
                ct_count,
                has_subject_id_mapping,
            )
        };

        // Header
        ui.label(
            RichText::new(format!("{} Transformations", egui_phosphor::regular::SHUFFLE))
                .strong(),
        );
        if !transforms.is_empty() {
            ui.add_space(spacing::XS);
            ui.label(
                RichText::new(format!(
                    "{} generated · {} CT",
                    generated_count, ct_count
                ))
                .color(theme.text_muted)
                .small(),
            );
        }
        ui.add_space(spacing::SM);
        ui.separator();

        if transforms.is_empty() {
            ui.add_space(spacing::LG);
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} No transformations available for this domain",
                        egui_phosphor::regular::INFO
                    ))
                    .color(theme.text_muted),
                );
            });
            return;
        }

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
                            new_selection = show_transform_list(
                                ui,
                                transforms,
                                selected_idx,
                                has_subject_id_mapping,
                                generated_count,
                                ct_count,
                                &theme,
                            );
                        });
                });

                strip.cell(|ui| {
                    ui.separator();
                });

                strip.cell(|ui| {
                    egui::ScrollArea::vertical()
                        .max_height(available_height)
                        .show(ui, |ui| {
                            let effective_selection = new_selection.or(selected_idx);
                            show_transform_detail(
                                ui,
                                state,
                                domain_code,
                                transforms,
                                effective_selection,
                                &theme,
                            );
                        });
                });
            });
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

/// Rebuild transforms from current mapping state
fn rebuild_transforms_if_needed(state: &mut AppState, domain_code: &str) {
    let mut transforms = Vec::new();

    if let Some(study) = &state.study {
        if let Some(domain) = study.get_domain(domain_code) {
            if let Some(ms) = &domain.mapping_state {
                transforms.reserve(ms.sdtm_domain.variables.len() + 4);
                if ms.sdtm_domain.column_name("STUDYID").is_some() {
                    transforms.push(TransformRule::StudyIdConstant);
                }

                if ms.sdtm_domain.column_name("DOMAIN").is_some() {
                    transforms.push(TransformRule::DomainConstant);
                }

                if ms.sdtm_domain.column_name("USUBJID").is_some() {
                    transforms.push(TransformRule::UsubjidDerivation);
                }

                if let Some(seq_column) = ms.sdtm_domain.infer_seq_column() {
                    transforms.push(TransformRule::SequenceNumbers {
                        seq_column: seq_column.to_string(),
                    });
                }

                // CT normalization for each mapped variable with a codelist
                for variable in &ms.sdtm_domain.variables {
                    if let Some(codelist_code) = &variable.codelist_code {
                        if ms.get_accepted_for(&variable.name).is_some() {
                            let code = codelist_code.split(';').next().unwrap_or("").trim();
                            if !code.is_empty() {
                                transforms.push(TransformRule::CtNormalization {
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
    transforms: &[TransformRule],
    selected_idx: Option<usize>,
    has_subject_id_mapping: bool,
    generated_count: usize,
    ct_count: usize,
    theme: &crate::theme::ThemeColors,
) -> Option<usize> {
    let mut new_selection: Option<usize> = None;

    if generated_count > 0 {
        ui.label(
            RichText::new(format!(
                "{} Generated & Derived",
                egui_phosphor::regular::LIGHTNING
            ))
            .strong()
            .color(theme.text_muted),
        );
        ui.add_space(spacing::SM);

        for (idx, t) in transforms.iter().enumerate().filter(|(_, t)| t.is_generated()) {
            let status_suffix = transform_status_suffix(t, has_subject_id_mapping);
            if render_row(ui, idx, t, selected_idx == Some(idx), status_suffix, theme) {
                new_selection = Some(idx);
            }
        }
    }

    if ct_count > 0 {
        if generated_count > 0 {
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

        for (idx, t) in transforms
            .iter()
            .enumerate()
            .filter(|(_, t)| matches!(t, TransformRule::CtNormalization { .. }))
        {
            if render_row(ui, idx, t, selected_idx == Some(idx), None, theme) {
                new_selection = Some(idx);
            }
        }
    }

    new_selection
}

fn subject_id_mapping<'a>(ms: &'a MappingState) -> Option<(&'a str, &'static str)> {
    if let Some((col, _)) = ms.get_accepted_for("SUBJID") {
        Some((col, "SUBJID"))
    } else if let Some((col, _)) = ms.get_accepted_for("USUBJID") {
        Some((col, "Subject ID"))
    } else {
        None
    }
}

fn transform_status_suffix(
    transform: &TransformRule,
    has_subject_id_mapping: bool,
) -> Option<&'static str> {
    match transform {
        TransformRule::UsubjidDerivation => {
            if has_subject_id_mapping {
                None
            } else {
                Some("Needs SUBJID")
            }
        }
        TransformRule::SequenceNumbers { .. } => {
            if has_subject_id_mapping {
                None
            } else {
                Some("Needs SUBJID")
            }
        }
        _ => None,
    }
}

fn render_row(
    ui: &mut Ui,
    _idx: usize,
    transform: &TransformRule,
    is_selected: bool,
    status_suffix: Option<&'static str>,
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
            let right_text = if let Some(status) = status_suffix {
                format!("{} · {}", transform.display_name(), status)
            } else {
                transform.display_name().to_string()
            };
            let right_color = if status_suffix.is_some() {
                theme.warning
            } else {
                theme.text_muted
            };

            ui.label(
                RichText::new(right_text)
                    .color(right_color)
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
    transforms: &[TransformRule],
    selected_idx: Option<usize>,
    theme: &crate::theme::ThemeColors,
) {
    let Some(idx) = selected_idx else {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} Select a transformation",
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
        (study.study_id.as_str(), ms, &domain.source_data)
    };

    // Header
    ui.horizontal(|ui| {
        ui.label(RichText::new(transform.icon()).size(24.0).color(theme.accent));
        ui.vertical(|ui| {
            ui.heading(transform.target_variable());
            ui.label(
                RichText::new(transform.display_name())
                    .color(theme.text_muted)
                    .small(),
            );
        });
    });

    ui.add_space(spacing::MD);
    ui.separator();
    ui.add_space(spacing::SM);

    match transform {
        TransformRule::StudyIdConstant => {
            ui.label(RichText::new("Value Source").strong().color(theme.text_muted));
            ui.add_space(spacing::SM);

            egui::Grid::new("studyid_detail")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Source").color(theme.text_muted));
                    ui.label("Study configuration");
                    ui.end_row();

                    ui.label(RichText::new("Target").color(theme.text_muted));
                    ui.label("STUDYID");
                    ui.end_row();

                    ui.label(RichText::new("Value").color(theme.text_muted));
                    ui.label(RichText::new(study_id).color(theme.accent));
                    ui.end_row();
                });
        }

        TransformRule::DomainConstant => {
            ui.label(RichText::new("Value Source").strong().color(theme.text_muted));
            ui.add_space(spacing::SM);

            egui::Grid::new("domain_detail")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Source").color(theme.text_muted));
                    ui.label("Domain code");
                    ui.end_row();

                    ui.label(RichText::new("Target").color(theme.text_muted));
                    ui.label("DOMAIN");
                    ui.end_row();

                    ui.label(RichText::new("Value").color(theme.text_muted));
                    ui.label(RichText::new(domain_code).color(theme.accent));
                    ui.end_row();
                });
        }

        TransformRule::UsubjidDerivation => {
            ui.label(RichText::new("Derivation").strong().color(theme.text_muted));
            ui.add_space(spacing::SM);

            egui::Grid::new("usubjid_detail")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Target").color(theme.text_muted));
                    ui.label("USUBJID");
                    ui.end_row();

                    ui.label(RichText::new("Formula").color(theme.text_muted));
                    ui.label("STUDYID-SUBJID");
                    ui.end_row();
                });

            if let Some((source_col, source_label)) = subject_id_mapping(mapping_state) {
                let samples = MappingService::get_sample_values(source_data, source_col, 3);

                ui.add_space(spacing::MD);
                ui.label(RichText::new("Mapping").strong().color(theme.text_muted));
                ui.add_space(spacing::SM);

                egui::Grid::new("usubjid_mapping")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new(source_label).color(theme.text_muted));
                        ui.label(source_col);
                        ui.end_row();

                        ui.label(RichText::new("Study ID").color(theme.text_muted));
                        ui.label(RichText::new(study_id).color(theme.accent));
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
                            ui.label(
                                RichText::new(format!("{}-{}", study_id, val))
                                    .code()
                                    .color(theme.accent),
                            );
                        });
                    }
                }
            } else {
                ui.add_space(spacing::MD);
                ui.label(
                    RichText::new(format!(
                        "{} Map the SUBJID column in Mapping to build USUBJID",
                        egui_phosphor::regular::INFO
                    ))
                    .color(theme.warning),
                );
            }
        }

        TransformRule::SequenceNumbers { seq_column } => {
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

            if subject_id_mapping(mapping_state).is_none() {
                ui.add_space(spacing::MD);
                ui.label(
                    RichText::new(format!(
                        "{} Requires SUBJID mapping to derive USUBJID",
                        egui_phosphor::regular::INFO
                    ))
                    .color(theme.warning),
                );
            }
        }

        TransformRule::CtNormalization { variable, codelist_code } => {
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

                if let Some(ct_info) = mapping_state.ct_cache.get(codelist_code) {
                    ui.add_space(spacing::MD);
                    ui.label(RichText::new("Codelist").strong().color(theme.text_muted));
                    ui.add_space(spacing::SM);

                    if ct_info.found {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&ct_info.code).color(theme.text_muted).small());
                            ui.label(RichText::new(&ct_info.name).strong());
                            if ct_info.extensible {
                                ui.label(
                                    RichText::new("(Extensible)")
                                        .color(theme.warning)
                                        .small(),
                                );
                            }
                        });

                        if !ct_info.terms.is_empty() {
                            ui.add_space(spacing::SM);
                            ui.label(
                                RichText::new(format!(
                                    "Valid values ({}):",
                                    ct_info.total_terms
                                ))
                                .color(theme.text_muted)
                                .small(),
                            );

                            for (value, def) in &ct_info.terms {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(value).strong().color(theme.accent));
                                    if let Some(d) = def {
                                        ui.label(
                                            RichText::new(d).color(theme.text_secondary).small(),
                                        );
                                    }
                                });
                            }

                            if ct_info.total_terms > ct_info.terms.len() {
                                ui.label(
                                    RichText::new(format!(
                                        "... and {} more values",
                                        ct_info.total_terms - ct_info.terms.len()
                                    ))
                                    .color(theme.text_muted)
                                    .small()
                                    .italics(),
                                );
                            }
                        }
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "{} - not found in CT registry",
                                ct_info.code
                            ))
                            .color(theme.warning)
                            .small(),
                        );
                    }
                }

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
