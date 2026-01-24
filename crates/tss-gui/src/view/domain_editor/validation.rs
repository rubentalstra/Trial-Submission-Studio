//! Validation tab view.
//!
//! The validation tab displays CDISC conformance issues found during
//! validation of the mapped and normalized data.
//!
//! - **Left (Master)**: Filterable list of validation issues
//! - **Right (Detail)**: Detailed view of the selected issue

use iced::widget::{Space, button, column, container, row, rule, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;
use tss_submit::{Issue, Severity, ValidationReport};

use crate::component::display::{EmptyState, MetadataCard, NoFilteredResults, SelectableRow};
use crate::component::layout::SplitView;
use crate::component::panels::DetailHeader;
use crate::message::domain_editor::{SeverityFilter as MsgSeverityFilter, ValidationMessage};
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, SeverityFilter, ValidationUiState, ViewState};
use crate::theme::{
    ClinicalColors, MASTER_WIDTH, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, button_primary,
    button_secondary,
};

// =============================================================================
// MAIN VALIDATION TAB VIEW
// =============================================================================

/// Render the validation tab content.
pub fn view_validation_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return text("Domain not found")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                })
                .into();
        }
    };

    // Get validation UI state from view
    let validation_ui = match &state.view {
        ViewState::DomainEditor { validation_ui, .. } => validation_ui,
        _ => return text("Invalid view state").into(),
    };

    // Get validation cache from domain (persists across navigation)
    let Some(report) = &domain.validation_cache else {
        return view_no_validation_run();
    };

    // If validation passed (no issues), show success state
    if report.is_empty() {
        return view_validation_passed();
    }

    // Filter issues by severity
    let filtered_issues: Vec<(usize, &Issue)> = report
        .issues
        .iter()
        .enumerate()
        .filter(|(_, issue)| match validation_ui.severity_filter {
            SeverityFilter::All => true,
            SeverityFilter::Errors => {
                matches!(issue.severity(), Severity::Error | Severity::Reject)
            }
            SeverityFilter::Warnings => matches!(issue.severity(), Severity::Warning),
            SeverityFilter::Info => false, // No info level in current model
        })
        .collect();

    // Master panel header
    let master_header = view_master_header(report, validation_ui, filtered_issues.len());

    // Master panel content (list)
    let master_content = view_issues_list(&filtered_issues, validation_ui);

    // Detail panel
    let detail = if let Some(selected_idx) = validation_ui.selected_issue {
        if let Some(issue) = report.issues.get(selected_idx) {
            view_issue_detail(issue)
        } else {
            view_no_selection()
        }
    } else {
        view_no_selection()
    };

    SplitView::new(master_content, detail)
        .master_width(MASTER_WIDTH)
        .master_header(master_header)
        .view()
}

// =============================================================================
// MASTER PANEL
// =============================================================================

/// Master panel header with title, filters, and stats.
fn view_master_header<'a>(
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

/// Issues list in master panel.
fn view_issues_list<'a>(
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

/// Get the color for a severity level.
/// Returns a static Color that works across themes.
fn get_severity_color(severity: Severity) -> iced::Color {
    match severity {
        Severity::Reject | Severity::Error => iced::Color::from_rgb(0.90, 0.30, 0.25),
        Severity::Warning => iced::Color::from_rgb(0.95, 0.65, 0.15),
    }
}

// =============================================================================
// DETAIL PANEL
// =============================================================================

/// Detail view for selected issue.
fn view_issue_detail<'a>(issue: &Issue) -> Element<'a, Message> {
    let severity = issue.severity();
    let severity_color = get_severity_color(severity);

    let variable_name = issue.variable().to_string();
    let category = issue_category(issue);

    // Severity badge icon
    let badge_icon: Element<'a, Message> = match severity {
        Severity::Reject | Severity::Error => {
            lucide::circle_x().size(12).color(severity_color).into()
        }
        Severity::Warning => lucide::circle_alert().size(12).color(severity_color).into(),
    };

    // Header with variable name and severity badge
    let header = DetailHeader::new(variable_name.clone())
        .subtitle(category)
        .badge(badge_icon, severity.label(), severity_color)
        .view();

    // Issue information metadata card
    let metadata_card = view_issue_metadata(issue, severity, category);

    // Description section
    let description_section = view_issue_description(issue);

    // Action buttons
    let actions = view_issue_actions(&variable_name);

    // Build detail content (matching mapping.rs and normalization.rs pattern)
    scrollable(column![
        header,
        Space::new().height(SPACING_MD),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        metadata_card,
        Space::new().height(SPACING_LG),
        description_section,
        Space::new().height(SPACING_LG),
        actions,
        Space::new().height(SPACING_MD),
    ])
    .height(Length::Fill)
    .into()
}

/// Build metadata card for issue details.
fn view_issue_metadata<'a>(
    issue: &Issue,
    severity: Severity,
    category: &'static str,
) -> Element<'a, Message> {
    let mut metadata = MetadataCard::new()
        .row("Variable", issue.variable())
        .row("Severity", severity.label())
        .row("Category", category);

    // Add issue-specific details
    match issue {
        Issue::RequiredEmpty { null_count, .. } | Issue::IdentifierNull { null_count, .. } => {
            metadata = metadata.row("Null Values", null_count.to_string());
        }
        Issue::InvalidDate {
            invalid_count,
            samples,
            ..
        } => {
            metadata = metadata.row("Invalid Count", invalid_count.to_string());
            if !samples.is_empty() {
                metadata = metadata.row("Examples", samples.join(", "));
            }
        }
        Issue::TextTooLong {
            exceeded_count,
            max_found,
            max_allowed,
            ..
        } => {
            metadata = metadata.row("Exceeded Count", exceeded_count.to_string());
            metadata = metadata.row("Max Found", max_found.to_string());
            metadata = metadata.row("Max Allowed", max_allowed.to_string());
        }
        Issue::DataTypeMismatch {
            non_numeric_count,
            samples,
            ..
        } => {
            metadata = metadata.row("Invalid Count", non_numeric_count.to_string());
            if !samples.is_empty() {
                metadata = metadata.row("Examples", samples.join(", "));
            }
        }
        Issue::DuplicateSequence {
            duplicate_count, ..
        } => {
            metadata = metadata.row("Duplicate Count", duplicate_count.to_string());
        }
        Issue::CtViolation {
            codelist_code,
            codelist_name,
            extensible,
            invalid_count,
            invalid_values,
            allowed_count,
            ..
        } => {
            metadata = metadata.row("Codelist", format!("{} ({})", codelist_name, codelist_code));
            metadata = metadata.row("Extensible", if *extensible { "Yes" } else { "No" });
            metadata = metadata.row("Invalid Values", invalid_count.to_string());
            metadata = metadata.row("Allowed Terms", allowed_count.to_string());
            if !invalid_values.is_empty() {
                let display_values: Vec<&str> =
                    invalid_values.iter().take(5).map(String::as_str).collect();
                let suffix = if invalid_values.len() > 5 {
                    format!("... +{} more", invalid_values.len() - 5)
                } else {
                    String::new()
                };
                metadata = metadata.row(
                    "Examples",
                    format!("{}{}", display_values.join(", "), suffix),
                );
            }
        }
        _ => {}
    }

    metadata.view()
}

/// Build description section for issue details.
fn view_issue_description<'a>(issue: &Issue) -> Element<'a, Message> {
    let title_row = row![
        container(lucide::file_text().size(14)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_muted),
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text("Description")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
    ]
    .align_y(Alignment::Center);

    let message_text = issue.message();
    let message_box = container(
        text(message_text)
            .size(13)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
    )
    .padding(SPACING_SM)
    .width(Length::Fill)
    .style(|theme: &Theme| container::Style {
        background: Some(theme.clinical().background_secondary.into()),
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    column![title_row, Space::new().height(SPACING_XS), message_box,].into()
}

/// Build action buttons for issue details.
fn view_issue_actions<'a>(variable_name: &str) -> Element<'a, Message> {
    let variable = variable_name.to_string();
    let go_to_button = button(
        row![
            container(lucide::arrow_right().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            }),
            text("Go to Variable").size(13),
        ]
        .spacing(SPACING_XS)
        .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Validation(
        ValidationMessage::GoToIssueSource { variable },
    )))
    .padding([10.0, 16.0])
    .style(button_primary);

    row![go_to_button,].into()
}

// =============================================================================
// EMPTY STATES
// =============================================================================

/// Empty state when no issue is selected.
fn view_no_selection<'a>() -> Element<'a, Message> {
    EmptyState::new(
        container(lucide::mouse_pointer_click().size(48)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_disabled),
            ..Default::default()
        }),
        "Select an Issue",
    )
    .description("Choose a validation issue from the list to view details")
    .centered()
    .view()
}

/// Empty state when no validation has been run.
fn view_no_validation_run<'a>() -> Element<'a, Message> {
    EmptyState::new(
        container(lucide::shield_check().size(48)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_disabled),
            ..Default::default()
        }),
        "No Validation Results",
    )
    .description("Click 'Re-validate' to check for CDISC conformance issues")
    .action(
        "Run Validation",
        Message::DomainEditor(DomainEditorMessage::Validation(
            ValidationMessage::RefreshValidation,
        )),
    )
    .centered()
    .view()
}

/// Success state when validation passed.
fn view_validation_passed<'a>() -> Element<'a, Message> {
    let refresh_btn = button(
        row![lucide::refresh_cw().size(14), text("Re-validate").size(13),]
            .spacing(SPACING_XS)
            .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Validation(
        ValidationMessage::RefreshValidation,
    )))
    .padding([10.0, 16.0])
    .style(button_secondary);

    container(
        column![
            container(lucide::circle_check().size(64)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().success.base.color),
                ..Default::default()
            }),
            Space::new().height(SPACING_MD),
            text("All Checks Passed")
                .size(20)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().success.base.color),
                })
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                }),
            Space::new().height(SPACING_XS),
            text("No CDISC conformance issues were found")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
            Space::new().height(SPACING_LG),
            refresh_btn,
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}

// =============================================================================
// HELPERS
// =============================================================================

/// Get issue category for display.
fn issue_category(issue: &Issue) -> &'static str {
    match issue {
        Issue::RequiredMissing { .. }
        | Issue::RequiredEmpty { .. }
        | Issue::ExpectedMissing { .. }
        | Issue::IdentifierNull { .. } => "Presence",
        Issue::InvalidDate { .. } | Issue::TextTooLong { .. } => "Format",
        Issue::DataTypeMismatch { .. } => "Type",
        Issue::DuplicateSequence { .. }
        | Issue::UsubjidNotInDm { .. }
        | Issue::ParentNotFound { .. } => "Consistency",
        Issue::CtViolation { .. } => "Terminology",
    }
}

/// Truncate message for display in list.
fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() <= max_len {
        msg.to_string()
    } else {
        format!("{}...", &msg[..max_len])
    }
}
