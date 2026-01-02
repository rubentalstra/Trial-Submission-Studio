//! Transform tab
//!
//! Displays SDTM transformations derived from mappings and domain metadata.
//! Transform list is built from the sdtm-transform pipeline using variable metadata.
//! Shows before→after previews for each transformation.

use crate::services::{MappingService, MappingState};
use crate::state::{
    AppState, DomainStatus, TransformRule, TransformRuleDisplay, TransformState, TransformType,
    TransformTypeDisplay, build_pipeline_from_domain,
};
use crate::theme::spacing;
use egui::{RichText, Ui};

use super::mapping::{initialize_mapping, show_loading_indicator};

pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
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
            show_loading_indicator(ui);
            ui.ctx().request_repaint();
            return;
        }
        (false, DomainStatus::Loading) => {
            show_loading_indicator(ui);
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
                    .color(ui.visuals().error_fg_color),
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
        let (rules, selected_idx, generated_count, ct_count, has_subject_id_mapping) = {
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

            let rules = ts.rules();
            let generated_count = ts.generated_count();
            let ct_count = ts.ct_count();
            let has_subject_id_mapping = domain
                .mapping_state
                .as_ref()
                .map(|ms| subject_id_mapping(ms).is_some())
                .unwrap_or(false);

            (
                rules,
                ts.selected_idx,
                generated_count,
                ct_count,
                has_subject_id_mapping,
            )
        };

        // Header
        ui.label(
            RichText::new(format!(
                "{} Transformations",
                egui_phosphor::regular::SHUFFLE
            ))
            .strong(),
        );
        if !rules.is_empty() {
            ui.add_space(spacing::XS);
            ui.label(
                RichText::new(format!("{} generated · {} CT", generated_count, ct_count))
                    .weak()
                    .small(),
            );
        }
        ui.add_space(spacing::SM);
        ui.separator();

        if rules.is_empty() {
            ui.add_space(spacing::LG);
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} No transformations available for this domain",
                        egui_phosphor::regular::INFO
                    ))
                    .weak(),
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
                                rules,
                                selected_idx,
                                has_subject_id_mapping,
                                generated_count,
                                ct_count,
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
                                rules,
                                effective_selection,
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

/// Rebuild transforms from current mapping state using the sdtm-transform pipeline
fn rebuild_transforms_if_needed(state: &mut AppState, domain_code: &str) {
    // Build pipeline from domain metadata
    let pipeline = if let Some(study) = &state.study {
        if let Some(domain) = study.get_domain(domain_code) {
            if let Some(ms) = &domain.mapping_state {
                Some(build_pipeline_from_domain(ms.domain()))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Store the pipeline
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            let selected_idx = domain
                .transform_state
                .as_ref()
                .and_then(|ts| ts.selected_idx);
            domain.transform_state = Some(TransformState {
                pipeline,
                selected_idx,
            });
        }
    }
}

fn show_transform_list(
    ui: &mut Ui,
    rules: &[TransformRule],
    selected_idx: Option<usize>,
    has_subject_id_mapping: bool,
    generated_count: usize,
    ct_count: usize,
) -> Option<usize> {
    let mut new_selection: Option<usize> = None;

    if generated_count > 0 {
        ui.label(
            RichText::new(format!(
                "{} Generated & Derived",
                egui_phosphor::regular::LIGHTNING
            ))
            .strong()
            .weak(),
        );
        ui.add_space(spacing::SM);

        for (idx, rule) in rules.iter().enumerate().filter(|(_, r)| r.is_generated()) {
            let status_suffix = transform_status_suffix(rule, has_subject_id_mapping);
            if render_row(ui, idx, rule, selected_idx == Some(idx), status_suffix) {
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
            .weak(),
        );
        ui.add_space(spacing::SM);

        for (idx, rule) in rules
            .iter()
            .enumerate()
            .filter(|(_, r)| matches!(r.transform_type, TransformType::CtNormalization { .. }))
        {
            if render_row(ui, idx, rule, selected_idx == Some(idx), None) {
                new_selection = Some(idx);
            }
        }
    }

    new_selection
}

fn subject_id_mapping<'a>(ms: &'a MappingState) -> Option<(&'a str, &'static str)> {
    if let Some((col, _)) = ms.accepted("SUBJID") {
        Some((col, "SUBJID"))
    } else if let Some((col, _)) = ms.accepted("USUBJID") {
        Some((col, "Subject ID"))
    } else {
        None
    }
}

fn transform_status_suffix(
    rule: &TransformRule,
    has_subject_id_mapping: bool,
) -> Option<&'static str> {
    match &rule.transform_type {
        TransformType::UsubjidPrefix | TransformType::SequenceNumber => {
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
    rule: &TransformRule,
    is_selected: bool,
    status_suffix: Option<&'static str>,
) -> bool {
    let mut clicked = false;

    ui.horizontal(|ui| {
        ui.label(RichText::new(rule.icon()).color(ui.visuals().hyperlink_color));

        let text = if is_selected {
            RichText::new(&rule.target_variable).strong()
        } else {
            RichText::new(&rule.target_variable)
        };

        if ui.selectable_label(is_selected, text).clicked() {
            clicked = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let right_text = if let Some(status) = status_suffix {
                format!("{} · {}", rule.category(), status)
            } else {
                rule.category().to_string()
            };
            let right_color = if status_suffix.is_some() {
                ui.visuals().warn_fg_color
            } else {
                ui.visuals().weak_text_color()
            };

            ui.label(RichText::new(right_text).color(right_color).small());
        });
    });

    clicked
}

fn show_transform_detail(
    ui: &mut Ui,
    state: &AppState,
    domain_code: &str,
    rules: &[TransformRule],
    selected_idx: Option<usize>,
) {
    let Some(idx) = selected_idx else {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} Select a transformation",
                    egui_phosphor::regular::INFO
                ))
                .weak(),
            );
        });
        return;
    };

    let Some(rule) = rules.get(idx) else {
        return;
    };

    // Get live data from state
    let (study_id, mapping_state, source_data) = {
        let Some(study) = &state.study else { return };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };
        let Some(ms) = &domain.mapping_state else {
            return;
        };
        (study.study_id.as_str(), ms, &domain.source_data)
    };

    // Header
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(rule.icon())
                .size(24.0)
                .color(ui.visuals().hyperlink_color),
        );
        ui.vertical(|ui| {
            ui.heading(&rule.target_variable);
            ui.label(RichText::new(rule.category()).weak().small());
        });
    });

    ui.add_space(spacing::MD);
    ui.separator();
    ui.add_space(spacing::SM);

    // Show details based on transform type
    match &rule.transform_type {
        TransformType::Constant => {
            show_constant_detail(ui, &rule.target_variable, study_id, domain_code);
        }
        TransformType::UsubjidPrefix => {
            show_usubjid_detail(ui, study_id, mapping_state, source_data);
        }
        TransformType::SequenceNumber => {
            show_sequence_detail(ui, &rule.target_variable, mapping_state);
        }
        TransformType::CtNormalization { codelist_code } => {
            show_ct_detail(
                ui,
                &rule.target_variable,
                codelist_code,
                mapping_state,
                source_data,
            );
        }
        TransformType::Iso8601DateTime | TransformType::Iso8601Date => {
            show_datetime_detail(ui, &rule.target_variable, mapping_state, source_data);
        }
        TransformType::Iso8601Duration => {
            show_duration_detail(ui, &rule.target_variable, mapping_state, source_data);
        }
        TransformType::StudyDay { reference_dtc } => {
            show_study_day_detail(ui, &rule.target_variable, reference_dtc);
        }
        TransformType::NumericConversion => {
            show_numeric_detail(ui, &rule.target_variable, mapping_state, source_data);
        }
        TransformType::CopyDirect => {
            show_copy_detail(ui, &rule.target_variable, mapping_state, source_data);
        }
        // Handle future transform types
        _ => {
            ui.label(RichText::new("Transform Details").strong().weak());
            ui.add_space(spacing::SM);
            ui.label(format!(
                "Transform type: {}",
                rule.transform_type.category()
            ));
        }
    }
}

/// Show details for STUDYID/DOMAIN constants
fn show_constant_detail(ui: &mut Ui, target: &str, study_id: &str, domain_code: &str) {
    ui.label(RichText::new("Value Source").strong().weak());
    ui.add_space(spacing::SM);

    let (source_desc, value) = if target == "STUDYID" {
        ("Study configuration", study_id)
    } else if target == "DOMAIN" {
        ("Domain code", domain_code)
    } else {
        ("Constant value", "")
    };

    egui::Grid::new("constant_detail")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Source").weak());
            ui.label(source_desc);
            ui.end_row();

            ui.label(RichText::new("Target").weak());
            ui.label(target);
            ui.end_row();

            ui.label(RichText::new("Value").weak());
            ui.label(RichText::new(value).color(ui.visuals().hyperlink_color));
            ui.end_row();
        });
}

/// Show details for USUBJID derivation
fn show_usubjid_detail(
    ui: &mut Ui,
    study_id: &str,
    mapping_state: &MappingState,
    source_data: &polars::prelude::DataFrame,
) {
    ui.label(RichText::new("Derivation").strong().weak());
    ui.add_space(spacing::SM);

    egui::Grid::new("usubjid_detail")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Target").weak());
            ui.label("USUBJID");
            ui.end_row();

            ui.label(RichText::new("Formula").weak());
            ui.label("STUDYID-SUBJID");
            ui.end_row();
        });

    if let Some((source_col, source_label)) = subject_id_mapping(mapping_state) {
        let samples = MappingService::get_sample_values(source_data, source_col, 3);

        ui.add_space(spacing::MD);
        ui.label(RichText::new("Mapping").strong().weak());
        ui.add_space(spacing::SM);

        egui::Grid::new("usubjid_mapping")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new(source_label).weak());
                ui.label(source_col);
                ui.end_row();

                ui.label(RichText::new("Study ID").weak());
                ui.label(RichText::new(study_id).color(ui.visuals().hyperlink_color));
                ui.end_row();
            });

        if !samples.is_empty() {
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Sample Values").strong().weak());
            ui.add_space(spacing::SM);

            for val in &samples {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(val).code());
                    ui.label(RichText::new("→").weak());
                    ui.label(
                        RichText::new(format!("{}-{}", study_id, val))
                            .code()
                            .color(ui.visuals().hyperlink_color),
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
            .color(ui.visuals().warn_fg_color),
        );
    }
}

/// Show details for sequence number generation
fn show_sequence_detail(ui: &mut Ui, seq_column: &str, mapping_state: &MappingState) {
    ui.label(RichText::new("Configuration").strong().weak());
    ui.add_space(spacing::SM);

    egui::Grid::new("seq_detail")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Column").weak());
            ui.label(RichText::new(seq_column).strong());
            ui.end_row();

            ui.label(RichText::new("Group By").weak());
            ui.label("USUBJID");
            ui.end_row();

            ui.label(RichText::new("Values").weak());
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
            .color(ui.visuals().warn_fg_color),
        );
    }
}

/// Show details for CT normalization with before→after preview
fn show_ct_detail(
    ui: &mut Ui,
    variable: &str,
    codelist_code: &str,
    mapping_state: &MappingState,
    source_data: &polars::prelude::DataFrame,
) {
    if let Some((source_col, _)) = mapping_state.accepted(variable) {
        let samples = MappingService::get_sample_values(source_data, source_col, 5);

        ui.label(RichText::new("Mapping").strong().weak());
        ui.add_space(spacing::SM);

        egui::Grid::new("ct_detail")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Source").weak());
                ui.label(source_col);
                ui.end_row();

                ui.label(RichText::new("Target").weak());
                ui.label(variable);
                ui.end_row();

                ui.label(RichText::new("Codelist").weak());
                ui.label(RichText::new(codelist_code).color(ui.visuals().hyperlink_color));
                ui.end_row();
            });

        if let Some(ct_info) = mapping_state.ct_cache.get(codelist_code) {
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Codelist").strong().weak());
            ui.add_space(spacing::SM);

            if ct_info.found {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(&ct_info.code).weak().small());
                    ui.add(egui::Label::new(RichText::new(&ct_info.name).strong()).wrap());
                    if ct_info.extensible {
                        ui.label(
                            RichText::new("(Extensible)")
                                .color(ui.visuals().warn_fg_color)
                                .small(),
                        );
                    }
                });

                if !ct_info.terms.is_empty() {
                    ui.add_space(spacing::SM);
                    ui.label(
                        RichText::new(format!("Valid values ({}):", ct_info.total_terms))
                            .weak()
                            .small(),
                    );

                    for (idx, (value, def)) in ct_info.terms.iter().enumerate() {
                        ui.vertical(|ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(value)
                                        .strong()
                                        .color(ui.visuals().hyperlink_color),
                                )
                                .wrap(),
                            );
                            if let Some(d) = def {
                                ui.add(egui::Label::new(RichText::new(d).weak().small()).wrap());
                            }
                        });
                        if idx + 1 < ct_info.terms.len() {
                            ui.add_space(spacing::XS);
                        }
                    }

                    if ct_info.total_terms > ct_info.terms.len() {
                        ui.label(
                            RichText::new(format!(
                                "... and {} more values",
                                ct_info.total_terms - ct_info.terms.len()
                            ))
                            .weak()
                            .small()
                            .italics(),
                        );
                    }
                }
            } else {
                ui.label(
                    RichText::new(format!("{} - not found in CT registry", ct_info.code))
                        .color(ui.visuals().warn_fg_color)
                        .small(),
                );
            }
        }

        // Show transformation preview with before→after
        if !samples.is_empty() {
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Transformation Preview").strong().weak());
            ui.add_space(spacing::SM);

            // Get lookup map from cache (includes synonyms → submission_value)
            let lookup = mapping_state
                .ct_cache
                .get(codelist_code)
                .map(|info| &info.lookup);

            for val in &samples {
                // Normalize using the lookup map (handles synonyms properly)
                let normalized = {
                    let trimmed = val.trim();
                    let upper = trimmed.to_uppercase();
                    lookup
                        .and_then(|m| m.get(&upper).cloned())
                        .unwrap_or_else(|| trimmed.to_string())
                };

                let is_changed = val != &normalized;
                ui.horizontal(|ui| {
                    ui.label(RichText::new(val).code());
                    ui.label(RichText::new("→").weak());
                    if is_changed {
                        ui.label(
                            RichText::new(&normalized)
                                .code()
                                .color(ui.visuals().hyperlink_color),
                        );
                    } else {
                        ui.label(RichText::new(&normalized).code().weak());
                        ui.label(RichText::new("(unchanged)").small().weak());
                    }
                });
            }
        }
    }
}

/// Show details for ISO 8601 datetime transformation with before→after preview
fn show_datetime_detail(
    ui: &mut Ui,
    variable: &str,
    mapping_state: &MappingState,
    source_data: &polars::prelude::DataFrame,
) {
    ui.label(RichText::new("ISO 8601 DateTime").strong().weak());
    ui.add_space(spacing::SM);

    egui::Grid::new("datetime_detail")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Target").weak());
            ui.label(variable);
            ui.end_row();

            ui.label(RichText::new("Format").weak());
            ui.label("YYYY-MM-DDTHH:MM:SS");
            ui.end_row();
        });

    if let Some((source_col, _)) = mapping_state.accepted(variable) {
        let samples = MappingService::get_sample_values(source_data, source_col, 3);
        if !samples.is_empty() {
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Transformation Preview").strong().weak());
            ui.add_space(spacing::SM);

            for val in &samples {
                // Try to parse and format as ISO 8601
                let normalized = sdtm_transform::normalization::datetime::parse_date(val)
                    .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
                    .unwrap_or_else(|| val.to_string());

                let is_changed = val != &normalized;
                ui.horizontal(|ui| {
                    ui.label(RichText::new(val).code());
                    ui.label(RichText::new("→").weak());
                    if is_changed {
                        ui.label(
                            RichText::new(&normalized)
                                .code()
                                .color(ui.visuals().hyperlink_color),
                        );
                    } else {
                        ui.label(RichText::new(&normalized).code());
                        ui.label(RichText::new("(already ISO 8601)").small().weak());
                    }
                });
            }
        }
    }
}

/// Show details for ISO 8601 duration transformation with before→after preview
fn show_duration_detail(
    ui: &mut Ui,
    variable: &str,
    mapping_state: &MappingState,
    source_data: &polars::prelude::DataFrame,
) {
    ui.label(RichText::new("ISO 8601 Duration").strong().weak());
    ui.add_space(spacing::SM);

    egui::Grid::new("duration_detail")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Target").weak());
            ui.label(variable);
            ui.end_row();

            ui.label(RichText::new("Format").weak());
            ui.label("PnYnMnDTnHnMnS");
            ui.end_row();
        });

    if let Some((source_col, _)) = mapping_state.accepted(variable) {
        let samples = MappingService::get_sample_values(source_data, source_col, 3);
        if !samples.is_empty() {
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Transformation Preview").strong().weak());
            ui.add_space(spacing::SM);

            for val in &samples {
                // Duration values typically pass through or get formatted
                let normalized = val.trim().to_string();
                let is_duration_format = normalized.starts_with('P')
                    || normalized.contains("day")
                    || normalized.contains("hour");

                ui.horizontal(|ui| {
                    ui.label(RichText::new(val).code());
                    ui.label(RichText::new("→").weak());
                    ui.label(
                        RichText::new(&normalized)
                            .code()
                            .color(ui.visuals().hyperlink_color),
                    );
                    if !is_duration_format {
                        ui.label(
                            RichText::new("(needs formatting)")
                                .small()
                                .color(ui.visuals().warn_fg_color),
                        );
                    }
                });
            }
        }
    }
}

/// Show details for study day calculation
fn show_study_day_detail(ui: &mut Ui, variable: &str, reference_dtc: &str) {
    ui.label(RichText::new("Study Day Calculation").strong().weak());
    ui.add_space(spacing::SM);

    egui::Grid::new("studyday_detail")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Target").weak());
            ui.label(variable);
            ui.end_row();

            ui.label(RichText::new("Reference").weak());
            ui.label(reference_dtc);
            ui.end_row();

            ui.label(RichText::new("Formula").weak());
            ui.label(format!("{} - RFSTDTC + 1 (if after)", reference_dtc));
            ui.end_row();
        });

    ui.add_space(spacing::MD);
    ui.label(
        RichText::new("Per SDTMIG 4.4.4: Study day is relative to RFSTDTC")
            .weak()
            .small(),
    );
}

/// Show details for numeric conversion with before→after preview
fn show_numeric_detail(
    ui: &mut Ui,
    variable: &str,
    mapping_state: &MappingState,
    source_data: &polars::prelude::DataFrame,
) {
    ui.label(RichText::new("Numeric Conversion").strong().weak());
    ui.add_space(spacing::SM);

    egui::Grid::new("numeric_detail")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Target").weak());
            ui.label(variable);
            ui.end_row();

            ui.label(RichText::new("Type").weak());
            ui.label("Float64");
            ui.end_row();
        });

    if let Some((source_col, _)) = mapping_state.accepted(variable) {
        let samples = MappingService::get_sample_values(source_data, source_col, 3);
        if !samples.is_empty() {
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Transformation Preview").strong().weak());
            ui.add_space(spacing::SM);

            for val in &samples {
                // Try to parse as number
                let parsed: Result<f64, _> = val.trim().parse();
                ui.horizontal(|ui| {
                    ui.label(RichText::new(val).code());
                    ui.label(RichText::new("→").weak());
                    match parsed {
                        Ok(num) => {
                            ui.label(
                                RichText::new(format!("{}", num))
                                    .code()
                                    .color(ui.visuals().hyperlink_color),
                            );
                        }
                        Err(_) => {
                            ui.label(
                                RichText::new("null")
                                    .code()
                                    .color(ui.visuals().warn_fg_color),
                            );
                            ui.label(
                                RichText::new("(not a number)")
                                    .small()
                                    .color(ui.visuals().warn_fg_color),
                            );
                        }
                    }
                });
            }
        }
    }
}

/// Show details for direct copy (passthrough) with before→after preview
fn show_copy_detail(
    ui: &mut Ui,
    variable: &str,
    mapping_state: &MappingState,
    source_data: &polars::prelude::DataFrame,
) {
    ui.label(RichText::new("Direct Copy").strong().weak());
    ui.add_space(spacing::SM);

    if let Some((source_col, _)) = mapping_state.accepted(variable) {
        egui::Grid::new("copy_detail")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Source").weak());
                ui.label(source_col);
                ui.end_row();

                ui.label(RichText::new("Target").weak());
                ui.label(variable);
                ui.end_row();
            });

        let samples = MappingService::get_sample_values(source_data, source_col, 3);
        if !samples.is_empty() {
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Transformation Preview").strong().weak());
            ui.add_space(spacing::SM);

            for val in &samples {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(val).code());
                    ui.label(RichText::new("→").weak());
                    ui.label(
                        RichText::new(val)
                            .code()
                            .color(ui.visuals().hyperlink_color),
                    );
                    ui.label(RichText::new("(copied as-is)").small().weak());
                });
            }
        }
    } else {
        ui.label(RichText::new(format!("Target: {}", variable)).weak());
        ui.add_space(spacing::SM);
        ui.label(
            RichText::new(format!(
                "{} No mapping - values will be empty",
                egui_phosphor::regular::WARNING
            ))
            .color(ui.visuals().warn_fg_color),
        );
    }
}
