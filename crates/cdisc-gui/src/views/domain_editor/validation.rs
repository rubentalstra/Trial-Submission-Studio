//! Validation tab
//!
//! Shows validation results for the mapped data.
//! Uses a 2-column layout: issues list on left, details on right.
//! Validation is run automatically when mappings change.

use crate::state::{AppState, Versioned};
use crate::theme::spacing;
use egui::{Color32, RichText, Ui};
use cdisc_standards::{load_ct, CtVersion};
use cdisc_validate::{Issue, Severity, ValidationReport, validate_domain_with_not_collected};
use std::collections::BTreeSet;

/// Render the validation tab
pub fn show(ui: &mut Ui, state: &mut AppState, domain_code: &str) {
    // Check if domain is accessible (DM check)
    let Some(domain) = state.domain(domain_code) else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Domain not accessible").color(ui.visuals().error_fg_color));
        });
        return;
    };

    let domain_version = domain.version;

    // Check if validation report needs rebuilding
    let needs_rebuild = domain
        .derived
        .validation
        .as_ref()
        .map(|v| v.is_stale(domain_version))
        .unwrap_or(true);

    if needs_rebuild {
        // Show loading and trigger rebuild
        ui.centered_and_justified(|ui| {
            ui.spinner();
            ui.label("Running validation checks...");
        });
        ui.ctx().request_repaint();
        rebuild_validation_report(state, domain_code);
        return;
    }

    // Get validation report (clone to avoid borrow issues)
    let report = state
        .domain(domain_code)
        .and_then(|d| d.derived.validation.as_ref())
        .map(|v| v.data.clone());

    let Some(report) = report else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("No validation report available").weak());
        });
        return;
    };

    // If no issues, show full-width success message
    if report.issues.is_empty() {
        show_no_issues(ui);
        return;
    }

    // 2-column layout using StripBuilder
    let available_height = ui.available_height();

    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(320.0)) // Left panel fixed width
        .size(egui_extras::Size::exact(1.0))   // Separator
        .size(egui_extras::Size::remainder())  // Right panel takes rest
        .horizontal(|mut strip| {
            // Left: Issues list
            strip.cell(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        show_issues_list(ui, state, domain_code, &report);
                    });
            });

            // Separator
            strip.cell(|ui| {
                ui.separator();
            });

            // Right: Detail panel
            strip.cell(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        show_issue_detail(ui, state, domain_code, &report);
                    });
            });
        });
}

/// Show "no issues" success state
fn show_no_issues(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(spacing::XL);
        ui.label(
            RichText::new(egui_phosphor::regular::CHECK_CIRCLE)
                .size(48.0)
                .color(Color32::from_rgb(100, 180, 100)),
        );
        ui.add_space(spacing::MD);
        ui.label(RichText::new("All Checks Passed").size(18.0).strong());
        ui.add_space(spacing::SM);
        ui.label(RichText::new("No validation issues found in the current mapping.").weak());
    });
}

/// Show the issues list (left panel)
fn show_issues_list(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    report: &ValidationReport,
) {
    let selected_idx = state
        .ui
        .get_domain_editor(domain_code)
        .and_then(|ui| ui.validation.selected_idx);

    // Count by severity
    let issues = &report.issues;
    let error_count = issues
        .iter()
        .filter(|i| matches!(i.default_severity(), Severity::Error | Severity::Reject))
        .count();
    let warning_count = issues
        .iter()
        .filter(|i| matches!(i.default_severity(), Severity::Warning))
        .count();

    // Header
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{}", issues.len())).strong());
        ui.label(RichText::new("issues").weak().small());
        ui.separator();
        if error_count > 0 {
            ui.label(
                RichText::new(format!("{} errors", error_count))
                    .small()
                    .color(ui.visuals().error_fg_color),
            );
        }
        if warning_count > 0 {
            ui.label(
                RichText::new(format!("{} warnings", warning_count))
                    .small()
                    .color(ui.visuals().warn_fg_color),
            );
        }
    });

    ui.add_space(spacing::SM);
    ui.separator();

    // Sort issues by severity
    let sorted_issues = report.sorted_by_severity(None);

    // Build table
    let mut new_selection: Option<usize> = None;
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(egui_extras::Column::exact(24.0))                // Severity icon
        .column(egui_extras::Column::exact(70.0))                // Rule ID
        .column(egui_extras::Column::remainder().at_least(80.0)) // Variable
        .header(text_height + 4.0, |mut header| {
            header.col(|_ui| {});
            header.col(|ui| {
                ui.label(RichText::new("Rule").small().strong());
            });
            header.col(|ui| {
                ui.label(RichText::new("Variable").small().strong());
            });
        })
        .body(|body| {
            body.rows(text_height + 8.0, sorted_issues.len(), |mut row| {
                let row_idx = row.index();
                let issue = &sorted_issues[row_idx];
                let is_selected = selected_idx == Some(row_idx);
                let severity = issue.default_severity();

                // Severity icon column
                row.col(|ui| {
                    let (icon, color) = severity_icon_color(severity, ui);
                    ui.label(RichText::new(icon).color(color));
                });

                // Rule ID column
                row.col(|ui| {
                    ui.label(
                        RichText::new(issue.rule_id())
                            .monospace()
                            .small(),
                    );
                });

                // Variable column (clickable)
                row.col(|ui| {
                    let mut label_text = RichText::new(issue.variable()).monospace();
                    if is_selected {
                        label_text = label_text.strong();
                    }

                    let response = ui.selectable_label(is_selected, label_text);
                    if response.clicked() {
                        new_selection = Some(row_idx);
                    }

                    // Show count on hover
                    if let Some(count) = issue.count() {
                        response.on_hover_text(format!("{} occurrences", count));
                    }
                });
            });
        });

    // Apply selection change
    if let Some(idx) = new_selection {
        let selection = if selected_idx == Some(idx) {
            None // Toggle off
        } else {
            Some(idx)
        };
        state
            .ui
            .domain_editor(domain_code)
            .validation
            .select(selection);
    }
}

/// Show the issue detail (right panel)
fn show_issue_detail(
    ui: &mut Ui,
    state: &mut AppState,
    domain_code: &str,
    report: &ValidationReport,
) {
    let selected_idx = state
        .ui
        .get_domain_editor(domain_code)
        .and_then(|ui| ui.validation.selected_idx);

    let Some(idx) = selected_idx else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Select an issue from the list").weak());
        });
        return;
    };

    // Get sorted issues (same order as list)
    let sorted_issues = report.sorted_by_severity(None);
    let Some(issue) = sorted_issues.get(idx) else {
        ui.label(RichText::new("Issue not found").weak());
        return;
    };

    let severity = issue.default_severity();
    let (icon, color) = severity_icon_color(severity, ui);

    // Header with severity and variable
    ui.horizontal(|ui| {
        ui.label(RichText::new(icon).color(color).size(20.0));
        ui.heading(issue.variable());
    });

    ui.add_space(spacing::SM);

    // Severity badge
    let severity_label = match severity {
        Severity::Reject => "REJECT",
        Severity::Error => "ERROR",
        Severity::Warning => "WARNING",
    };
    ui.label(RichText::new(severity_label).small().color(color).strong());

    ui.add_space(spacing::MD);

    // Metadata grid
    egui::Grid::new("issue_metadata")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Rule ID").weak());
            ui.label(RichText::new(issue.rule_id()).monospace());
            ui.end_row();

            if let Some(count) = issue.count() {
                ui.label(RichText::new("Occurrences").weak());
                ui.label(RichText::new(format!("{}", count)).strong());
                ui.end_row();
            }
        });

    ui.add_space(spacing::LG);

    // Message section
    ui.label(
        RichText::new(format!("{} Message", egui_phosphor::regular::CHAT_TEXT))
            .strong()
            .weak(),
    );
    ui.separator();
    ui.add_space(spacing::SM);

    ui.label(issue.message(None));

    // Issue-specific details
    match issue {
        Issue::CtViolation {
            invalid_values,
            codelist_name,
            codelist_code,
            extensible,
            ..
        } => {
            ui.add_space(spacing::LG);

            ui.label(
                RichText::new(format!(
                    "{} Controlled Terminology",
                    egui_phosphor::regular::LIST_CHECKS
                ))
                .strong()
                .weak(),
            );
            ui.separator();
            ui.add_space(spacing::SM);

            egui::Grid::new("ct_details")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Codelist").weak());
                    ui.label(RichText::new(codelist_name).monospace());
                    ui.end_row();

                    ui.label(RichText::new("Code").weak());
                    ui.label(RichText::new(codelist_code).monospace().small());
                    ui.end_row();

                    ui.label(RichText::new("Extensible").weak());
                    ui.label(if *extensible { "Yes" } else { "No" });
                    ui.end_row();
                });

            if !invalid_values.is_empty() {
                ui.add_space(spacing::MD);
                ui.label(RichText::new("Invalid Values:").weak());
                ui.add_space(spacing::XS);

                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(egui::Margin::same(8))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        for (i, value) in invalid_values.iter().take(10).enumerate() {
                            ui.label(
                                RichText::new(format!("{}. {}", i + 1, value))
                                    .monospace()
                                    .color(ui.visuals().error_fg_color),
                            );
                        }
                        if invalid_values.len() > 10 {
                            ui.label(
                                RichText::new(format!(
                                    "... and {} more",
                                    invalid_values.len() - 10
                                ))
                                .weak()
                                .small()
                                .italics(),
                            );
                        }
                    });
            }
        }

        Issue::RequiredMissing { .. } => {
            ui.add_space(spacing::LG);

            ui.label(
                RichText::new(format!("{} How to Fix", egui_phosphor::regular::LIGHTBULB))
                    .strong()
                    .weak(),
            );
            ui.separator();
            ui.add_space(spacing::SM);

            ui.label("This variable is Required per SDTMIG and must be populated.");
            ui.add_space(spacing::XS);
            ui.label(RichText::new("Options:").weak());
            ui.label("• Map a source column to this variable in the Mapping tab");
            ui.label("• If the value can be derived, it may be auto-generated");
        }

        Issue::RequiredEmpty { null_count, .. } => {
            ui.add_space(spacing::LG);

            ui.label(
                RichText::new(format!("{} Details", egui_phosphor::regular::INFO))
                    .strong()
                    .weak(),
            );
            ui.separator();
            ui.add_space(spacing::SM);

            ui.label(format!(
                "Found {} null/empty values in a Required variable.",
                null_count
            ));
            ui.add_space(spacing::XS);
            ui.label(
                RichText::new("Required variables must be populated for every record.").weak(),
            );
        }

        Issue::ExpectedMissing { .. } => {
            ui.add_space(spacing::LG);

            ui.label(
                RichText::new(format!("{} Details", egui_phosphor::regular::INFO))
                    .strong()
                    .weak(),
            );
            ui.separator();
            ui.add_space(spacing::SM);

            ui.label("This variable is Expected per SDTMIG.");
            ui.add_space(spacing::XS);
            ui.label(
                RichText::new("Expected variables should be included when data is available.")
                    .weak(),
            );
        }

        Issue::IdentifierNull { null_count, .. } => {
            ui.add_space(spacing::LG);

            ui.label(
                RichText::new(format!("{} Details", egui_phosphor::regular::INFO))
                    .strong()
                    .weak(),
            );
            ui.separator();
            ui.add_space(spacing::SM);

            ui.label(format!(
                "Found {} null/empty values in an identifier variable.",
                null_count
            ));
            ui.add_space(spacing::XS);
            ui.label(
                RichText::new("Identifier variables must be populated for every record.").weak(),
            );
        }

        _ => {
            // Generic issue - no additional details needed
        }
    }

    ui.add_space(spacing::LG);

    // Recommendation section
    ui.label(
        RichText::new(format!(
            "{} Recommendation",
            egui_phosphor::regular::ARROW_RIGHT
        ))
        .strong()
        .weak(),
    );
    ui.separator();
    ui.add_space(spacing::SM);

    let recommendation = match severity {
        Severity::Reject => "This issue will cause submission rejection. Must be resolved.",
        Severity::Error => "This is a significant issue that should be fixed before submission.",
        Severity::Warning => "Review this issue and fix if applicable to your study.",
    };
    ui.label(RichText::new(recommendation).italics());
}

/// Get icon and color for severity
fn severity_icon_color(severity: Severity, ui: &Ui) -> (&'static str, Color32) {
    match severity {
        Severity::Reject => (
            egui_phosphor::regular::PROHIBIT,
            ui.visuals().error_fg_color,
        ),
        Severity::Error => (
            egui_phosphor::regular::X_CIRCLE,
            ui.visuals().error_fg_color,
        ),
        Severity::Warning => (egui_phosphor::regular::WARNING, ui.visuals().warn_fg_color),
    }
}

/// Rebuild the validation report
fn rebuild_validation_report(state: &mut AppState, domain_code: &str) {
    let report_result = {
        let Some(study) = state.study() else {
            return;
        };
        let Some(domain) = study.get_domain(domain_code) else {
            return;
        };

        // Get the SDTM domain definition
        let sdtm_domain = domain.mapping.domain();

        // Get preview DataFrame if available
        let preview_df = domain.derived.preview.as_ref().map(|v| &v.data);

        // Get "not collected" variables (omitted but intentionally so)
        let not_collected: BTreeSet<String> = domain.mapping.all_omitted().clone();

        // Run validation if we have preview data
        if let Some(df) = preview_df {
            // Load CT registry for terminology validation
            let ct_registry = load_ct(CtVersion::default()).ok();
            validate_domain_with_not_collected(
                sdtm_domain,
                df,
                ct_registry.as_ref(),
                &not_collected,
            )
        } else {
            // No preview data, return empty report
            ValidationReport::new(domain_code)
        }
    };

    // Store the result in derived state
    if let Some(domain) = state
        .study_mut()
        .and_then(|s| s.get_domain_mut(domain_code))
    {
        let version = domain.version;
        domain.derived.validation = Some(Versioned {
            data: report_result,
            source_version: version,
        });
    }
}
