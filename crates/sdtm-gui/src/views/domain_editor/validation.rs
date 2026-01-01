//! Validation tab
//!
//! Displays validation issues derived from current mapping state.
//! Read-only display - issues are shown for user awareness, not resolution.

use crate::state::{AppState, DomainStatus};
use crate::theme::{ThemeColors, colors, spacing};
use egui::{RichText, Ui};
use sdtm_standards::load_default_ct_registry;
use sdtm_transform::build_preview_dataframe_with_omitted;
use sdtm_validate::{Issue, Severity, validate_domain_with_not_collected};
use std::collections::{BTreeMap, BTreeSet};

use super::mapping::{initialize_mapping, show_loading_indicator};

/// Render the validation tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    let theme = colors(state.settings.general.dark_mode);

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
                    report.error_count(None),
                    report.warning_count(None),
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
///
/// This validates the **transformed** data (with mappings applied and CT normalized),
/// not the raw source data. This ensures validation reflects what the final output will be.
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

    // Get the data we need for building the preview and validation
    let validation_data = {
        let Some(study) = &state.study else { return };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };
        let Some(ms) = &domain.mapping_state else {
            return;
        };

        // Build accepted mappings map: SDTM variable -> source column
        let accepted_mappings: BTreeMap<String, String> = ms
            .all_accepted()
            .iter()
            .map(|(var, (col, _))| (var.clone(), col.clone()))
            .collect();

        // Get omitted variables (to exclude from output)
        let omitted_vars: BTreeSet<String> = ms.all_omitted().clone();

        // Get not_collected variables (to suppress ExpectedMissing warnings)
        let not_collected_vars: BTreeSet<String> = ms.all_not_collected().keys().cloned().collect();

        // Clone what we need for validation
        Some((
            ms.domain().clone(),
            ms.study_id().to_string(),
            domain.source_data.clone(),
            accepted_mappings,
            omitted_vars,
            not_collected_vars,
        ))
    };

    let Some((
        sdtm_domain,
        study_id,
        source_df,
        accepted_mappings,
        omitted_vars,
        not_collected_vars,
    )) = validation_data
    else {
        return;
    };

    // Load CT registry
    let ct_registry = load_default_ct_registry().ok();

    // Build preview DataFrame with mappings applied, CT normalized, and omitted vars excluded
    let preview_df = match build_preview_dataframe_with_omitted(
        &source_df,
        &accepted_mappings,
        &omitted_vars,
        &sdtm_domain,
        &study_id,
        ct_registry.as_ref(),
    ) {
        Ok(df) => df,
        Err(e) => {
            tracing::warn!("Failed to build preview DataFrame: {}", e);
            // Fall back to source data if preview fails
            source_df
        }
    };

    // Run validation on the transformed preview data
    // Pass not_collected vars to suppress ExpectedMissing warnings for acknowledged missing data
    let report = validate_domain_with_not_collected(
        &sdtm_domain,
        &preview_df,
        ct_registry.as_ref(),
        &not_collected_vars,
    );

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

/// Get severity for an issue
fn get_severity(issue: &Issue) -> Severity {
    issue.default_severity()
}

/// Get check type label for an issue
fn get_check_label(issue: &Issue) -> &'static str {
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

/// Show list of validation issues grouped by severity
fn show_issue_list(
    ui: &mut Ui,
    issues: &[Issue],
    selected_idx: Option<usize>,
    theme: &ThemeColors,
) -> Option<usize> {
    let mut new_selection = None;

    // Separate errors and warnings
    let errors: Vec<(usize, &Issue)> = issues
        .iter()
        .enumerate()
        .filter(|(_, i)| matches!(get_severity(i), Severity::Error | Severity::Reject))
        .collect();

    let warnings: Vec<(usize, &Issue)> = issues
        .iter()
        .enumerate()
        .filter(|(_, i)| matches!(get_severity(i), Severity::Warning))
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
    issue: &Issue,
    selected_idx: Option<usize>,
    theme: &ThemeColors,
) -> bool {
    let is_selected = selected_idx == Some(idx);
    let severity = get_severity(issue);

    let (icon, icon_color) = match severity {
        Severity::Error | Severity::Reject => (egui_phosphor::regular::X_CIRCLE, theme.error),
        Severity::Warning => (egui_phosphor::regular::WARNING, theme.warning),
    };

    let variable_name = issue.variable();
    let p21_rule_id = issue.rule_id();
    let check_label = get_check_label(issue);
    let count_text = issue
        .count()
        .map(|c| format!("{} issue(s)", c))
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
                        // Show P21 rule ID badge
                        ui.label(
                            RichText::new(p21_rule_id)
                                .small()
                                .monospace()
                                .color(theme.text_muted),
                        );
                    });
                    ui.label(RichText::new(check_label).small().color(theme.text_muted));
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
fn show_issue_detail(ui: &mut Ui, issue: Option<&Issue>, theme: &ThemeColors) {
    let Some(issue) = issue else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Select an issue to view details").color(theme.text_muted));
        });
        return;
    };

    let variable_name = issue.variable();
    let check_label = get_check_label(issue);
    let p21_rule_id = issue.rule_id();
    let severity = get_severity(issue);

    // Header with variable name and severity
    ui.label(
        RichText::new(format!(
            "{} {} — {}",
            match severity {
                Severity::Error | Severity::Reject => egui_phosphor::regular::X_CIRCLE,
                Severity::Warning => egui_phosphor::regular::WARNING,
            },
            variable_name,
            severity.label()
        ))
        .strong()
        .size(16.0),
    );

    ui.add_space(spacing::XS);

    // P21 Rule ID badge
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(theme.accent.gamma_multiply(0.2))
            .inner_margin(egui::Margin::symmetric(6, 2))
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(p21_rule_id)
                        .monospace()
                        .strong()
                        .color(theme.accent),
                );
            });
    });

    ui.add_space(spacing::XS);

    // Check type label
    ui.label(
        RichText::new(format!("{} {}", egui_phosphor::regular::TAG, check_label))
            .small()
            .color(theme.text_muted),
    );

    ui.add_space(spacing::MD);

    // Show context based on issue type
    show_issue_context(ui, issue, theme);

    ui.add_space(spacing::MD);

    // Message
    ui.label(issue.message(None));

    ui.add_space(spacing::MD);

    // Show issue-specific details
    show_issue_specific_details(ui, issue, theme);
}

/// Show context-specific information based on issue type
fn show_issue_context(ui: &mut Ui, issue: &Issue, theme: &ThemeColors) {
    match issue {
        Issue::CtViolation {
            codelist_name,
            extensible,
            ..
        } => {
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Codelist").color(theme.text_muted));
                        ui.label(RichText::new(codelist_name).strong());
                    });
                    ui.add_space(spacing::XS);
                    let ext_text = if *extensible {
                        "This codelist is extensible. Values are flagged but allowed."
                    } else {
                        "This codelist is non-extensible. Invalid values will block XPT export."
                    };
                    ui.label(RichText::new(ext_text).small().color(theme.text_muted));
                });
        }
        Issue::RequiredMissing { .. } => {
            show_context_frame(
                ui,
                theme,
                egui_phosphor::regular::INFO,
                theme.accent,
                "Required variables (Req) must be present in the dataset.",
                "SDTMIG 4.1: Required variables are essential for submission.",
            );
        }
        Issue::RequiredEmpty { .. } => {
            show_context_frame(
                ui,
                theme,
                egui_phosphor::regular::INFO,
                theme.accent,
                "Required variables must have values for all records.",
                "SDTMIG 4.1: Null values are not permitted for Req variables.",
            );
        }
        Issue::ExpectedMissing { .. } => {
            show_context_frame(
                ui,
                theme,
                egui_phosphor::regular::INFO,
                theme.warning,
                "Expected variables should be included when applicable.",
                "SDTMIG 4.1: Expected variables are included when data is collected.",
            );
        }
        Issue::DataTypeMismatch { .. } => {
            show_context_frame(
                ui,
                theme,
                egui_phosphor::regular::INFO,
                theme.error,
                "Numeric (Num) variables must contain valid numeric data.",
                "SDTMIG 2.4: Values must match the specified data type.",
            );
        }
        Issue::InvalidDate { .. } => {
            show_context_frame(
                ui,
                theme,
                egui_phosphor::regular::CALENDAR,
                theme.error,
                "Date/time values must use ISO 8601 format.",
                "SDTMIG Ch.7: Use YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS format.",
            );
        }
        Issue::DuplicateSequence { .. } => {
            show_context_frame(
                ui,
                theme,
                egui_phosphor::regular::HASH,
                theme.error,
                "Sequence numbers must be unique per subject.",
                "SDTMIG 4.1.5: --SEQ uniquely identifies records within USUBJID.",
            );
        }
        Issue::TextTooLong { .. } => {
            show_context_frame(
                ui,
                theme,
                egui_phosphor::regular::TEXT_AA,
                theme.warning,
                "Character value exceeds the defined maximum length.",
                "SDTMIG 2.4: Values may be truncated in XPT output.",
            );
        }
        Issue::IdentifierNull { .. } => {
            show_context_frame(
                ui,
                theme,
                egui_phosphor::regular::IDENTIFICATION_CARD,
                theme.error,
                "Identifier variables must not contain null values.",
                "SDTMIG 4.1.2: Identifiers uniquely identify subject observations.",
            );
        }
    }
}

/// Helper to show a context frame
fn show_context_frame(
    ui: &mut Ui,
    theme: &ThemeColors,
    icon: &str,
    icon_color: egui::Color32,
    main_text: &str,
    ref_text: &str,
) {
    egui::Frame::new()
        .fill(theme.bg_secondary)
        .inner_margin(spacing::SM as f32)
        .corner_radius(4.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).color(icon_color));
                ui.label(main_text);
            });
            ui.add_space(spacing::XS);
            ui.label(RichText::new(ref_text).small().color(theme.text_muted));
        });
}

/// Show issue-specific details (samples, allowed values, etc.)
fn show_issue_specific_details(ui: &mut Ui, issue: &Issue, theme: &ThemeColors) {
    match issue {
        Issue::InvalidDate { samples, .. } | Issue::DataTypeMismatch { samples, .. } => {
            if !samples.is_empty() {
                ui.label(
                    RichText::new(format!(
                        "{} Invalid Values Found",
                        egui_phosphor::regular::LIST_DASHES
                    ))
                    .strong(),
                );
                ui.add_space(spacing::XS);
                show_value_list(ui, samples, theme.error, theme);
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
                        "{} Invalid Values Found",
                        egui_phosphor::regular::LIST_DASHES
                    ))
                    .strong(),
                );
                ui.add_space(spacing::XS);
                show_value_list(ui, invalid_values, theme.error, theme);
            }

            ui.add_space(spacing::MD);
            ui.label(
                RichText::new(format!(
                    "{} Codelist has {} allowed values",
                    egui_phosphor::regular::LIST_CHECKS,
                    allowed_count
                ))
                .color(theme.text_muted),
            );
        }
        Issue::TextTooLong {
            max_found,
            max_allowed,
            ..
        } => {
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Max allowed:").color(theme.text_muted));
                        ui.label(RichText::new(format!("{} chars", max_allowed)).strong());
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Max found:").color(theme.text_muted));
                        ui.label(
                            RichText::new(format!("{} chars", max_found))
                                .strong()
                                .color(theme.error),
                        );
                    });
                });
        }
        _ => {}
    }
}

/// Show a list of values in a frame
fn show_value_list(
    ui: &mut Ui,
    values: &[String],
    bullet_color: egui::Color32,
    theme: &ThemeColors,
) {
    egui::Frame::new()
        .fill(theme.bg_secondary)
        .inner_margin(spacing::SM as f32)
        .corner_radius(4.0)
        .show(ui, |ui| {
            for value in values.iter().take(10) {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("•").color(bullet_color));
                    ui.label(
                        RichText::new(format!("\"{}\"", value))
                            .monospace()
                            .color(theme.text_primary),
                    );
                });
            }
            if values.len() > 10 {
                ui.label(
                    RichText::new(format!("... and {} more", values.len() - 10))
                        .small()
                        .color(theme.text_muted),
                );
            }
        });
}
