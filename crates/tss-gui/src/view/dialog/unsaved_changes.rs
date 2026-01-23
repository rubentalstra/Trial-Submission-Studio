//! Unsaved changes confirmation dialog view.
//!
//! Dialog shown when the user has unsaved changes and wants to:
//! - Create a new project
//! - Open another project
//! - Quit the application
//!
//! Offers three options: Save, Don't Save, Cancel.

use iced::widget::{Space, button, column, container, row, text};
use iced::window;
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::message::Message;
use crate::theme::{ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, button_secondary};

/// Render the unsaved changes dialog content for a standalone window.
pub fn view_unsaved_changes_dialog_content<'a>(window_id: window::Id) -> Element<'a, Message> {
    let warning_icon =
        container(lucide::triangle_alert().size(48)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().warning.base.color),
            ..Default::default()
        });

    let title = text("Unsaved Changes")
        .size(20)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });

    let message_text = text("Do you want to save your changes before continuing?")
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    let subtitle = text("Your changes will be lost if you don't save them.")
        .size(12)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    // Cancel button - closes dialog without action
    let cancel_button = button(text("Cancel").size(14))
        .on_press(Message::CloseWindow(window_id))
        .padding([10.0, 16.0])
        .style(button_secondary);

    // Don't Save button - proceed without saving
    let dont_save_button = button(text("Don't Save").size(14))
        .on_press(Message::UnsavedChangesDiscard)
        .padding([10.0, 16.0])
        .style(|theme: &Theme, _status| iced::widget::button::Style {
            background: Some(theme.extended_palette().danger.weak.color.into()),
            text_color: theme.extended_palette().danger.base.text,
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    // Save button - save then proceed
    let save_button = button(
        row![
            container(lucide::save().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text("Save").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::UnsavedChangesSave)
    .padding([10.0, 20.0])
    .style(|theme: &Theme, _status| iced::widget::button::Style {
        background: Some(theme.extended_palette().primary.base.color.into()),
        text_color: theme.clinical().text_on_accent,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let buttons = row![
        cancel_button,
        Space::new().width(SPACING_SM),
        dont_save_button,
        Space::new().width(SPACING_SM),
        save_button
    ]
    .align_y(Alignment::Center);

    let content = column![
        warning_icon,
        Space::new().height(SPACING_MD),
        title,
        Space::new().height(SPACING_SM),
        message_text,
        Space::new().height(4),
        subtitle,
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
            background: Some(theme.clinical().background_elevated.into()),
            ..Default::default()
        })
        .into()
}
