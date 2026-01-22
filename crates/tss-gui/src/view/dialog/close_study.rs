//! Close study confirmation dialog view.
//!
//! Confirmation dialog shown when closing a study with potential unsaved changes.
//! Uses the semantic color system for accessibility mode support.

use iced::widget::{Space, button, column, container, row, text};
use iced::window;
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::message::{HomeMessage, Message};
use crate::theme::{ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, button_secondary};

/// Render the Close Study confirmation dialog content for a standalone window.
pub fn view_close_study_dialog_content<'a>(window_id: window::Id) -> Element<'a, Message> {
    let warning_icon =
        container(lucide::triangle_alert().size(48)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().warning.base.color),
            ..Default::default()
        });

    let title = text("Close Study?")
        .size(20)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });

    let message = text("All unsaved mapping progress will be lost.")
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    let cancel_button = button(text("Cancel").size(14))
        .on_press(Message::CloseWindow(window_id))
        .padding([10.0, 20.0])
        .style(button_secondary);

    let confirm_button = button(
        row![
            container(lucide::trash().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text("Close Study").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::CloseStudyConfirmed))
    .padding([10.0, 20.0])
    .style(|theme: &Theme, _status| iced::widget::button::Style {
        background: Some(theme.extended_palette().danger.base.color.into()),
        text_color: theme.clinical().text_on_accent,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let buttons = row![
        cancel_button,
        Space::new().width(SPACING_MD),
        confirm_button
    ]
    .align_y(Alignment::Center);

    let content = column![
        warning_icon,
        Space::new().height(SPACING_MD),
        title,
        Space::new().height(SPACING_SM),
        message,
        Space::new().height(SPACING_LG),
        buttons,
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG);

    // Wrap in a styled container for the window
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
            ..Default::default()
        })
        .into()
}
