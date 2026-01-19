//! Reusable UI components for the desktop menu bar.
//!
//! Provides menu items, separators, labels, and dropdown containers.

use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding, Theme};

use crate::message::Message;
use crate::theme::{
    BORDER_RADIUS_MD, GRAY_200, GRAY_400, GRAY_500, GRAY_600, GRAY_800, SPACING_SM, SPACING_XS,
    WHITE,
};

/// Render a menu item with optional icon and shortcut.
pub fn view_menu_item<'a>(
    icon: Element<'a, Message>,
    label: &'a str,
    shortcut: Option<&'a str>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let is_enabled = on_press.is_some();
    let text_color = if is_enabled { GRAY_800 } else { GRAY_600 };

    let content = row![
        container(icon).width(20),
        Space::new().width(SPACING_XS),
        text(label).size(13).color(text_color),
        Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let content = if let Some(shortcut) = shortcut {
        row![content, text(shortcut).size(11).color(GRAY_600),].align_y(Alignment::Center)
    } else {
        content
    };

    let btn = button(content)
        .padding([SPACING_XS, SPACING_SM])
        .width(Length::Fill)
        .style(menu_item_style);

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
        text(label).size(13).color(GRAY_400),
    ]
    .align_y(Alignment::Center);

    container(content)
        .padding([SPACING_XS, SPACING_SM])
        .width(Length::Fill)
        .into()
}

/// Render a menu label (non-clickable section header).
pub fn view_menu_label<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(11).color(GRAY_500))
        .padding([SPACING_XS, SPACING_SM])
        .into()
}

/// Render a menu separator line.
pub fn view_separator<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fill).height(1))
        .style(|_theme: &Theme| container::Style {
            background: Some(GRAY_200.into()),
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
        .style(|_theme: &Theme| container::Style {
            background: Some(WHITE.into()),
            border: Border {
                color: GRAY_200,
                width: 1.0,
                radius: BORDER_RADIUS_MD.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        })
        .padding(SPACING_XS)
        .into()
}

/// Style for menu items (transparent background, hover effects).
fn menu_item_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: GRAY_800,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
