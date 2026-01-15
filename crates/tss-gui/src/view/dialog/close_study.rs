//! Close study confirmation dialog view.
//!
//! Confirmation dialog shown when closing a study with potential unsaved changes.

use iced::widget::{Space, button, column, container, row, text};
use iced::window;
use iced::{Alignment, Border, Element, Length};
use iced_fonts::lucide;

use crate::message::{HomeMessage, Message};
use crate::theme::{
    GRAY_100, GRAY_600, GRAY_900, SPACING_LG, SPACING_MD, SPACING_SM, WARNING, WHITE,
    button_secondary,
};

/// Render the Close Study confirmation dialog content for a standalone window.
pub fn view_close_study_dialog_content<'a>(window_id: window::Id) -> Element<'a, Message> {
    let warning_icon = lucide::triangle_alert().size(48).color(WARNING);

    let title = text("Close Study?").size(20).color(GRAY_900);

    let message = text("All unsaved mapping progress will be lost.")
        .size(14)
        .color(GRAY_600);

    let cancel_button = button(text("Cancel").size(14))
        .on_press(Message::CloseWindow(window_id))
        .padding([10.0, 20.0])
        .style(button_secondary);

    let confirm_button = button(
        row![
            lucide::trash().size(14),
            Space::new().width(SPACING_SM),
            text("Close Study").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::CloseStudyConfirmed))
    .padding([10.0, 20.0])
    .style(|_theme, _status| iced::widget::button::Style {
        background: Some(iced::Color::from_rgb(0.75, 0.22, 0.17).into()), // Red
        text_color: WHITE,
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
        .style(|_| container::Style {
            background: Some(GRAY_100.into()),
            ..Default::default()
        })
        .into()
}
