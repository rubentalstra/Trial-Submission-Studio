//! Validation tab view.
//!
//! The validation tab displays CDISC conformance issues found during
//! validation of the mapped and normalized data.

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length};
use iced_fonts::lucide;
use tss_submit::{Issue, Severity, ValidationReport};

use crate::message::domain_editor::{SeverityFilter as MsgSeverityFilter, ValidationMessage};
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, SeverityFilter, ValidationUiState, ViewState};
use crate::theme::{
    ERROR, GRAY_100, GRAY_200, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900,
    PRIMARY_100, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, SUCCESS, WARNING, WHITE,
    button_ghost, button_primary,
};

// =============================================================================
// MAIN VALIDATION TAB VIEW
// =============================================================================

/// Render the validation tab content.
pub fn view_validation_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let _domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return text("Domain not found").size(14).color(GRAY_500).into();
        }
    };

    // Get validation cache and UI state
    let (validation_cache, validation_ui) = match &state.view {
        ViewState::DomainEditor {
            validation_cache,
            validation_ui,
            ..
        } => (validation_cache, validation_ui),
        _ => return text("Invalid view state").into(),
    };

    // Header
    let header = view_validation_header(validation_cache.as_ref(), validation_ui);

    // Content based on state
    let content: Element<'a, Message> = if let Some(report) = validation_cache {
        if report.is_empty() {
            view_no_issues_state()
        } else {
            view_issues_list(report, validation_ui)
        }
    } else {
        view_empty_state()
    };

    column![header, Space::new().height(SPACING_MD), content,]
        .spacing(0)
        .into()
}

// =============================================================================
// HEADER
// =============================================================================

/// Validation header with stats and filter.
fn view_validation_header<'a>(
    report: Option<&ValidationReport>,
    ui: &ValidationUiState,
) -> Element<'a, Message> {
    let title = text("Validation Issues").size(18).color(GRAY_900);

    // Stats subtitle
    let subtitle_text = if let Some(r) = report {
        let errors = r.error_count();
        let warnings = r.warning_count();
        if errors == 0 && warnings == 0 {
            "No issues found".to_string()
        } else {
            format!("{} errors, {} warnings", errors, warnings)
        }
    } else {
        "Run validation to check for issues".to_string()
    };
    let subtitle = text(subtitle_text).size(13).color(GRAY_600);

    // Severity filter buttons
    let filter_buttons = row![
        filter_button("All", SeverityFilter::All, ui.severity_filter),
        filter_button("Errors", SeverityFilter::Errors, ui.severity_filter),
        filter_button("Warnings", SeverityFilter::Warnings, ui.severity_filter),
    ]
    .spacing(4.0);

    // Refresh button
    let refresh_button = button(
        row![lucide::refresh_cw().size(12), text("Re-validate").size(14),]
            .spacing(SPACING_SM)
            .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Validation(
        ValidationMessage::RefreshValidation,
    )))
    .padding([8.0, 16.0])
    .style(button_primary);

    row![
        column![title, Space::new().height(4.0), subtitle,],
        Space::new().width(Length::Fill),
        filter_buttons,
        Space::new().width(SPACING_MD),
        refresh_button,
    ]
    .align_y(Alignment::Start)
    .into()
}

/// Filter button for severity.
fn filter_button<'a>(
    label: &'static str,
    filter: SeverityFilter,
    current: SeverityFilter,
) -> Element<'a, Message> {
    let is_selected = filter == current;
    let msg_filter = match filter {
        SeverityFilter::All => MsgSeverityFilter::All,
        SeverityFilter::Errors => MsgSeverityFilter::Errors,
        SeverityFilter::Warnings => MsgSeverityFilter::Warnings,
        SeverityFilter::Info => MsgSeverityFilter::Info,
    };

    button(text(label).size(12))
        .on_press(Message::DomainEditor(DomainEditorMessage::Validation(
            ValidationMessage::SeverityFilterChanged(msg_filter),
        )))
        .padding([6.0, 12.0])
        .style(move |_theme, _status| {
            if is_selected {
                iced::widget::button::Style {
                    background: Some(PRIMARY_500.into()),
                    text_color: iced::Color::WHITE,
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            } else {
                button_ghost(_theme, _status)
            }
        })
        .into()
}

// =============================================================================
// ISSUES LIST
// =============================================================================

/// Display list of validation issues.
fn view_issues_list<'a>(
    report: &'a ValidationReport,
    ui: &ValidationUiState,
) -> Element<'a, Message> {
    // Filter issues by severity
    let filtered: Vec<&Issue> = report
        .issues
        .iter()
        .filter(|issue| match ui.severity_filter {
            SeverityFilter::All => true,
            SeverityFilter::Errors => {
                matches!(issue.severity(), Severity::Error | Severity::Reject)
            }
            SeverityFilter::Warnings => matches!(issue.severity(), Severity::Warning),
            SeverityFilter::Info => false, // No info level in current model
        })
        .collect();

    if filtered.is_empty() {
        return view_no_filtered_issues(ui.severity_filter);
    }

    // Build issue rows
    let mut issues_col = column![].spacing(SPACING_SM);

    for (idx, issue) in filtered.iter().enumerate() {
        let is_selected = ui.selected_issue == Some(idx);
        issues_col = issues_col.push(view_issue_row(issue, idx, is_selected));
    }

    // Wrap in scrollable container
    container(scrollable(issues_col.padding(SPACING_SM)))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            border: Border {
                width: 1.0,
                radius: 4.0.into(),
                color: GRAY_200,
            },
            ..Default::default()
        })
        .into()
}

/// Single issue row.
fn view_issue_row<'a>(issue: &Issue, idx: usize, is_selected: bool) -> Element<'a, Message> {
    let severity = issue.severity();
    let severity_color = match severity {
        Severity::Reject | Severity::Error => ERROR,
        Severity::Warning => WARNING,
    };

    // Clone values we need to own
    let rule_id = issue.rule_id().to_string();
    let variable_name = issue.variable().to_string();
    let message_text = issue.message();

    // Severity icon
    let severity_icon: Element<'a, Message> = match severity {
        Severity::Reject | Severity::Error => {
            lucide::circle_x().size(16).color(severity_color).into()
        }
        Severity::Warning => lucide::circle_alert().size(16).color(severity_color).into(),
    };

    // Rule ID badge
    let rule_badge = container(text(rule_id).size(10).color(GRAY_600))
        .padding([2.0, 6.0])
        .style(|_theme| container::Style {
            background: Some(GRAY_100.into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    // Variable name
    let variable = text(variable_name.clone()).size(13).color(GRAY_800);

    // Issue message
    let message = text(message_text).size(12).color(GRAY_600);

    // Go to source button
    let go_to_button = button(
        row![lucide::arrow_right().size(10), text("Go to").size(11),]
            .spacing(4.0)
            .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Validation(
        ValidationMessage::GoToIssueSource {
            variable: variable_name,
        },
    )))
    .padding([4.0, 8.0])
    .style(button_ghost);

    // Row content
    let row_content = row![
        severity_icon,
        Space::new().width(SPACING_SM),
        rule_badge,
        Space::new().width(SPACING_SM),
        variable,
        Space::new().width(SPACING_SM),
        text("•").size(12).color(GRAY_400),
        Space::new().width(SPACING_SM),
        message,
        Space::new().width(Length::Fill),
        go_to_button,
    ]
    .align_y(Alignment::Center);

    // Clickable row
    let bg_color = if is_selected { PRIMARY_100 } else { WHITE };

    button(row_content)
        .on_press(Message::DomainEditor(DomainEditorMessage::Validation(
            ValidationMessage::IssueSelected(idx),
        )))
        .width(Length::Fill)
        .padding([SPACING_SM, SPACING_MD])
        .style(move |_theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered if !is_selected => Some(GRAY_100.into()),
                _ => Some(bg_color.into()),
            };
            iced::widget::button::Style {
                background: bg,
                border: Border {
                    width: 1.0,
                    radius: 6.0.into(),
                    color: if is_selected { PRIMARY_500 } else { GRAY_200 },
                },
                ..Default::default()
            }
        })
        .into()
}

// =============================================================================
// STATES
// =============================================================================

/// State when there are no filtered issues.
fn view_no_filtered_issues<'a>(filter: SeverityFilter) -> Element<'a, Message> {
    let filter_name = match filter {
        SeverityFilter::All => "issues",
        SeverityFilter::Errors => "errors",
        SeverityFilter::Warnings => "warnings",
        SeverityFilter::Info => "info messages",
    };

    container(
        column![
            lucide::search().size(32).color(GRAY_400),
            Space::new().height(SPACING_MD),
            text(format!("No {} found", filter_name))
                .size(14)
                .color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Try changing the severity filter")
                .size(12)
                .color(GRAY_500),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(200.0))
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}

/// State when validation has no issues.
fn view_no_issues_state<'a>() -> Element<'a, Message> {
    container(
        column![
            lucide::circle_check().size(48).color(SUCCESS),
            Space::new().height(SPACING_MD),
            text("No Issues Found").size(16).color(GRAY_700),
            Space::new().height(SPACING_SM),
            text("All validation checks passed successfully")
                .size(13)
                .color(GRAY_500),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(300.0))
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}

/// Empty state when no validation has been run.
fn view_empty_state<'a>() -> Element<'a, Message> {
    container(
        column![
            lucide::shield_check().size(48).color(GRAY_400),
            Space::new().height(SPACING_MD),
            text("No Validation Results").size(16).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Click 'Re-validate' to check for CDISC conformance issues")
                .size(13)
                .color(GRAY_500),
            Space::new().height(SPACING_LG),
            button(
                row![
                    lucide::refresh_cw().size(12),
                    text("Run Validation").size(14),
                ]
                .spacing(SPACING_SM)
                .align_y(Alignment::Center),
            )
            .on_press(Message::DomainEditor(DomainEditorMessage::Validation(
                ValidationMessage::RefreshValidation,
            )))
            .padding([10.0, 20.0])
            .style(button_primary),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(300.0))
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}
