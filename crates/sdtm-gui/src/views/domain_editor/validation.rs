//! Validation tab - Displays validation issues from current mapping state

use crate::state::AppState;
use crate::theme::spacing;
use egui::{Color32, RichText, Ui};
use sdtm_standards::load_default_ct_registry;
use sdtm_transform::build_preview_dataframe_with_omitted;
use sdtm_validate::{Issue, Severity, validate_domain_with_not_collected};
use std::collections::{BTreeMap, BTreeSet};

use super::ensure_mapping_initialized;

pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    if !ensure_mapping_initialized(ui, state, domain_code) {
        return;
    }

    rebuild_validation_if_needed(state, domain_code);

    let Some(study) = &state.study else { return };
    let Some(domain) = study.get_domain(domain_code) else {
        return;
    };

    let (issues, selected_idx, error_count, warning_count) = match &domain.validation {
        Some(report) => (
            report.issues.as_slice(),
            domain.validation_selected_idx,
            report.error_count(None),
            report.warning_count(None),
        ),
        None => (&[][..], None, 0, 0),
    };

    // Header
    ui.label(
        RichText::new(format!(
            "{} Validation Issues",
            egui_phosphor::regular::CHECK_SQUARE
        ))
        .strong(),
    );
    ui.add_space(spacing::XS);
    show_summary(ui, error_count, warning_count);
    ui.add_space(spacing::SM);
    ui.separator();

    if issues.is_empty() {
        ui.add_space(spacing::LG);
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} No validation issues",
                    egui_phosphor::regular::CHECK_CIRCLE
                ))
                .color(Color32::GREEN),
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
                        new_selection = show_list(ui, issues, selected_idx);
                    });
            });
            strip.cell(|ui| {
                ui.separator();
            });
            strip.cell(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        show_detail(
                            ui,
                            new_selection.or(selected_idx).and_then(|i| issues.get(i)),
                        );
                    });
            });
        });

    if let Some(idx) = new_selection {
        if let Some(study) = &mut state.study {
            if let Some(domain) = study.get_domain_mut(domain_code) {
                domain.validation_selected_idx = Some(idx);
            }
        }
    }
}

fn rebuild_validation_if_needed(state: &mut AppState, domain_code: &str) {
    let needs_rebuild = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .map(|d| d.mapping_state.is_some() && d.validation.is_none())
        .unwrap_or(false);

    if !needs_rebuild {
        return;
    }

    let data = {
        let Some(study) = &state.study else { return };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };
        let Some(ms) = &domain.mapping_state else {
            return;
        };

        let accepted: BTreeMap<String, String> = ms
            .all_accepted()
            .iter()
            .map(|(var, (col, _))| (var.clone(), col.clone()))
            .collect();
        let omitted: BTreeSet<String> = ms.all_omitted().clone();
        let not_collected: BTreeSet<String> = ms.all_not_collected().keys().cloned().collect();

        Some((
            ms.domain().clone(),
            ms.study_id().to_string(),
            domain.source_data.clone(),
            accepted,
            omitted,
            not_collected,
        ))
    };

    let Some((sdtm_domain, study_id, source_df, accepted, omitted, not_collected)) = data else {
        return;
    };

    let ct = load_default_ct_registry().ok();
    let preview_df = build_preview_dataframe_with_omitted(
        &source_df,
        &accepted,
        &omitted,
        &sdtm_domain,
        &study_id,
        ct.as_ref(),
    )
    .unwrap_or(source_df);
    let report =
        validate_domain_with_not_collected(&sdtm_domain, &preview_df, ct.as_ref(), &not_collected);

    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            domain.validation = Some(report);
        }
    }
}

fn show_summary(ui: &mut Ui, errors: usize, warnings: usize) {
    ui.horizontal(|ui| {
        if errors > 0 {
            ui.label(
                RichText::new(format!(
                    "{} {} Error{}",
                    egui_phosphor::regular::X_CIRCLE,
                    errors,
                    if errors == 1 { "" } else { "s" }
                ))
                .color(ui.visuals().error_fg_color),
            );
        }
        if errors > 0 && warnings > 0 {
            ui.label(RichText::new(" · ").weak());
        }
        if warnings > 0 {
            ui.label(
                RichText::new(format!(
                    "{} {} Warning{}",
                    egui_phosphor::regular::WARNING,
                    warnings,
                    if warnings == 1 { "" } else { "s" }
                ))
                .color(ui.visuals().warn_fg_color),
            );
        }
        if errors == 0 && warnings == 0 {
            ui.label(
                RichText::new(format!(
                    "{} No issues",
                    egui_phosphor::regular::CHECK_CIRCLE
                ))
                .color(Color32::GREEN),
            );
        }
    });
}

fn show_list(ui: &mut Ui, issues: &[Issue], selected: Option<usize>) -> Option<usize> {
    let mut selection = None;
    let errors: Vec<_> = issues
        .iter()
        .enumerate()
        .filter(|(_, i)| matches!(i.default_severity(), Severity::Error | Severity::Reject))
        .collect();
    let warnings: Vec<_> = issues
        .iter()
        .enumerate()
        .filter(|(_, i)| matches!(i.default_severity(), Severity::Warning))
        .collect();

    if !errors.is_empty() {
        ui.label(
            RichText::new(format!("Errors ({})", errors.len()))
                .color(ui.visuals().error_fg_color)
                .strong(),
        );
        ui.add_space(spacing::XS);
        for (idx, issue) in errors {
            if show_row(ui, idx, issue, selected) {
                selection = Some(idx);
            }
        }
        ui.add_space(spacing::MD);
    }

    if !warnings.is_empty() {
        ui.label(
            RichText::new(format!("Warnings ({})", warnings.len()))
                .color(ui.visuals().warn_fg_color)
                .strong(),
        );
        ui.add_space(spacing::XS);
        for (idx, issue) in warnings {
            if show_row(ui, idx, issue, selected) {
                selection = Some(idx);
            }
        }
    }

    selection
}

fn show_row(ui: &mut Ui, idx: usize, issue: &Issue, selected: Option<usize>) -> bool {
    let is_selected = selected == Some(idx);
    let severity = issue.default_severity();
    let (icon, color) = match severity {
        Severity::Error | Severity::Reject => (
            egui_phosphor::regular::X_CIRCLE,
            ui.visuals().error_fg_color,
        ),
        Severity::Warning => (egui_phosphor::regular::WARNING, ui.visuals().warn_fg_color),
    };

    let frame = egui::Frame::new()
        .fill(if is_selected {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            egui::Color32::TRANSPARENT
        })
        .inner_margin(egui::Margin::symmetric(
            spacing::SM as i8,
            spacing::XS as i8,
        ))
        .corner_radius(4.0);

    let response = frame
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).color(color));
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(issue.variable()).strong());
                        ui.label(RichText::new(issue.rule_id()).small().monospace().weak());
                    });
                    ui.label(RichText::new(check_label(issue)).small().weak());
                });
            });
        })
        .response;

    response.interact(egui::Sense::click()).clicked()
}

fn show_detail(ui: &mut Ui, issue: Option<&Issue>) {
    let Some(issue) = issue else {
        ui.centered_and_justified(|ui| ui.label(RichText::new("Select an issue").weak()));
        return;
    };

    let severity = issue.default_severity();
    let (icon, _) = match severity {
        Severity::Error | Severity::Reject => (
            egui_phosphor::regular::X_CIRCLE,
            ui.visuals().error_fg_color,
        ),
        Severity::Warning => (egui_phosphor::regular::WARNING, ui.visuals().warn_fg_color),
    };

    // Header
    ui.label(
        RichText::new(format!(
            "{} {} — {}",
            icon,
            issue.variable(),
            severity.label()
        ))
        .strong()
        .size(16.0),
    );
    ui.add_space(spacing::XS);

    // Rule ID badge
    egui::Frame::new()
        .fill(ui.visuals().selection.bg_fill)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .corner_radius(4.0)
        .show(ui, |ui| {
            ui.label(
                RichText::new(issue.rule_id())
                    .monospace()
                    .strong()
                    .color(ui.visuals().hyperlink_color),
            );
        });

    ui.add_space(spacing::XS);
    ui.label(
        RichText::new(format!(
            "{} {}",
            egui_phosphor::regular::TAG,
            check_label(issue)
        ))
        .small()
        .weak(),
    );
    ui.add_space(spacing::MD);

    // Context
    show_context(ui, issue);
    ui.add_space(spacing::MD);

    // Message
    ui.label(issue.message(None));
    ui.add_space(spacing::MD);

    // Details
    show_specifics(ui, issue);
}

fn check_label(issue: &Issue) -> &'static str {
    match issue {
        Issue::RequiredMissing { .. } => "Required Variable Missing",
        Issue::RequiredEmpty { .. } => "Required Variable Empty",
        Issue::ExpectedMissing { .. } => "Expected Variable Missing",
        Issue::IdentifierNull { .. } => "Identifier Null",
        Issue::InvalidDate { .. } => "Invalid Date Format",
        Issue::TextTooLong { .. } => "Text Length Exceeded",
        Issue::DataTypeMismatch { .. } => "Data Type Mismatch",
        Issue::DuplicateSequence { .. } => "Duplicate Sequence",
        Issue::CtViolation { .. } => "Controlled Terminology",
    }
}

fn show_context(ui: &mut Ui, issue: &Issue) {
    match issue {
        Issue::CtViolation {
            codelist_name,
            extensible,
            ..
        } => {
            context_frame(
                ui,
                egui_phosphor::regular::LIST_CHECKS,
                ui.visuals().hyperlink_color,
                &format!("Codelist: {}", codelist_name),
                if *extensible {
                    "Extensible codelist - values flagged but allowed"
                } else {
                    "Non-extensible - invalid values block export"
                },
            );
        }
        Issue::RequiredMissing { .. } => {
            context_frame(
                ui,
                egui_phosphor::regular::INFO,
                ui.visuals().hyperlink_color,
                "Required variables (Req) must be present",
                "SDTMIG 4.1",
            );
        }
        Issue::RequiredEmpty { .. } => {
            context_frame(
                ui,
                egui_phosphor::regular::INFO,
                ui.visuals().hyperlink_color,
                "Required variables must have values for all records",
                "SDTMIG 4.1",
            );
        }
        Issue::ExpectedMissing { .. } => {
            context_frame(
                ui,
                egui_phosphor::regular::INFO,
                ui.visuals().warn_fg_color,
                "Expected variables should be included when applicable",
                "SDTMIG 4.1",
            );
        }
        Issue::InvalidDate { .. } => {
            context_frame(
                ui,
                egui_phosphor::regular::CALENDAR,
                ui.visuals().error_fg_color,
                "Date/time values must use ISO 8601 format",
                "YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS",
            );
        }
        Issue::DataTypeMismatch { .. } => {
            context_frame(
                ui,
                egui_phosphor::regular::INFO,
                ui.visuals().error_fg_color,
                "Numeric variables must contain valid numeric data",
                "SDTMIG 2.4",
            );
        }
        Issue::DuplicateSequence { .. } => {
            context_frame(
                ui,
                egui_phosphor::regular::HASH,
                ui.visuals().error_fg_color,
                "Sequence numbers must be unique per subject",
                "SDTMIG 4.1.5",
            );
        }
        Issue::TextTooLong { .. } => {
            context_frame(
                ui,
                egui_phosphor::regular::TEXT_AA,
                ui.visuals().warn_fg_color,
                "Character value exceeds maximum length",
                "Values may be truncated",
            );
        }
        Issue::IdentifierNull { .. } => {
            context_frame(
                ui,
                egui_phosphor::regular::IDENTIFICATION_CARD,
                ui.visuals().error_fg_color,
                "Identifier variables must not contain null values",
                "SDTMIG 4.1.2",
            );
        }
    }
}

fn context_frame(ui: &mut Ui, icon: &str, color: egui::Color32, main: &str, sub: &str) {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .inner_margin(spacing::SM as f32)
        .corner_radius(4.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).color(color));
                ui.label(main);
            });
            ui.add_space(spacing::XS);
            ui.label(RichText::new(sub).small().weak());
        });
}

fn show_specifics(ui: &mut Ui, issue: &Issue) {
    match issue {
        Issue::InvalidDate { samples, .. } | Issue::DataTypeMismatch { samples, .. } => {
            if !samples.is_empty() {
                ui.label(
                    RichText::new(format!(
                        "{} Invalid Values",
                        egui_phosphor::regular::LIST_DASHES
                    ))
                    .strong(),
                );
                ui.add_space(spacing::XS);
                value_list(ui, samples, ui.visuals().error_fg_color);
            }
        }
        Issue::CtViolation {
            invalid_values,
            allowed_count,
            ..
        } => {
            if !invalid_values.is_empty() {
                ui.label(
                    RichText::new(format!(
                        "{} Invalid Values",
                        egui_phosphor::regular::LIST_DASHES
                    ))
                    .strong(),
                );
                ui.add_space(spacing::XS);
                value_list(ui, invalid_values, ui.visuals().error_fg_color);
            }
            ui.add_space(spacing::MD);
            ui.label(
                RichText::new(format!(
                    "{} {} allowed values",
                    egui_phosphor::regular::LIST_CHECKS,
                    allowed_count
                ))
                .weak(),
            );
        }
        Issue::TextTooLong {
            max_found,
            max_allowed,
            ..
        } => {
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Max allowed:").weak());
                        ui.label(RichText::new(format!("{} chars", max_allowed)).strong());
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Max found:").weak());
                        ui.label(
                            RichText::new(format!("{} chars", max_found))
                                .strong()
                                .color(ui.visuals().error_fg_color),
                        );
                    });
                });
        }
        _ => {}
    }
}

fn value_list(ui: &mut Ui, values: &[String], color: egui::Color32) {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .inner_margin(spacing::SM as f32)
        .corner_radius(4.0)
        .show(ui, |ui| {
            for val in values.iter().take(10) {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("•").color(color));
                    ui.label(RichText::new(format!("\"{}\"", val)).monospace());
                });
            }
            if values.len() > 10 {
                ui.label(
                    RichText::new(format!("... and {} more", values.len() - 10))
                        .small()
                        .weak(),
                );
            }
        });
}
