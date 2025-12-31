//! Validation tab
//!
//! Displays CT validation issues derived from current mapping state.
//! Read-only display - issues are shown for user awareness, not resolution.

use crate::state::{AppState, DomainStatus};
use crate::theme::{ThemeColors, colors, spacing};
use egui::{RichText, Ui};
use sdtm_standards::load_default_ct_registry;
use sdtm_transform::build_preview_dataframe;
use sdtm_validate::{CheckType, P21Category, Severity, ValidationIssue, validate_domain};
use std::collections::BTreeMap;

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

        // Clone what we need for validation
        Some((
            ms.domain().clone(),
            ms.study_id().to_string(),
            domain.source_data.clone(),
            accepted_mappings,
        ))
    };

    let Some((sdtm_domain, study_id, source_df, accepted_mappings)) = validation_data else {
        return;
    };

    // Load CT registry
    let ct_registry = load_default_ct_registry().ok();

    // Build preview DataFrame with mappings applied and CT normalized
    let preview_df = match build_preview_dataframe(
        &source_df,
        &accepted_mappings,
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
    let report = validate_domain(&sdtm_domain, &preview_df, ct_registry.as_ref());

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
    let p21_rule_id = &issue.code; // P21 rule ID (e.g., CT2001, SD0002)
    let check_label = issue
        .check_type
        .map(|ct| ct.label())
        .unwrap_or("Validation");
    let count_text = issue
        .count
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
fn show_issue_detail(ui: &mut Ui, issue: Option<&ValidationIssue>, theme: &ThemeColors) {
    let Some(issue) = issue else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Select an issue to view details").color(theme.text_muted));
        });
        return;
    };

    let variable_name = issue.variable.as_deref().unwrap_or("Unknown");
    let check_label = issue
        .check_type
        .map(|ct| ct.label())
        .unwrap_or("Validation");
    let p21_rule_id = &issue.code;
    let p21_category = issue.p21_category();

    // Header with variable name and severity
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

    ui.add_space(spacing::XS);

    // P21 Rule ID and Category
    ui.horizontal(|ui| {
        // P21 Rule ID badge
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

        // P21 Category badge
        if let Some(category) = p21_category {
            let category_label = match category {
                P21Category::Terminology => "Terminology",
                P21Category::Presence => "Presence",
                P21Category::Format => "Format",
                P21Category::Consistency => "Consistency",
                P21Category::Limit => "Limit",
                P21Category::Metadata => "Metadata",
                P21Category::CrossReference => "Cross-reference",
                P21Category::Structure => "Structure",
            };
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(egui::Margin::symmetric(6, 2))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(category_label)
                            .small()
                            .color(theme.text_muted),
                    );
                });
        }
    });

    ui.add_space(spacing::XS);

    // Check type label
    ui.label(
        RichText::new(format!("{} {}", egui_phosphor::regular::TAG, check_label))
            .small()
            .color(theme.text_muted),
    );

    ui.add_space(spacing::MD);

    // Show context based on check type
    show_check_type_context(ui, issue, theme);

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

/// Show context-specific information based on check type
fn show_check_type_context(ui: &mut Ui, issue: &ValidationIssue, theme: &ThemeColors) {
    let Some(check_type) = issue.check_type else {
        // Fallback: show codelist info if available
        show_codelist_context(ui, issue, theme);
        return;
    };

    match check_type {
        CheckType::ControlledTerminology => {
            show_codelist_context(ui, issue, theme);
        }
        CheckType::RequiredVariableMissing => {
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(egui_phosphor::regular::INFO).color(theme.accent));
                        ui.label("Required variables (Req) must be present in the dataset.");
                    });
                    ui.add_space(spacing::XS);
                    ui.label(
                        RichText::new(
                            "SDTMIG 4.1: Required variables are essential for submission.",
                        )
                        .small()
                        .color(theme.text_muted),
                    );
                });
        }
        CheckType::RequiredVariableEmpty => {
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(egui_phosphor::regular::INFO).color(theme.accent));
                        ui.label("Required variables must have values for all records.");
                    });
                    ui.add_space(spacing::XS);
                    ui.label(
                        RichText::new(
                            "SDTMIG 4.1: Null values are not permitted for Req variables.",
                        )
                        .small()
                        .color(theme.text_muted),
                    );
                });
        }
        CheckType::ExpectedVariableMissing => {
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(egui_phosphor::regular::INFO).color(theme.warning));
                        ui.label("Expected variables should be included when applicable.");
                    });
                    ui.add_space(spacing::XS);
                    ui.label(
                        RichText::new(
                            "SDTMIG 4.1: Expected variables are included when data is collected.",
                        )
                        .small()
                        .color(theme.text_muted),
                    );
                });
        }
        CheckType::DataTypeMismatch => {
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(egui_phosphor::regular::INFO).color(theme.error));
                        ui.label("Numeric (Num) variables must contain valid numeric data.");
                    });
                    ui.add_space(spacing::XS);
                    ui.label(
                        RichText::new("SDTMIG 2.4: Values must match the specified data type.")
                            .small()
                            .color(theme.text_muted),
                    );
                });
        }
        CheckType::InvalidDateFormat => {
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(egui_phosphor::regular::CALENDAR).color(theme.error),
                        );
                        ui.label("Date/time values must use ISO 8601 format.");
                    });
                    ui.add_space(spacing::XS);
                    ui.label(
                        RichText::new("SDTMIG Ch.7: Use YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS format.")
                            .small()
                            .color(theme.text_muted),
                    );
                });
        }
        CheckType::DuplicateSequence => {
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(egui_phosphor::regular::HASH).color(theme.error));
                        ui.label("Sequence numbers must be unique per subject.");
                    });
                    ui.add_space(spacing::XS);
                    ui.label(
                        RichText::new(
                            "SDTMIG 4.1.5: --SEQ uniquely identifies records within USUBJID.",
                        )
                        .small()
                        .color(theme.text_muted),
                    );
                });
        }
        CheckType::TextLengthExceeded => {
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(egui_phosphor::regular::TEXT_AA).color(theme.warning),
                        );
                        ui.label("Character value exceeds the defined maximum length.");
                    });
                    ui.add_space(spacing::XS);
                    ui.label(
                        RichText::new("SDTMIG 2.4: Values may be truncated in XPT output.")
                            .small()
                            .color(theme.text_muted),
                    );
                });
        }
        CheckType::IdentifierNull => {
            egui::Frame::new()
                .fill(theme.bg_secondary)
                .inner_margin(spacing::SM as f32)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(egui_phosphor::regular::IDENTIFICATION_CARD)
                                .color(theme.error),
                        );
                        ui.label("Identifier variables must not contain null values.");
                    });
                    ui.add_space(spacing::XS);
                    ui.label(
                        RichText::new(
                            "SDTMIG 4.1.2: Identifiers uniquely identify subject observations.",
                        )
                        .small()
                        .color(theme.text_muted),
                    );
                });
        }
        _ => {
            // Fallback for any new check types
            show_codelist_context(ui, issue, theme);
        }
    }
}

/// Show codelist-specific context for CT validation
fn show_codelist_context(ui: &mut Ui, issue: &ValidationIssue, theme: &ThemeColors) {
    if issue.ct_source.is_none() {
        return;
    }

    egui::Frame::new()
        .fill(theme.bg_secondary)
        .inner_margin(spacing::SM as f32)
        .corner_radius(4.0)
        .show(ui, |ui| {
            // P21 Rule explanation
            let p21_explanation = match issue.code.as_str() {
                "CT2001" => "Non-extensible codelist violation",
                "CT2002" => "Extensible codelist violation",
                _ => "Controlled terminology check",
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new("P21 Rule").color(theme.text_muted));
                ui.label(RichText::new(p21_explanation).strong());
            });

            if let Some(ct_source) = &issue.ct_source {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("CT Source").color(theme.text_muted));
                    ui.label(ct_source);
                });
            }

            // Show extensibility info based on severity
            ui.add_space(spacing::XS);
            let extensible_text = match issue.severity {
                Severity::Error | Severity::Reject => {
                    "This codelist is non-extensible. Invalid values will block XPT export."
                }
                Severity::Warning => "This codelist is extensible. Values are flagged but allowed.",
                _ => "Informational notice.",
            };
            ui.label(
                RichText::new(extensible_text)
                    .small()
                    .color(theme.text_muted),
            );
        });
}
