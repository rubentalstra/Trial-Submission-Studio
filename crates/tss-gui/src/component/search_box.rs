//! Search box component.
//!
//! A text input with search icon and clear button.

use iced::widget::{Space, button, container, row, text_input};
use iced::{Border, Element, Length, Padding};
use iced_fonts::lucide;

use crate::theme::{
    BORDER_RADIUS_SM, GRAY_200, GRAY_400, GRAY_500, SPACING_SM, SPACING_XS, button_ghost,
    text_input_default,
};

// =============================================================================
// SEARCH BOX
// =============================================================================

/// Creates a search input with clear button.
///
/// Shows a search icon prefix and a clear button when text is entered.
///
/// # Arguments
///
/// * `value` - Current search text
/// * `placeholder` - Placeholder text
/// * `on_change` - Message factory for text changes
/// * `on_clear` - Message to send when clear button is clicked
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::search_box;
///
/// let search = search_box(
///     &state.search_query,
///     "Search variables...",
///     Message::SearchChanged,
///     Message::SearchCleared,
/// );
/// ```
pub fn search_box<'a, M: Clone + 'a>(
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) -> M + 'a,
    on_clear: M,
) -> Element<'a, M> {
    // Search icon (magnifying glass)
    let search_icon = lucide::search().size(14).color(GRAY_400);

    // Text input (extra left padding for icon)
    let input = text_input(placeholder, value)
        .on_input(on_change)
        .padding(Padding::new(8.0).left(32.0))
        .width(Length::Fill)
        .style(text_input_default);

    // Clear button (only shown when there's text)
    let clear_button = if value.is_empty() {
        None
    } else {
        Some(
            button(lucide::x().size(16).color(GRAY_500))
                .on_press(on_clear)
                .padding([4.0, 8.0])
                .style(button_ghost),
        )
    };

    // Layout: [icon][input][clear?]
    let mut content = row![
        container(search_icon)
            .width(Length::Fixed(32.0))
            .center_x(Length::Shrink)
            .center_y(Length::Shrink),
    ];

    // The input overlays the icon area
    content = content.push(container(input).width(Length::Fill));

    if let Some(btn) = clear_button {
        content = content.push(btn);
    }

    container(content)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Color::WHITE.into()),
            border: Border {
                color: GRAY_200,
                width: 1.0,
                radius: BORDER_RADIUS_SM.into(),
            },
            ..Default::default()
        })
        .into()
}

/// Creates a compact search box.
///
/// Smaller variant for use in toolbars or sidebars.
pub fn search_box_compact<'a, M: Clone + 'a>(
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) -> M + 'a,
    on_clear: M,
) -> Element<'a, M> {
    let input = text_input(placeholder, value)
        .on_input(on_change)
        .padding([6.0, 8.0])
        .size(13)
        .width(Length::Fill)
        .style(text_input_default);

    let clear_button = if value.is_empty() {
        None
    } else {
        Some(
            button(lucide::x().size(14).color(GRAY_500))
                .on_press(on_clear)
                .padding([2.0, 6.0])
                .style(button_ghost),
        )
    };

    let mut content = row![input].spacing(SPACING_XS);

    if let Some(btn) = clear_button {
        content = content.push(btn);
    }

    container(content).width(Length::Fill).into()
}

/// Creates a search box with filter toggle.
///
/// Includes a filter button for additional filtering options.
pub fn search_box_with_filter<'a, M: Clone + 'a>(
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) -> M + 'a,
    on_clear: M,
    filter_active: bool,
    on_filter_toggle: M,
) -> Element<'a, M> {
    let input = text_input(placeholder, value)
        .on_input(on_change)
        .padding([8.0, 12.0])
        .width(Length::Fill)
        .style(text_input_default);

    let clear_button: Element<'a, M> = if value.is_empty() {
        Space::new().width(0.0).into()
    } else {
        button(lucide::x().size(16).color(GRAY_500))
            .on_press(on_clear)
            .padding([4.0, 8.0])
            .style(button_ghost)
            .into()
    };

    // Filter button
    let filter_icon_color = if filter_active {
        crate::theme::PRIMARY_500
    } else {
        GRAY_400
    };

    let filter_button: Element<'a, M> = button(lucide::funnel().size(14).color(filter_icon_color))
        .on_press(on_filter_toggle)
        .padding([4.0, 8.0])
        .style(button_ghost)
        .into();

    row![input, clear_button, filter_button]
        .spacing(SPACING_SM)
        .align_y(iced::Alignment::Center)
        .into()
}
