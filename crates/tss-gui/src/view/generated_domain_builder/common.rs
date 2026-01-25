//! Shared UI components specific to generated domain builders.
//!
//! Entry list components used across CO, RELREC, RELSPEC, and RELSUB builders.
//! For general form fields and other components, use `crate::component::*`.

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::message::Message;
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
};

// =============================================================================
// ENTRY LIST COMPONENTS
// =============================================================================

/// Container for the entry list with header.
pub fn entry_list_container<'a>(
    title: &'a str,
    count: usize,
    entries: Element<'a, Message>,
) -> Element<'a, Message> {
    let header = row![
        text(title).size(14).style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        }),
        Space::new().width(Length::Fill),
        text(format!("{} entries", count))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
    ]
    .align_y(Alignment::Center);

    column![
        header,
        Space::new().height(SPACING_SM),
        container(scrollable(entries).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.clinical().background_secondary.into()),
                border: Border {
                    width: 1.0,
                    radius: BORDER_RADIUS_SM.into(),
                    color: theme.clinical().border_subtle,
                },
                ..Default::default()
            }),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Entry row for the list.
pub fn entry_row<'a>(
    primary: impl Into<String>,
    secondary: impl Into<String>,
    index: usize,
    on_remove: impl Fn(usize) -> Message + 'a,
) -> Element<'a, Message> {
    let primary_text = text(primary.into())
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });

    let secondary_text = text(secondary.into())
        .size(12)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    let remove_btn =
        button(
            container(lucide::trash().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().danger.base.color),
                ..Default::default()
            }),
        )
        .on_press(on_remove(index))
        .padding(SPACING_XS)
        .style(|theme: &Theme, status| {
            let base = button::Style {
                background: None,
                text_color: theme.extended_palette().danger.base.color,
                border: Border::default(),
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(theme.clinical().background_elevated.into()),
                    ..base
                },
                _ => base,
            }
        });

    container(
        row![
            column![primary_text, secondary_text,].width(Length::Fill),
            remove_btn,
        ]
        .align_y(Alignment::Center)
        .padding([SPACING_SM, SPACING_MD]),
    )
    .width(Length::Fill)
    .style(|theme: &Theme| container::Style {
        border: Border {
            width: 0.0,
            radius: 0.0.into(),
            color: theme.clinical().border_subtle,
        },
        ..Default::default()
    })
    .into()
}

/// Empty state for entry list.
pub fn entry_list_empty<'a>(message: &'a str) -> Element<'a, Message> {
    container(text(message).size(14).style(|theme: &Theme| text::Style {
        color: Some(theme.clinical().text_muted),
    }))
    .width(Length::Fill)
    .padding(SPACING_LG)
    .center_x(Length::Fill)
    .into()
}

/// Truncate string with ellipsis.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
