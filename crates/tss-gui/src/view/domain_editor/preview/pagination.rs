//! Pagination components for the Preview tab.
//!
//! Contains pagination controls and rows per page selector.

use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Border, Color, Element, Theme};
use iced_fonts::lucide;

use crate::message::domain_editor::PreviewMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::theme::{
    ALPHA_LIGHT, BORDER_RADIUS_SM, ClinicalColors, SPACING_SM, SPACING_XS, ThemeConfig,
    button_ghost,
};

// =============================================================================
// PAGINATION CONTROLS
// =============================================================================

/// Pagination controls.
pub(super) fn view_pagination<'a>(
    config: &ThemeConfig,
    page: usize,
    total_rows: usize,
    page_size: usize,
) -> Element<'a, Message> {
    let theme = config.to_theme(false);
    let text_secondary = theme.clinical().text_secondary;
    let text_disabled = theme.clinical().text_disabled;
    let bg_elevated = theme.clinical().background_elevated;
    let border_default = theme.clinical().border_default;

    let total_pages = total_rows.div_ceil(page_size).max(1);

    let prev_enabled = page > 0;
    let next_enabled = page < total_pages.saturating_sub(1);

    // First page button
    let first_button = button(lucide::chevrons_left().size(14).color(if prev_enabled {
        text_secondary
    } else {
        text_disabled
    }))
    .on_press_maybe(if prev_enabled {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::GoToPage(0),
        )))
    } else {
        None
    })
    .padding([6.0, 8.0])
    .style(button_ghost);

    // Previous page button
    let prev_button = button(lucide::chevron_left().size(14).color(if prev_enabled {
        text_secondary
    } else {
        text_disabled
    }))
    .on_press_maybe(if prev_enabled {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::GoToPage(page - 1),
        )))
    } else {
        None
    })
    .padding([6.0, 8.0])
    .style(button_ghost);

    // Page info
    let start_row = page * page_size + 1;
    let end_row = ((page + 1) * page_size).min(total_rows);
    let page_info = container(
        text(format!(
            "{}-{} of {}",
            if total_rows == 0 { 0 } else { start_row },
            end_row,
            total_rows
        ))
        .size(12)
        .color(text_secondary),
    )
    .padding([6.0, 12.0])
    .style(move |_: &Theme| container::Style {
        background: Some(bg_elevated.into()),
        border: Border {
            color: border_default,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    });

    // Next page button
    let next_button = button(lucide::chevron_right().size(14).color(if next_enabled {
        text_secondary
    } else {
        text_disabled
    }))
    .on_press_maybe(if next_enabled {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::GoToPage(page + 1),
        )))
    } else {
        None
    })
    .padding([6.0, 8.0])
    .style(button_ghost);

    // Last page button
    let last_button = button(lucide::chevrons_right().size(14).color(if next_enabled {
        text_secondary
    } else {
        text_disabled
    }))
    .on_press_maybe(if next_enabled {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::GoToPage(total_pages.saturating_sub(1)),
        )))
    } else {
        None
    })
    .padding([6.0, 8.0])
    .style(button_ghost);

    row![
        first_button,
        prev_button,
        Space::new().width(SPACING_XS),
        page_info,
        Space::new().width(SPACING_XS),
        next_button,
        last_button,
    ]
    .align_y(Alignment::Center)
    .into()
}

// =============================================================================
// ROWS PER PAGE SELECTOR
// =============================================================================

/// Rows per page selector.
pub(super) fn view_rows_per_page_selector<'a>(
    config: &ThemeConfig,
    current: usize,
) -> Element<'a, Message> {
    let theme = config.to_theme(false);
    let text_secondary = theme.clinical().text_secondary;
    let accent_primary = theme.extended_palette().primary.base.color;
    let bg_elevated = theme.clinical().background_elevated;
    let border_default = theme.clinical().border_default;

    // Create a lighter accent background for selected state
    let accent_light = Color {
        a: ALPHA_LIGHT,
        ..accent_primary
    };

    let options = [25, 50, 100, 200];

    let label = text("Rows:").size(12).color(text_secondary);

    let buttons: Vec<Element<'a, Message>> = options
        .iter()
        .map(|&n| {
            let is_selected = current == n;
            button(text(format!("{}", n)).size(11).color(if is_selected {
                accent_primary
            } else {
                text_secondary
            }))
            .on_press(Message::DomainEditor(DomainEditorMessage::Preview(
                PreviewMessage::RowsPerPageChanged(n),
            )))
            .padding([4.0, 8.0])
            .style(move |_: &Theme, _status| {
                if is_selected {
                    iced::widget::button::Style {
                        background: Some(accent_light.into()),
                        text_color: accent_primary,
                        border: Border {
                            color: accent_primary,
                            width: 1.0,
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        ..Default::default()
                    }
                } else {
                    iced::widget::button::Style {
                        background: Some(bg_elevated.into()),
                        text_color: text_secondary,
                        border: Border {
                            color: border_default,
                            width: 1.0,
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        ..Default::default()
                    }
                }
            })
            .into()
        })
        .collect();

    row![label, Space::new().width(SPACING_SM),]
        .push(row(buttons).spacing(4.0))
        .align_y(Alignment::Center)
        .into()
}
