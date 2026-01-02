//! Transform tab - Displays SDTM transformations derived from mappings

use crate::services::{MappingService, MappingState};
use crate::state::{
    AppState, TransformRule, TransformRuleDisplay, TransformState, TransformType,
    TransformTypeDisplay, build_pipeline_from_domain,
};
use crate::theme::spacing;
use egui::{RichText, Ui};

use super::ensure_mapping_initialized;

pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    if !ensure_mapping_initialized(ui, state, domain_code) {
        return;
    }

    rebuild_transforms_if_needed(state, domain_code);

    let Some(study) = &state.study else { return };
    let Some(domain) = study.get_domain(domain_code) else {
        return;
    };
    let Some(ts) = &domain.transform_state else {
        return;
    };
    let Some(ms) = &domain.mapping_state else {
        return;
    };

    let rules = ts.rules();
    let selected_idx = ts.selected_idx;
    let has_subjid = ms.accepted("SUBJID").is_some() || ms.accepted("USUBJID").is_some();

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
            RichText::new(format!(
                "{} generated · {} CT",
                ts.generated_count(),
                ts.ct_count()
            ))
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
                    "{} No transformations",
                    egui_phosphor::regular::INFO
                ))
                .weak(),
            );
        });
        return;
    }

    let mut new_selection: Option<usize> = None;
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
                        new_selection = show_list(ui, rules, selected_idx, has_subjid)
                    });
            });
            strip.cell(|ui| {
                ui.separator();
            });
            strip.cell(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        let sel = new_selection.or(selected_idx);
                        show_detail(ui, state, domain_code, rules, sel);
                    });
            });
        });

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

fn rebuild_transforms_if_needed(state: &mut AppState, domain_code: &str) {
    // Get the full pipeline and mapping state to filter
    let pipeline_and_excluded = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .and_then(|d| d.mapping_state.as_ref())
        .map(|ms| {
            let pipeline = build_pipeline_from_domain(ms.domain());
            // Collect variables that are omitted or not collected
            let excluded: std::collections::BTreeSet<String> = ms
                .all_omitted()
                .iter()
                .cloned()
                .chain(ms.all_not_collected().keys().cloned())
                .collect();
            (pipeline, excluded)
        });

    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            let selected = domain
                .transform_state
                .as_ref()
                .and_then(|ts| ts.selected_idx);

            // Filter the pipeline to exclude omitted/not collected variables
            let filtered_pipeline = pipeline_and_excluded.map(|(pipeline, excluded)| {
                let filtered_rules: Vec<_> = pipeline
                    .rules
                    .into_iter()
                    .filter(|rule| !excluded.contains(&rule.target_variable))
                    .collect();
                sdtm_transform::DomainPipeline {
                    domain_code: pipeline.domain_code,
                    study_id: pipeline.study_id,
                    rules: filtered_rules,
                }
            });

            domain.transform_state = Some(TransformState {
                pipeline: filtered_pipeline,
                selected_idx: selected,
            });
        }
    }
}

fn show_list(
    ui: &mut Ui,
    rules: &[TransformRule],
    selected_idx: Option<usize>,
    has_subjid: bool,
) -> Option<usize> {
    let mut selection: Option<usize> = None;

    // Generated section
    let generated: Vec<_> = rules
        .iter()
        .enumerate()
        .filter(|(_, r)| r.is_generated())
        .collect();
    if !generated.is_empty() {
        ui.label(
            RichText::new(format!("{} Generated", egui_phosphor::regular::LIGHTNING))
                .strong()
                .weak(),
        );
        ui.add_space(spacing::SM);
        for (idx, rule) in &generated {
            let status = match &rule.transform_type {
                TransformType::UsubjidPrefix | TransformType::SequenceNumber if !has_subjid => {
                    Some("Needs SUBJID")
                }
                _ => None,
            };
            if show_row(ui, rule, selected_idx == Some(*idx), status) {
                selection = Some(*idx);
            }
        }
    }

    // CT section
    let ct: Vec<_> = rules
        .iter()
        .enumerate()
        .filter(|(_, r)| matches!(r.transform_type, TransformType::CtNormalization { .. }))
        .collect();
    if !ct.is_empty() {
        if !generated.is_empty() {
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
        for (idx, rule) in ct {
            if show_row(ui, rule, selected_idx == Some(idx), None) {
                selection = Some(idx);
            }
        }
    }

    selection
}

fn show_row(ui: &mut Ui, rule: &TransformRule, is_selected: bool, status: Option<&str>) -> bool {
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
            let label = if let Some(s) = status {
                format!("{} · {}", rule.category(), s)
            } else {
                rule.category().to_string()
            };
            let color = if status.is_some() {
                ui.visuals().warn_fg_color
            } else {
                ui.visuals().weak_text_color()
            };
            ui.label(RichText::new(label).color(color).small());
        });
    });
    clicked
}

fn show_detail(
    ui: &mut Ui,
    state: &AppState,
    domain_code: &str,
    rules: &[TransformRule],
    selected: Option<usize>,
) {
    let Some(idx) = selected else {
        ui.centered_and_justified(|ui| ui.label(RichText::new("Select a transformation").weak()));
        return;
    };
    let Some(rule) = rules.get(idx) else { return };
    let Some(study) = &state.study else { return };
    let Some(domain) = study.get_domain(domain_code) else {
        return;
    };
    let Some(ms) = &domain.mapping_state else {
        return;
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

    match &rule.transform_type {
        TransformType::Constant => {
            show_constant(ui, &rule.target_variable, &study.study_id, domain_code);
        }
        TransformType::UsubjidPrefix => {
            show_usubjid(ui, &study.study_id, ms, &domain.source_data);
        }
        TransformType::SequenceNumber => {
            show_sequence(ui, &rule.target_variable, ms);
        }
        TransformType::CtNormalization { codelist_code } => {
            show_ct(
                ui,
                &rule.target_variable,
                codelist_code,
                ms,
                &domain.source_data,
            );
        }
        TransformType::Iso8601DateTime | TransformType::Iso8601Date => {
            show_datetime(ui, &rule.target_variable, ms, &domain.source_data);
        }
        TransformType::CopyDirect | TransformType::NumericConversion => {
            show_copy(ui, &rule.target_variable, ms, &domain.source_data);
        }
        _ => {
            ui.label(format!("Transform: {}", rule.transform_type.category()));
        }
    }
}

fn show_constant(ui: &mut Ui, target: &str, study_id: &str, domain_code: &str) {
    ui.label(RichText::new("Value Source").strong().weak());
    ui.add_space(spacing::SM);
    let (source, value) = match target {
        "STUDYID" => ("Study configuration", study_id),
        "DOMAIN" => ("Domain code", domain_code),
        _ => ("Constant", ""),
    };
    egui::Grid::new("const")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Source").weak());
            ui.label(source);
            ui.end_row();
            ui.label(RichText::new("Value").weak());
            ui.label(RichText::new(value).color(ui.visuals().hyperlink_color));
            ui.end_row();
        });
}

fn show_usubjid(
    ui: &mut Ui,
    study_id: &str,
    ms: &MappingState,
    source_data: &polars::prelude::DataFrame,
) {
    ui.label(RichText::new("Derivation").strong().weak());
    ui.add_space(spacing::SM);
    egui::Grid::new("usubjid")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Formula").weak());
            ui.label("STUDYID-SUBJID");
            ui.end_row();
        });

    if let Some((col, _)) = ms.accepted("SUBJID") {
        let samples = MappingService::get_sample_values(source_data, col, 3);
        ui.add_space(spacing::MD);
        ui.label(RichText::new(format!("SUBJID → {}", col)).weak().small());
        for val in samples {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&val).code());
                ui.label(RichText::new("→").weak());
                ui.label(
                    RichText::new(format!("{}-{}", study_id, val))
                        .code()
                        .color(ui.visuals().hyperlink_color),
                );
            });
        }
    } else {
        ui.add_space(spacing::MD);
        ui.label(
            RichText::new(format!("{} Map SUBJID first", egui_phosphor::regular::INFO))
                .color(ui.visuals().warn_fg_color),
        );
    }
}

fn show_sequence(ui: &mut Ui, seq_col: &str, ms: &MappingState) {
    ui.label(RichText::new("Configuration").strong().weak());
    ui.add_space(spacing::SM);
    egui::Grid::new("seq")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Column").weak());
            ui.label(seq_col);
            ui.end_row();
            ui.label(RichText::new("Group By").weak());
            ui.label("USUBJID");
            ui.end_row();
            ui.label(RichText::new("Values").weak());
            ui.label("1, 2, 3... per subject");
            ui.end_row();
        });
    if ms.accepted("SUBJID").is_none() {
        ui.add_space(spacing::MD);
        ui.label(
            RichText::new(format!(
                "{} Requires SUBJID mapping",
                egui_phosphor::regular::INFO
            ))
            .color(ui.visuals().warn_fg_color),
        );
    }
}

fn show_ct(
    ui: &mut Ui,
    variable: &str,
    codelist: &str,
    ms: &MappingState,
    source_data: &polars::prelude::DataFrame,
) {
    if let Some((col, _)) = ms.accepted(variable) {
        egui::Grid::new("ct_map")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Source").weak());
                ui.label(col);
                ui.end_row();
                ui.label(RichText::new("Codelist").weak());
                ui.label(RichText::new(codelist).color(ui.visuals().hyperlink_color));
                ui.end_row();
            });

        if let Some(info) = ms.ct_cache.get(codelist) {
            if info.found {
                ui.add_space(spacing::MD);
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(&info.name).strong());
                    if info.extensible {
                        ui.label(
                            RichText::new("(Extensible)")
                                .color(ui.visuals().warn_fg_color)
                                .small(),
                        );
                    }
                });
                ui.label(
                    RichText::new(format!("{} terms", info.total_terms))
                        .weak()
                        .small(),
                );
            }
        }

        // Preview
        let samples = MappingService::get_sample_values(source_data, col, 4);
        if !samples.is_empty() {
            let lookup = ms.ct_cache.get(codelist).map(|i| &i.lookup);
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Preview").strong().weak());
            ui.add_space(spacing::SM);
            for val in samples {
                let normalized = lookup
                    .and_then(|m| m.get(&val.trim().to_uppercase()).cloned())
                    .unwrap_or_else(|| val.trim().to_string());
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&val).code());
                    ui.label(RichText::new("→").weak());
                    let color = if val.trim() != normalized {
                        ui.visuals().hyperlink_color
                    } else {
                        ui.visuals().text_color()
                    };
                    ui.label(RichText::new(&normalized).code().color(color));
                });
            }
        }
    }
}

fn show_datetime(
    ui: &mut Ui,
    variable: &str,
    ms: &MappingState,
    source_data: &polars::prelude::DataFrame,
) {
    ui.label(RichText::new("ISO 8601 DateTime").strong().weak());
    ui.add_space(spacing::SM);
    egui::Grid::new("dt")
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

    if let Some((col, _)) = ms.accepted(variable) {
        let samples = MappingService::get_sample_values(source_data, col, 3);
        if !samples.is_empty() {
            ui.add_space(spacing::MD);
            ui.label(RichText::new("Preview").strong().weak());
            ui.add_space(spacing::SM);
            for val in samples {
                let normalized = sdtm_transform::normalization::datetime::parse_date(&val)
                    .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
                    .unwrap_or_else(|| val.to_string());
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&val).code());
                    ui.label(RichText::new("→").weak());
                    ui.label(
                        RichText::new(&normalized)
                            .code()
                            .color(ui.visuals().hyperlink_color),
                    );
                });
            }
        }
    }
}

fn show_copy(
    ui: &mut Ui,
    variable: &str,
    ms: &MappingState,
    source_data: &polars::prelude::DataFrame,
) {
    ui.label(RichText::new("Direct Copy").strong().weak());
    ui.add_space(spacing::SM);
    if let Some((col, _)) = ms.accepted(variable) {
        egui::Grid::new("copy")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Source").weak());
                ui.label(col);
                ui.end_row();
                ui.label(RichText::new("Target").weak());
                ui.label(variable);
                ui.end_row();
            });
        let samples = MappingService::get_sample_values(source_data, col, 3);
        if !samples.is_empty() {
            ui.add_space(spacing::MD);
            for val in samples {
                ui.label(RichText::new(&val).code());
            }
        }
    } else {
        ui.label(
            RichText::new(format!("{} No mapping", egui_phosphor::regular::WARNING))
                .color(ui.visuals().warn_fg_color),
        );
    }
}
