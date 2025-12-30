//! Validation tab
//!
//! Displays CT validation issues derived from current mapping state.
//! Read-only display - issues are shown for user awareness, not resolution.

use crate::state::{AppState, DomainStatus};
use crate::theme::{ThemeColors, colors, spacing};
use egui::{RichText, Ui};
use sdtm_model::{Severity, ValidationIssue};
use sdtm_standards::load_default_ct_registry;
use sdtm_validate::validate_domain;

use super::mapping::{initialize_mapping, show_loading_indicator};

/// Render the validation tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    let theme = colors(state.preferences.dark_mode);

    // Ensure mapping state is initialized so validation can run
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

    // Rebuild validation from current mapping state
    rebuild_validation_if_needed(state, domain_code);

    let mut new_selection: Option<usize> = None;

    {
        // Get data for display
        let (issues, selected_idx, error_count, warning_count) = {
            let Some(study) = &state.study else {
                ui.label("No study loaded");
                return;
            };
            let Some(domain) = study.get_domain(domain_code) else {
                ui.label("Domain not found");
                return;
            };

            let (issues, error_count, warning_count) = match &domain.validation {
                Some(report) => (
                    report.issues.as_slice(),
                    report.error_count(),
                    report.warning_count(),
                ),
                None => (&[][..], 0, 0),
            };

            (
                issues,
                domain.validation_selected_idx,
                error_count,
                warning_count,
            )
        };

        // Header
        ui.label(
            RichText::new(format!(
                "{} Validation Issues",
                egui_phosphor::regular::CHECK_SQUARE
            ))
            .strong(),
        );

        // Summary bar
        ui.add_space(spacing::XS);
        show_summary_bar(ui, error_count, warning_count, &theme);
        ui.add_space(spacing::SM);
        ui.separator();

        if issues.is_empty() {
            ui.add_space(spacing::LG);
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} No validation issues found",
                        egui_phosphor::regular::CHECK_CIRCLE
                    ))
                    .color(theme.success),
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
                            new_selection = show_issue_list(ui, issues, selected_idx, &theme);
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
                            let selected_issue =
                                effective_selection.and_then(|idx| issues.get(idx));
                            show_issue_detail(ui, selected_issue, &theme);
                        });
                });
            });
    }

    // Update selection state
    if let Some(idx) = new_selection {
        if let Some(study) = &mut state.study {
            if let Some(domain) = study.get_domain_mut(domain_code) {
                domain.validation_selected_idx = Some(idx);
            }
        }
    }
}

/// Rebuild validation from current mapping state
fn rebuild_validation_if_needed(state: &mut AppState, domain_code: &str) {
    // Check if we need to rebuild
    let needs_rebuild = state
        .study
        .as_ref()
        .and_then(|s| s.get_domain(domain_code))
        .map(|d| d.mapping_state.is_some() && d.validation.is_none())
        .unwrap_or(false);

    if !needs_rebuild {
        return;
    }

    // Get the SDTM domain definition and source data
    let validation_data = {
        let Some(study) = &state.study else { return };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };
        let Some(ms) = &domain.mapping_state else {
            return;
        };

        // Clone what we need for validation
        Some((ms.sdtm_domain.clone(), domain.source_data.clone()))
    };

    let Some((sdtm_domain, source_df)) = validation_data else {
        return;
    };

    // Load CT registry and run validation
    let ct_registry = load_default_ct_registry().ok();
    let report = validate_domain(&sdtm_domain, &source_df, ct_registry.as_ref());

    // Store result
    if let Some(study) = &mut state.study {
        if let Some(domain) = study.get_domain_mut(domain_code) {
            domain.validation = Some(report);
        }
    }
}

/// Show summary bar with error/warning counts
fn show_summary_bar(ui: &mut Ui, error_count: usize, warning_count: usize, theme: &ThemeColors) {
    ui.horizontal(|ui| {
        if error_count > 0 {
            ui.label(
                RichText::new(format!(
                    "{} {} Error{}",
                    egui_phosphor::regular::X_CIRCLE,
                    error_count,
                    if error_count == 1 { "" } else { "s" }
                ))
                .color(theme.error),
            );
        }

        if error_count > 0 && warning_count > 0 {
            ui.label(RichText::new(" · ").color(theme.text_muted));
        }

        if warning_count > 0 {
            ui.label(
                RichText::new(format!(
                    "{} {} Warning{}",
                    egui_phosphor::regular::WARNING,
                    warning_count,
                    if warning_count == 1 { "" } else { "s" }
                ))
                .color(theme.warning),
            );
        }

        if error_count == 0 && warning_count == 0 {
            ui.label(
                RichText::new(format!(
                    "{} No issues",
                    egui_phosphor::regular::CHECK_CIRCLE
                ))
                .color(theme.success),
            );
        }
    });
}

/// Show list of validation issues grouped by severity
fn show_issue_list(
    ui: &mut Ui,
    issues: &[ValidationIssue],
    selected_idx: Option<usize>,
    theme: &ThemeColors,
) -> Option<usize> {
    let mut new_selection = None;

    // Separate errors and warnings
    let errors: Vec<(usize, &ValidationIssue)> = issues
        .iter()
        .enumerate()
        .filter(|(_, i)| matches!(i.severity, Severity::Error | Severity::Reject))
        .collect();

    let warnings: Vec<(usize, &ValidationIssue)> = issues
        .iter()
        .enumerate()
        .filter(|(_, i)| matches!(i.severity, Severity::Warning))
        .collect();

    // Errors section
    if !errors.is_empty() {
        ui.label(
            RichText::new(format!("Errors ({})", errors.len()))
                .color(theme.error)
                .strong(),
        );
        ui.add_space(spacing::XS);

        for (idx, issue) in errors {
            if show_issue_row(ui, idx, issue, selected_idx, theme) {
                new_selection = Some(idx);
            }
        }

        ui.add_space(spacing::MD);
    }

    // Warnings section
    if !warnings.is_empty() {
        ui.label(
            RichText::new(format!("Warnings ({})", warnings.len()))
                .color(theme.warning)
                .strong(),
        );
        ui.add_space(spacing::XS);

        for (idx, issue) in warnings {
            if show_issue_row(ui, idx, issue, selected_idx, theme) {
                new_selection = Some(idx);
            }
        }
    }

    new_selection
}

/// Show a single issue row in the list
fn show_issue_row(
    ui: &mut Ui,
    idx: usize,
    issue: &ValidationIssue,
    selected_idx: Option<usize>,
    theme: &ThemeColors,
) -> bool {
    let is_selected = selected_idx == Some(idx);

    let (icon, icon_color) = match issue.severity {
        Severity::Error | Severity::Reject => (egui_phosphor::regular::X_CIRCLE, theme.error),
        Severity::Warning => (egui_phosphor::regular::WARNING, theme.warning),
        _ => (egui_phosphor::regular::INFO, theme.text_muted),
    };

    let variable_name = issue.variable.as_deref().unwrap_or("Unknown");
    let count_text = issue
        .count
        .map(|c| format!("{} invalid", c))
        .unwrap_or_default();

    let frame = egui::Frame::new()
        .fill(if is_selected {
            theme.bg_hover
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
                ui.label(RichText::new(icon).color(icon_color));
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(variable_name).strong().color(if is_selected {
                            theme.accent
                        } else {
                            theme.text_primary
                        }));
                        if !issue.code.is_empty() {
                            ui.label(RichText::new(&issue.code).small().color(theme.text_muted));
                        }
                    });
                    if !count_text.is_empty() {
                        ui.label(RichText::new(&count_text).small().color(theme.text_muted));
                    }
                });
            });
        })
        .response;

    response.interact(egui::Sense::click()).clicked()
}

/// Show details for a selected validation issue
fn show_issue_detail(ui: &mut Ui, issue: Option<&ValidationIssue>, theme: &ThemeColors) {
    let Some(issue) = issue else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Select an issue to view details").color(theme.text_muted));
        });
        return;
    };

    let variable_name = issue.variable.as_deref().unwrap_or("Unknown");

    // Header with variable name
    ui.label(
        RichText::new(format!(
            "{} {} — {}",
            match issue.severity {
                Severity::Error | Severity::Reject => egui_phosphor::regular::X_CIRCLE,
                Severity::Warning => egui_phosphor::regular::WARNING,
                _ => egui_phosphor::regular::INFO,
            },
            variable_name,
            match issue.severity {
                Severity::Error => "Error",
                Severity::Reject => "Reject",
                Severity::Warning => "Warning",
                _ => "Info",
            }
        ))
        .strong()
        .size(16.0),
    );

    ui.add_space(spacing::MD);

    // Codelist info
    if !issue.code.is_empty() {
        egui::Frame::new()
            .fill(theme.bg_secondary)
            .inner_margin(spacing::SM as f32)
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Codelist").color(theme.text_muted));
                    ui.label(RichText::new(&issue.code).strong());
                });

                if let Some(ct_source) = &issue.ct_source {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Source").color(theme.text_muted));
                        ui.label(ct_source);
                    });
                }

                // Show extensibility info based on severity
                ui.add_space(spacing::XS);
                let extensible_text = match issue.severity {
                    Severity::Error | Severity::Reject => {
                        "This codelist is non-extensible. Invalid values will block XPT export."
                    }
                    Severity::Warning => {
                        "This codelist is extensible. Values are flagged but allowed."
                    }
                    _ => "Informational notice.",
                };
                ui.label(
                    RichText::new(extensible_text)
                        .small()
                        .color(theme.text_muted),
                );
            });
    }

    ui.add_space(spacing::MD);

    // Message
    ui.label(&issue.message);

    ui.add_space(spacing::MD);

    // Invalid values found
    if let Some(observed) = &issue.observed_values {
        if !observed.is_empty() {
            ui.label(
                RichText::new(format!(
                    "{} Invalid Values Found",
                    egui_phosphor::regular::LIST_DASHES
                ))
                .strong(),
            );
            ui.add_space(spacing::XS);

            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    for value in observed {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("•").color(theme.error));
                            ui.label(
                                RichText::new(format!("\"{}\"", value))
                                    .monospace()
                                    .color(theme.text_primary),
                            );
                        });
                    }
                });
        }
    }

    ui.add_space(spacing::MD);

    // Allowed values
    if let Some(allowed) = &issue.allowed_values {
        if !allowed.is_empty() {
            ui.label(
                RichText::new(format!(
                    "{} Allowed Values (from CT)",
                    egui_phosphor::regular::LIST_CHECKS
                ))
                .strong(),
            );
            ui.add_space(spacing::XS);

            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    // Show up to 10 values
                    let display_values: Vec<_> = allowed.iter().take(10).collect();
                    let remaining = allowed.len().saturating_sub(10);

                    for value in &display_values {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("•").color(theme.success));
                            ui.label(RichText::new(*value).monospace().color(theme.text_primary));
                        });
                    }

                    if remaining > 0 {
                        ui.label(
                            RichText::new(format!("... and {} more", remaining))
                                .small()
                                .color(theme.text_muted),
                        );
                    }
                });
        }
    } else if let Some(ct_examples) = &issue.ct_examples {
        // Show examples for large codelists
        if !ct_examples.is_empty() {
            let total = issue.allowed_count.unwrap_or(0);
            ui.label(
                RichText::new(format!(
                    "{} Allowed Values ({} total)",
                    egui_phosphor::regular::LIST_CHECKS,
                    total
                ))
                .strong(),
            );
            ui.add_space(spacing::XS);

            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    for value in ct_examples {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("•").color(theme.success));
                            ui.label(RichText::new(value).monospace().color(theme.text_primary));
                        });
                    }
                    ui.label(
                        RichText::new("(showing examples)")
                            .small()
                            .color(theme.text_muted),
                    );
                });
        }
    }
}
