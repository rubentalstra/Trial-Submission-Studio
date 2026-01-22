//! Reusable UI components for the desktop menu bar.
//!
//! Provides menu items, separators, labels, and dropdown containers.

use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Padding, Theme};

use crate::message::Message;
use crate::theme::{BORDER_RADIUS_MD, SPACING_SM, SPACING_XS, colors};

/// Render a menu item with optional icon and shortcut.
pub fn view_menu_item<'a>(
    icon: Element<'a, Message>,
    label: &'a str,
    shortcut: Option<&'a str>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let c = colors();
    let is_enabled = on_press.is_some();
    let text_color = if is_enabled {
        c.text_primary
    } else {
        c.text_muted
    };
    let text_muted = c.text_muted;
    let text_primary = c.text_primary;

    let content = row![
        container(icon).width(20),
        Space::new().width(SPACING_XS),
        text(label).size(13).color(text_color),
        Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let content = if let Some(shortcut) = shortcut {
        row![content, text(shortcut).size(11).color(text_muted),].align_y(Alignment::Center)
    } else {
        content
    };

    let btn = button(content)
        .padding([SPACING_XS, SPACING_SM])
        .width(Length::Fill)
        .style(
            move |_theme: &Theme, _status: button::Status| button::Style {
                background: None,
                text_color: text_primary,
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

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
    let c = colors();

    let content = row![
        container(icon).width(20),
        Space::new().width(SPACING_XS),
        text(label).size(13).color(c.text_disabled),
    ]
    .align_y(Alignment::Center);

    container(content)
        .padding([SPACING_XS, SPACING_SM])
        .width(Length::Fill)
        .into()
}

/// Render a menu label (non-clickable section header).
pub fn view_menu_label<'a>(label: &'a str) -> Element<'a, Message> {
    let c = colors();

    container(text(label).size(11).color(c.text_muted))
        .padding([SPACING_XS, SPACING_SM])
        .into()
}

/// Render a menu separator line.
pub fn view_separator<'a>() -> Element<'a, Message> {
    let c = colors();
    let border_default = c.border_default;

    container(Space::new().width(Length::Fill).height(1))
        .style(move |_theme: &Theme| container::Style {
            background: Some(border_default.into()),
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
    let c = colors();
    let bg_elevated = c.background_elevated;
    let border_default = c.border_default;

    let shadow_color = c.shadow;
    container(content)
        .style(move |_theme: &Theme| container::Style {
            background: Some(bg_elevated.into()),
            border: Border {
                color: border_default,
                width: 1.0,
                radius: BORDER_RADIUS_MD.into(),
            },
            shadow: iced::Shadow {
                color: shadow_color,
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        })
        .padding(SPACING_XS)
        .into()
}
