//! Detail panel components for the Validation tab.
//!
//! Contains the right-side issue detail view with metadata and actions.

use iced::widget::{Space, button, column, container, row, rule, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;
use tss_submit::{Issue, Severity};

use crate::component::display::MetadataCard;
use crate::component::panels::DetailHeader;
use crate::message::domain_editor::ValidationMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::theme::{
    ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, button_primary,
};

use super::helpers::{get_severity_color, issue_category};

// =============================================================================
// DETAIL PANEL
// =============================================================================

/// Detail view for selected issue.
pub(super) fn view_issue_detail<'a>(issue: &Issue) -> Element<'a, Message> {
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
        Severity::Info => lucide::info().size(12).color(severity_color).into(),
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

// =============================================================================
// METADATA CARD
// =============================================================================

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
            total_invalid,
            invalid_values,
            allowed_count,
            ..
        } => {
            metadata = metadata.row("Codelist", format!("{} ({})", codelist_name, codelist_code));
            metadata = metadata.row("Extensible", if *extensible { "Yes" } else { "No" });
            metadata = metadata.row("Invalid Values", total_invalid.to_string());
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

// =============================================================================
// DESCRIPTION SECTION
// =============================================================================

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

// =============================================================================
// ACTIONS
// =============================================================================

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
