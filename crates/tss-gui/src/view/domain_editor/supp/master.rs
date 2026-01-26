//! Master panel components for the SUPP tab.
//!
//! Contains the left-side column list with search/filter header.

use iced::widget::{Space, button, column, row, rule, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::display::NoFilteredResults;
use crate::component::panels::FilterToggle;
use crate::message::domain_editor::SuppMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{SourceDomainState, SuppAction, SuppFilterMode, SuppUiState};
use crate::theme::{ALPHA_LIGHT, BORDER_RADIUS_SM, ClinicalColors, SPACING_SM, SPACING_XS};

// =============================================================================
// MASTER PANEL: HEADER (PINNED)
// =============================================================================

/// Left panel header: search, filters, and stats (pinned at top).
pub(super) fn build_master_header_pinned<'a>(
    ui: &'a SuppUiState,
    filtered_count: usize,
) -> Element<'a, Message> {
    // Search box
    let search = text_input("Search columns...", &ui.search_filter)
        .on_input(|s| {
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::SearchChanged(s)))
        })
        .padding([8.0, 12.0])
        .size(13);

    // Filter buttons
    let filters = build_filter_buttons(ui.filter_mode);

    // Stats
    let stats = row![
        text(format!("{}", filtered_count))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
        Space::new().width(4.0),
        text("columns").size(11).style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_muted),
        }),
    ]
    .align_y(Alignment::Center);

    column![
        search,
        Space::new().height(SPACING_XS),
        filters,
        Space::new().height(SPACING_SM),
        stats,
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
    ]
    .into()
}

// =============================================================================
// MASTER PANEL: CONTENT (SCROLLABLE)
// =============================================================================

/// Left panel content: scrollable list of columns.
pub(super) fn build_master_content<'a>(
    filtered: &[String],
    domain: &'a SourceDomainState,
    ui: &'a SuppUiState,
) -> Element<'a, Message> {
    if filtered.is_empty() {
        return NoFilteredResults::new("No columns match filter")
            .hint("Try adjusting your search or filter")
            .height(120.0)
            .view();
    }

    // Build column items
    let mut items = column![].spacing(SPACING_XS);

    for col_name in filtered {
        let supp_config = domain.supp_config.get(col_name);
        let action = supp_config.map_or(SuppAction::Pending, |c| c.action);
        let is_selected = ui.selected_column.as_deref() == Some(col_name.as_str());
        let item = build_column_item(col_name.clone(), action, is_selected);
        items = items.push(item);
    }

    items.into()
}

fn build_filter_buttons(current: SuppFilterMode) -> Element<'static, Message> {
    row![
        FilterToggle::new(
            "All",
            current == SuppFilterMode::All,
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::FilterModeChanged(
                SuppFilterMode::All
            )))
        )
        .view(),
        FilterToggle::new(
            "Pending",
            current == SuppFilterMode::Pending,
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::FilterModeChanged(
                SuppFilterMode::Pending
            )))
        )
        .view(),
        FilterToggle::new(
            "SUPP",
            current == SuppFilterMode::Included,
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::FilterModeChanged(
                SuppFilterMode::Included
            )))
        )
        .view(),
        FilterToggle::new(
            "Skip",
            current == SuppFilterMode::Skipped,
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::FilterModeChanged(
                SuppFilterMode::Skipped
            )))
        )
        .view(),
    ]
    .spacing(SPACING_XS)
    .into()
}

fn build_column_item(
    col_name: String,
    action: SuppAction,
    is_selected: bool,
) -> Element<'static, Message> {
    // Use static colors for status icons
    let status_icon: Element<'static, Message> = match action {
        SuppAction::Pending => lucide::circle()
            .size(10)
            .color(Color::from_rgb(0.65, 0.65, 0.70))
            .into(),
        SuppAction::Include => lucide::circle_check()
            .size(10)
            .color(Color::from_rgb(0.20, 0.78, 0.35))
            .into(),
        SuppAction::Skip => lucide::circle_x()
            .size(10)
            .color(Color::from_rgb(0.65, 0.65, 0.70))
            .into(),
    };

    let display_name = col_name.clone();

    button(
        row![
            status_icon,
            Space::new().width(SPACING_SM),
            text(display_name)
                .size(13)
                .style(move |theme: &Theme| text::Style {
                    color: Some(if is_selected {
                        theme.extended_palette().primary.base.color
                    } else {
                        theme.extended_palette().background.base.text
                    }),
                }),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
        SuppMessage::ColumnSelected(col_name),
    )))
    .padding([8.0, 12.0])
    .width(Length::Fill)
    .style(move |theme: &Theme, _status| {
        let accent_primary = theme.extended_palette().primary.base.color;
        let accent_light = Color {
            a: ALPHA_LIGHT,
            ..accent_primary
        };
        let bg_elevated = theme.clinical().background_elevated;

        let bg_color = if is_selected {
            accent_light
        } else {
            bg_elevated
        };

        iced::widget::button::Style {
            background: Some(bg_color.into()),
            text_color: if is_selected {
                accent_primary
            } else {
                theme.extended_palette().background.base.text
            },
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}
