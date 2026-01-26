//! Master panel components for the Validation tab.
//!
//! Contains the left-side issue list with header and filters.

use iced::widget::{Space, button, column, container, row, rule, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;
use tss_submit::{Issue, Severity, ValidationReport};

use crate::component::display::{NoFilteredResults, SelectableRow};
use crate::message::domain_editor::{SeverityFilter as MsgSeverityFilter, ValidationMessage};
use crate::message::{DomainEditorMessage, Message};
use crate::state::{SeverityFilter, ValidationUiState};
use crate::theme::{ClinicalColors, SPACING_SM, SPACING_XS, button_primary, button_secondary};

use super::helpers::{get_severity_color, truncate_message};

// =============================================================================
// MASTER PANEL HEADER
// =============================================================================

/// Master panel header with title, filters, and stats.
pub(super) fn view_master_header<'a>(
    report: &ValidationReport,
    ui: &ValidationUiState,
    filtered_count: usize,
) -> Element<'a, Message> {
    // Title
    let title = text("Issues")
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        })
        .font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..Default::default()
        });

    // Re-validate button
    let refresh_button = button(
        row![
            container(lucide::refresh_cw().size(12)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            }),
            text("Re-validate").size(12),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Validation(
        ValidationMessage::RefreshValidation,
    )))
    .padding([6.0, 12.0])
    .style(button_primary);

    let title_row =
        row![title, Space::new().width(Length::Fill), refresh_button,].align_y(Alignment::Center);

    // Severity filter buttons
    let filter_buttons = row![
        severity_filter_button("All", SeverityFilter::All, ui.severity_filter),
        severity_filter_button("Errors", SeverityFilter::Errors, ui.severity_filter),
        severity_filter_button("Warnings", SeverityFilter::Warnings, ui.severity_filter),
    ]
    .spacing(SPACING_XS);

    // Stats
    let error_count = report.error_count();
    let warning_count = report.warning_count();
    let stats_text = format!(
        "{} shown • {} errors, {} warnings total",
        filtered_count, error_count, warning_count
    );

    let stats = row![
        text(stats_text)
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().width(Length::Fill),
    ];

    // Build header column
    column![
        title_row,
        Space::new().height(SPACING_SM),
        filter_buttons,
        Space::new().height(SPACING_SM),
        stats,
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
    ]
    .into()
}

/// Severity filter button.
fn severity_filter_button<'a>(
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

    button(text(label).size(11))
        .on_press(Message::DomainEditor(DomainEditorMessage::Validation(
            ValidationMessage::SeverityFilterChanged(msg_filter),
        )))
        .padding([4.0, 8.0])
        .style(move |theme: &Theme, status| {
            if is_selected {
                let accent = theme.extended_palette().primary.base.color;
                let accent_light = theme.clinical().accent_primary_light;
                iced::widget::button::Style {
                    background: Some(accent_light.into()),
                    text_color: accent,
                    border: Border {
                        radius: 4.0.into(),
                        color: accent,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            } else {
                button_secondary(theme, status)
            }
        })
        .into()
}

// =============================================================================
// ISSUES LIST
// =============================================================================

/// Issues list in master panel.
pub(super) fn view_issues_list<'a>(
    issues: &[(usize, &'a Issue)],
    ui: &ValidationUiState,
) -> Element<'a, Message> {
    if issues.is_empty() {
        let filter_name = match ui.severity_filter {
            SeverityFilter::All => "issues",
            SeverityFilter::Errors => "errors",
            SeverityFilter::Warnings => "warnings",
            SeverityFilter::Info => "info messages",
        };
        return NoFilteredResults::new(format!("No {} found", filter_name))
            .hint("Try changing the severity filter")
            .height(200.0)
            .view();
    }

    let mut list = column![].spacing(2.0);

    for (original_idx, issue) in issues {
        let is_selected = ui.selected_issue == Some(*original_idx);
        list = list.push(view_issue_row(issue, *original_idx, is_selected));
    }

    scrollable(list.padding([0.0, SPACING_SM]))
        .height(Length::Fill)
        .into()
}

/// Single issue row in the master list using SelectableRow component.
fn view_issue_row<'a>(issue: &'a Issue, idx: usize, is_selected: bool) -> Element<'a, Message> {
    let severity = issue.severity();
    let severity_color = get_severity_color(severity);

    // Severity icon as leading element
    let severity_icon: Element<'a, Message> = match severity {
        Severity::Reject | Severity::Error => {
            lucide::circle_x().size(14).color(severity_color).into()
        }
        Severity::Warning => lucide::circle_alert().size(14).color(severity_color).into(),
        Severity::Info => lucide::info().size(14).color(severity_color).into(),
    };

    // Short description (truncated)
    let short_msg = truncate_message(issue.message().as_str(), 40);

    SelectableRow::new(
        issue.variable().to_string(),
        Message::DomainEditor(DomainEditorMessage::Validation(
            ValidationMessage::IssueSelected(idx),
        )),
    )
    .secondary(short_msg)
    .leading(severity_icon)
    .selected(is_selected)
    .view()
}
