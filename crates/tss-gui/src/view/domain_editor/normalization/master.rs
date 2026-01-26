//! Master panel components for the Normalization tab.
//!
//! Contains the left-side rule list with header.

use iced::widget::{Space, column, row, rule, text};
use iced::{Alignment, Color, Element, Theme};
use iced_fonts::lucide;

use crate::component::display::VariableListItem;
use crate::message::domain_editor::NormalizationMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{NormalizationUiState, SourceDomainState};
use crate::theme::{ClinicalColors, SPACING_SM, SPACING_XS};

use tss_submit::{NormalizationType, VariableStatus};

use super::helpers::get_transform_color;

// =============================================================================
// MASTER PANEL
// =============================================================================

pub(super) fn view_rules_header<'a>(
    total_rules: usize,
    rules: &[tss_submit::NormalizationRule],
) -> Element<'a, Message> {
    let title = text("Normalization Rules")
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    let auto_count = rules
        .iter()
        .filter(|r| {
            matches!(
                r.transform_type,
                NormalizationType::Constant
                    | NormalizationType::UsubjidPrefix
                    | NormalizationType::SequenceNumber
            )
        })
        .count();
    let transform_count = rules
        .iter()
        .filter(|r| {
            matches!(
                r.transform_type,
                NormalizationType::Iso8601DateTime
                    | NormalizationType::Iso8601Date
                    | NormalizationType::Iso8601Duration
                    | NormalizationType::StudyDay { .. }
                    | NormalizationType::NumericConversion
            )
        })
        .count();
    let ct_count = rules
        .iter()
        .filter(|r| matches!(r.transform_type, NormalizationType::CtNormalization { .. }))
        .count();

    let stats = row![
        text(format!("{} rules", total_rules))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
        Space::new().width(SPACING_SM),
        text("•").size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_disabled),
        }),
        Space::new().width(SPACING_SM),
        text(format!("{} auto", auto_count))
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().width(4.0),
        text(format!("{} transform", transform_count))
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().width(4.0),
        text(format!("{} CT", ct_count))
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
    ]
    .align_y(Alignment::Center);

    column![
        title,
        Space::new().height(SPACING_XS),
        stats,
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
    ]
    .into()
}

pub(super) fn view_rules_list<'a>(
    domain: &'a SourceDomainState,
    rules: &'a [tss_submit::NormalizationRule],
    ui_state: &'a NormalizationUiState,
) -> Element<'a, Message> {
    let mut items = column![].spacing(2.0);
    for (idx, rule) in rules.iter().enumerate() {
        let is_selected = ui_state.selected_rule == Some(idx);
        let var_status = domain.mapping.status(&rule.target_variable);
        items = items.push(view_rule_row(idx, rule, var_status, is_selected));
    }
    items.into()
}

fn view_rule_row<'a>(
    index: usize,
    rule: &'a tss_submit::NormalizationRule,
    var_status: VariableStatus,
    is_selected: bool,
) -> Element<'a, Message> {
    let icon_color = get_transform_color(&rule.transform_type);
    let type_label = get_transform_short_label(&rule.transform_type);

    // Build icon
    let icon: Element<'a, Message> = match &rule.transform_type {
        NormalizationType::Constant => lucide::hash().size(14).color(icon_color).into(),
        NormalizationType::UsubjidPrefix => lucide::user().size(14).color(icon_color).into(),
        NormalizationType::SequenceNumber => {
            lucide::list_ordered().size(14).color(icon_color).into()
        }
        NormalizationType::Iso8601DateTime | NormalizationType::Iso8601Date => {
            lucide::calendar().size(14).color(icon_color).into()
        }
        NormalizationType::Iso8601Duration => lucide::timer().size(14).color(icon_color).into(),
        NormalizationType::StudyDay { .. } => {
            lucide::calendar_days().size(14).color(icon_color).into()
        }
        NormalizationType::CtNormalization { .. } => {
            lucide::list().size(14).color(icon_color).into()
        }
        NormalizationType::NumericConversion => {
            lucide::calculator().size(14).color(icon_color).into()
        }
        NormalizationType::CopyDirect => lucide::copy().size(14).color(icon_color).into(),
        _ => lucide::wand_sparkles().size(14).color(icon_color).into(),
    };

    // Status dot color - use semantic colors
    let dot_color = get_status_dot_color(var_status);

    let mut item = VariableListItem::new(
        &rule.target_variable,
        Message::DomainEditor(DomainEditorMessage::Normalization(
            NormalizationMessage::RuleSelected(index),
        )),
    )
    .leading_icon(icon)
    .label(type_label)
    .selected(is_selected);

    // Add trailing status indicator as text for now
    // (VariableListItem doesn't support arbitrary trailing content, but we can use the badge)
    item = item.trailing_badge("●", dot_color);

    item.view()
}

/// Get the status dot color based on variable status.
/// This function returns a static Color that works across all themes.
fn get_status_dot_color(var_status: VariableStatus) -> Color {
    match var_status {
        VariableStatus::Accepted | VariableStatus::Suggested => {
            // Success green
            Color::from_rgb(0.20, 0.78, 0.35)
        }
        VariableStatus::AutoGenerated => {
            // Primary blue
            Color::from_rgb(0.13, 0.53, 0.90)
        }
        _ => {
            // Border default gray
            Color::from_rgb(0.75, 0.75, 0.78)
        }
    }
}

fn get_transform_short_label(transform_type: &NormalizationType) -> &'static str {
    match transform_type {
        NormalizationType::Constant => "Constant",
        NormalizationType::UsubjidPrefix => "USUBJID",
        NormalizationType::SequenceNumber => "Sequence",
        NormalizationType::Iso8601DateTime => "DateTime",
        NormalizationType::Iso8601Date => "Date",
        NormalizationType::Iso8601Duration => "Duration",
        NormalizationType::StudyDay { .. } => "Study Day",
        NormalizationType::CtNormalization { .. } => "CT",
        NormalizationType::NumericConversion => "Numeric",
        NormalizationType::CopyDirect => "Copy",
        _ => "Transform",
    }
}
