//! Tab bar navigation component.
//!
//! Horizontal tab navigation for switching between views or panels.

use iced::widget::{button, container, row, text};
use iced::{Border, Element, Length};

use crate::theme::{
    BORDER_RADIUS_SM, GRAY_100, GRAY_200, GRAY_600, PRIMARY_100, PRIMARY_500, PRIMARY_700,
    TAB_PADDING_X, TAB_PADDING_Y,
};

// =============================================================================
// TAB DEFINITION
// =============================================================================

/// A tab item for the tab bar.
pub struct Tab<M> {
    /// Tab label text
    pub label: String,
    /// Message to send when tab is clicked
    pub message: M,
}

impl<M> Tab<M> {
    /// Create a new tab.
    pub fn new(label: impl Into<String>, message: M) -> Self {
        Self {
            label: label.into(),
            message,
        }
    }
}

// =============================================================================
// TAB BAR COMPONENT
// =============================================================================

/// Creates a horizontal tab bar.
///
/// Renders a row of tab buttons with the active tab highlighted.
///
/// # Arguments
///
/// * `tabs` - List of tabs to display
/// * `active_index` - Index of the currently active tab
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::{tab_bar, Tab};
///
/// let tabs = vec![
///     Tab::new("Mapping", Message::TabSelected(0)),
///     Tab::new("Transform", Message::TabSelected(1)),
///     Tab::new("Validation", Message::TabSelected(2)),
/// ];
///
/// let bar = tab_bar(tabs, state.active_tab);
/// ```
pub fn tab_bar<'a, M: Clone + 'a>(tabs: Vec<Tab<M>>, active_index: usize) -> Element<'a, M> {
    let mut tab_row = row![].spacing(0);

    for (index, tab) in tabs.into_iter().enumerate() {
        let is_active = index == active_index;
        let label = tab.label.clone();

        let tab_button = button(
            container(text(label).size(14))
                .padding([TAB_PADDING_Y, TAB_PADDING_X])
                .center_x(Length::Shrink),
        )
        .on_press(tab.message)
        .style(move |theme, status| {
            if is_active {
                tab_style_active(theme, status)
            } else {
                tab_style_inactive(theme, status)
            }
        });

        tab_row = tab_row.push(tab_button);
    }

    container(tab_row)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(GRAY_100.into()),
            border: Border {
                color: GRAY_200,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// Creates a tab bar with rounded container.
///
/// Variant with rounded corners for use in cards or panels.
pub fn tab_bar_rounded<'a, M: Clone + 'a>(
    tabs: Vec<Tab<M>>,
    active_index: usize,
) -> Element<'a, M> {
    let mut tab_row = row![].spacing(2.0);

    for (index, tab) in tabs.into_iter().enumerate() {
        let is_active = index == active_index;
        let label = tab.label.clone();

        let tab_button = button(
            container(text(label).size(13))
                .padding([6.0, 12.0])
                .center_x(Length::Shrink),
        )
        .on_press(tab.message)
        .style(move |theme, status| {
            if is_active {
                tab_style_active_rounded(theme, status)
            } else {
                tab_style_inactive_rounded(theme, status)
            }
        });

        tab_row = tab_row.push(tab_button);
    }

    container(tab_row)
        .padding(4.0)
        .style(|_theme| container::Style {
            background: Some(GRAY_100.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

// =============================================================================
// TAB STYLES
// =============================================================================

fn tab_style_active(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(PRIMARY_100.into()),
        text_color: PRIMARY_700,
        border: Border {
            color: PRIMARY_500,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn tab_style_inactive(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(GRAY_200.into()),
        _ => None,
    };

    button::Style {
        background: bg,
        text_color: GRAY_600,
        border: Border::default(),
        ..Default::default()
    }
}

fn tab_style_active_rounded(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(PRIMARY_100.into()),
        text_color: PRIMARY_700,
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn tab_style_inactive_rounded(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(GRAY_200.into()),
        _ => None,
    };

    button::Style {
        background: bg,
        text_color: GRAY_600,
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
