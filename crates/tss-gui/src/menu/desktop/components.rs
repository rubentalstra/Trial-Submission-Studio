//! Reusable UI components for the desktop menu bar.
//!
//! Provides menu items, separators, labels, and dropdown containers.

use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding, Theme};

use crate::message::Message;
use crate::theme::{BORDER_RADIUS_MD, ClinicalColors, SPACING_SM, SPACING_XS};

/// Render a menu item with optional icon and shortcut.
pub fn view_menu_item<'a>(
    icon: Element<'a, Message>,
    label: &'a str,
    shortcut: Option<&'a str>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let is_enabled = on_press.is_some();

    let content = row![
        container(icon).width(20),
        Space::new().width(SPACING_XS),
        text(label).size(13).style(move |theme: &Theme| {
            let color = if is_enabled {
                theme.extended_palette().background.base.text
            } else {
                theme.clinical().text_muted
            };
            text::Style { color: Some(color) }
        }),
        Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let content = if let Some(shortcut) = shortcut {
        row![
            content,
            text(shortcut).size(11).style(|theme: &Theme| {
                text::Style {
                    color: Some(theme.clinical().text_muted),
                }
            }),
        ]
        .align_y(Alignment::Center)
    } else {
        content
    };

    let btn = button(content)
        .padding([SPACING_XS, SPACING_SM])
        .width(Length::Fill)
        .style(|theme: &Theme, _status: button::Status| button::Style {
            background: None,
            text_color: theme.extended_palette().background.base.text,
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    if let Some(msg) = on_press {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}

/// Render a disabled menu item.
pub fn view_menu_item_disabled<'a>(
    icon: Element<'a, Message>,
    label: &'a str,
) -> Element<'a, Message> {
    let content = row![
        container(icon).width(20),
        Space::new().width(SPACING_XS),
        text(label).size(13).style(|theme: &Theme| {
            text::Style {
                color: Some(theme.clinical().text_disabled),
            }
        }),
    ]
    .align_y(Alignment::Center);

    container(content)
        .padding([SPACING_XS, SPACING_SM])
        .width(Length::Fill)
        .into()
}

/// Render a menu label (non-clickable section header).
pub fn view_menu_label<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(11).style(|theme: &Theme| text::Style {
        color: Some(theme.clinical().text_muted),
    }))
    .padding([SPACING_XS, SPACING_SM])
    .into()
}

/// Render a menu separator line.
pub fn view_separator<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fill).height(1))
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().border_default.into()),
            ..Default::default()
        })
        .padding(Padding::from([SPACING_XS, 0.0]))
        .into()
}

/// Wrap dropdown content in a styled container with shadow.
pub fn view_dropdown_container<'a>(
    content: impl Into<Element<'a, Message>>,
    _left_offset: f32,
) -> Element<'a, Message> {
    container(content)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_elevated.into()),
            border: Border {
                color: theme.clinical().border_default,
                width: 1.0,
                radius: BORDER_RADIUS_MD.into(),
            },
            shadow: iced::Shadow {
                color: theme.clinical().shadow,
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        })
        .padding(SPACING_XS)
        .into()
}
