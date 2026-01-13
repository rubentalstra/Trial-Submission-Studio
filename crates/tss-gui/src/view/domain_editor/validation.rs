//! Validation tab view.
//!
//! The validation tab displays CDISC conformance issues found during
//! validation of the mapped and normalized data.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length};

use crate::message::domain_editor::ValidationMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, ViewState};
use crate::theme::{
    GRAY_100, GRAY_400, GRAY_500, GRAY_600, GRAY_900, SPACING_LG, SPACING_MD, SPACING_SM, SUCCESS,
    button_primary,
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

    // Get validation cache
    let validation_cache = match &state.view {
        ViewState::DomainEditor {
            validation_cache, ..
        } => validation_cache,
        _ => return text("Invalid view state").into(),
    };

    // Header
    let header = view_validation_header();

    // Content
    let content = if validation_cache.is_some() {
        view_validation_content()
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

/// Validation header.
fn view_validation_header<'a>() -> Element<'a, Message> {
    let title = text("Validation Issues").size(18).color(GRAY_900);

    let subtitle = text("Review CDISC conformance issues")
        .size(13)
        .color(GRAY_600);

    let refresh_button = button(
        row![
            text("\u{f021}") // refresh icon
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(12),
            text("Re-validate").size(14),
        ]
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
        refresh_button,
    ]
    .align_y(Alignment::Start)
    .into()
}

// =============================================================================
// STATES
// =============================================================================

/// Empty state when no validation has been run.
fn view_empty_state<'a>() -> Element<'a, Message> {
    container(
        column![
            text("\u{f058}") // check-circle
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(48)
                .color(GRAY_400),
            Space::new().height(SPACING_MD),
            text("No Validation Results").size(16).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Click 'Re-validate' to check for CDISC conformance issues")
                .size(13)
                .color(GRAY_500),
            Space::new().height(SPACING_LG),
            button(
                row![
                    text("\u{f021}") // refresh icon
                        .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                        .size(12),
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

/// Validation content placeholder.
fn view_validation_content<'a>() -> Element<'a, Message> {
    container(
        column![
            text("\u{f058}") // check-circle
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(32)
                .color(SUCCESS),
            Space::new().height(SPACING_SM),
            text("Validation complete").size(14).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Detailed issue list coming soon")
                .size(12)
                .color(GRAY_500),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(300.0))
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
