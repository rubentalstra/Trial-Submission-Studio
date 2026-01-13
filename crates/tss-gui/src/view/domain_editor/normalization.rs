//! Normalization tab view.
//!
//! The normalization tab displays data normalization rules that can be
//! enabled/disabled to standardize the output data format.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length};
use iced_fonts::lucide;

use crate::message::domain_editor::NormalizationMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::AppState;
use crate::theme::{
    GRAY_100, GRAY_400, GRAY_500, GRAY_600, GRAY_900, SPACING_LG, SPACING_MD, SPACING_SM,
    button_secondary,
};

// =============================================================================
// MAIN NORMALIZATION TAB VIEW
// =============================================================================

/// Render the normalization tab content.
pub fn view_normalization_tab<'a>(
    state: &'a AppState,
    domain_code: &'a str,
) -> Element<'a, Message> {
    let _domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return text("Domain not found").size(14).color(GRAY_500).into();
        }
    };

    // Header
    let header = view_header();

    // Rules list placeholder
    let content = view_rules_placeholder();

    column![header, Space::new().height(SPACING_MD), content,]
        .spacing(0)
        .into()
}

// =============================================================================
// HEADER
// =============================================================================

/// Header with title and refresh button.
fn view_header<'a>() -> Element<'a, Message> {
    let title = text("Data Normalization").size(18).color(GRAY_900);

    let subtitle = text("Configure rules to standardize data format for CDISC compliance")
        .size(13)
        .color(GRAY_600);

    let refresh_button = button(
        row![
            lucide::refresh_cw().size(12),
            text("Refresh Preview").size(14),
        ]
        .spacing(SPACING_SM)
        .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Normalization(
        NormalizationMessage::RefreshPreview,
    )))
    .padding([8.0, 16.0])
    .style(button_secondary);

    row![
        column![title, Space::new().height(4.0), subtitle,],
        Space::new().width(Length::Fill),
        refresh_button,
    ]
    .align_y(Alignment::Start)
    .into()
}

// =============================================================================
// RULES LIST (Placeholder)
// =============================================================================

/// Placeholder rules list.
fn view_rules_placeholder<'a>() -> Element<'a, Message> {
    container(
        column![
            lucide::settings().size(48).color(GRAY_400),
            Space::new().height(SPACING_MD),
            text("Normalization Rules").size(16).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Configure data normalization settings")
                .size(13)
                .color(GRAY_500),
            Space::new().height(SPACING_LG),
            text("Available rule categories:").size(13).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("• Date Format Standardization (ISO 8601)")
                .size(12)
                .color(GRAY_500),
            text("• Text Case Normalization (uppercase for CT)")
                .size(12)
                .color(GRAY_500),
            text("• Whitespace Trimming").size(12).color(GRAY_500),
            text("• Missing Value Standardization")
                .size(12)
                .color(GRAY_500),
            Space::new().height(SPACING_MD),
            text("Full configuration interface coming soon")
                .size(11)
                .color(GRAY_400),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(400.0))
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .style(|_theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
