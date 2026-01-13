//! Preview tab view.
//!
//! The preview tab displays a paginated data table showing the
//! transformed output data.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length};

use crate::message::domain_editor::PreviewMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, ViewState};
use crate::theme::{
    GRAY_100, GRAY_400, GRAY_500, GRAY_600, GRAY_900, SPACING_LG, SPACING_MD, SPACING_SM,
    button_primary,
};

// =============================================================================
// MAIN PREVIEW TAB VIEW
// =============================================================================

/// Render the preview tab content.
pub fn view_preview_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let _domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return text("Domain not found").size(14).color(GRAY_500).into();
        }
    };

    // Get preview UI state and cached DataFrame
    let preview_cache = match &state.view {
        ViewState::DomainEditor { preview_cache, .. } => preview_cache,
        _ => return text("Invalid view state").into(),
    };

    // Header
    let header = view_preview_header();

    // Content
    let content = if preview_cache.is_some() {
        view_preview_content()
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

/// Preview header.
fn view_preview_header<'a>() -> Element<'a, Message> {
    let title = text("Data Preview").size(18).color(GRAY_900);

    let subtitle = text("Preview of transformed SDTM output data")
        .size(13)
        .color(GRAY_600);

    let rebuild_button = button(
        row![
            text("\u{f021}") // refresh icon
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(12),
            text("Rebuild Preview").size(14),
        ]
        .spacing(SPACING_SM)
        .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Preview(
        PreviewMessage::RebuildPreview,
    )))
    .padding([8.0, 16.0])
    .style(button_primary);

    row![
        column![title, Space::new().height(4.0), subtitle,],
        Space::new().width(Length::Fill),
        rebuild_button,
    ]
    .align_y(Alignment::Start)
    .into()
}

// =============================================================================
// STATES
// =============================================================================

/// Empty state when no preview is available.
fn view_empty_state<'a>() -> Element<'a, Message> {
    container(
        column![
            text("\u{f1c0}") // database icon
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(48)
                .color(GRAY_400),
            Space::new().height(SPACING_MD),
            text("No Preview Available").size(16).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Click 'Rebuild Preview' to generate the output preview")
                .size(13)
                .color(GRAY_500),
            Space::new().height(SPACING_LG),
            button(
                row![
                    text("\u{f021}") // refresh icon
                        .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                        .size(12),
                    text("Rebuild Preview").size(14),
                ]
                .spacing(SPACING_SM)
                .align_y(Alignment::Center),
            )
            .on_press(Message::DomainEditor(DomainEditorMessage::Preview(
                PreviewMessage::RebuildPreview,
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

/// Preview content placeholder.
fn view_preview_content<'a>() -> Element<'a, Message> {
    container(
        column![
            text("Preview data loaded").size(14).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Full data table coming soon").size(12).color(GRAY_500),
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
