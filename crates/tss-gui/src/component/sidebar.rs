//! Sidebar navigation component.
//!
//! A vertical sidebar for domain/feature navigation.

use iced::widget::{button, column, container, scrollable, space, text};
use iced::{Border, Color, Element, Length, Padding};

use crate::theme::{BORDER_RADIUS_SM, SIDEBAR_WIDTH, SPACING_SM, SPACING_XS, colors};

// =============================================================================
// SIDEBAR ITEM
// =============================================================================

/// A sidebar navigation item.
pub struct SidebarItem<M> {
    /// Item label text
    pub label: String,
    /// Optional badge text (e.g., error count)
    pub badge: Option<String>,
    /// Message to send when clicked
    pub message: M,
}

impl<M> SidebarItem<M> {
    /// Create a new sidebar item.
    pub fn new(label: impl Into<String>, message: M) -> Self {
        Self {
            label: label.into(),
            badge: None,
            message,
        }
    }

    /// Add a badge to the item.
    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

// =============================================================================
// SIDEBAR COMPONENT
// =============================================================================

/// Creates a vertical sidebar navigation.
///
/// Renders a column of navigation items with optional badges.
///
/// # Arguments
///
/// * `items` - List of sidebar items
/// * `active_index` - Index of the currently active item (or None)
/// * `width` - Width of the sidebar in pixels
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::{sidebar, SidebarItem};
///
/// let items = vec![
///     SidebarItem::new("DM", Message::DomainSelected("DM")),
///     SidebarItem::new("AE", Message::DomainSelected("AE"))
///         .with_badge("3"),
///     SidebarItem::new("CM", Message::DomainSelected("CM")),
/// ];
///
/// let nav = sidebar(items, Some(0), 280.0);
/// ```
pub fn sidebar<'a, M: Clone + 'a>(
    items: Vec<SidebarItem<M>>,
    active_index: Option<usize>,
    width: f32,
) -> Element<'a, M> {
    let c = colors();
    let bg_primary = c.background_primary;
    let bg_secondary = c.background_secondary;
    let border_default = c.border_default;
    let text_secondary = c.text_secondary;
    let text_primary = c.text_primary;
    let accent_primary = c.accent_primary;
    let accent_pressed = c.accent_pressed;
    // Light tint of accent for active background
    let accent_light = Color {
        a: 0.15,
        ..accent_primary
    };

    let mut item_column = column![].spacing(SPACING_XS);

    for (index, item) in items.into_iter().enumerate() {
        let is_active = active_index == Some(index);
        let label = item.label.clone();

        // Item content with optional badge
        let item_content = if let Some(badge) = item.badge {
            iced::widget::row![
                text(label).size(14).color(if is_active {
                    accent_pressed
                } else {
                    text_secondary
                }),
                space::horizontal(),
                container(text(badge).size(11).color(text_secondary))
                    .padding([2.0, 6.0])
                    .style(move |_theme| container::Style {
                        background: Some(border_default.into()),
                        border: Border {
                            radius: 10.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            ]
            .align_y(iced::Alignment::Center)
        } else {
            iced::widget::row![text(label).size(14).color(if is_active {
                accent_pressed
            } else {
                text_secondary
            })]
        };

        let item_button = button(
            container(item_content)
                .padding([SPACING_SM, 12.0])
                .width(Length::Fill),
        )
        .on_press(item.message)
        .width(Length::Fill)
        .style(move |_theme, status| {
            if is_active {
                button::Style {
                    background: Some(accent_light.into()),
                    text_color: accent_pressed,
                    border: Border {
                        color: accent_primary,
                        width: 0.0,
                        radius: BORDER_RADIUS_SM.into(),
                    },
                    ..Default::default()
                }
            } else {
                let bg = match status {
                    button::Status::Hovered => Some(bg_secondary.into()),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    text_color: text_primary,
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
        });

        item_column = item_column.push(item_button);
    }

    container(scrollable(item_column).height(Length::Fill))
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .padding(SPACING_SM)
        .style(move |_theme| container::Style {
            background: Some(bg_primary.into()),
            border: Border {
                color: border_default,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// Creates a sidebar with default width.
pub fn sidebar_default<'a, M: Clone + 'a>(
    items: Vec<SidebarItem<M>>,
    active_index: Option<usize>,
) -> Element<'a, M> {
    sidebar(items, active_index, SIDEBAR_WIDTH)
}

/// Creates a sidebar with a header.
pub fn sidebar_with_header<'a, M: Clone + 'a>(
    header: Element<'a, M>,
    items: Vec<SidebarItem<M>>,
    active_index: Option<usize>,
    width: f32,
) -> Element<'a, M> {
    let c = colors();
    let bg_primary = c.background_primary;
    let bg_secondary = c.background_secondary;
    let border_default = c.border_default;
    let text_secondary = c.text_secondary;
    let text_primary = c.text_primary;
    let accent_primary = c.accent_primary;
    let accent_pressed = c.accent_pressed;
    // Light tint of accent for active background
    let accent_light = Color {
        a: 0.15,
        ..accent_primary
    };

    let mut item_column = column![].spacing(SPACING_XS);

    for (index, item) in items.into_iter().enumerate() {
        let is_active = active_index == Some(index);
        let label = item.label.clone();

        let item_button = button(
            container(text(label).size(14).color(if is_active {
                accent_pressed
            } else {
                text_secondary
            }))
            .padding([SPACING_SM, 12.0])
            .width(Length::Fill),
        )
        .on_press(item.message)
        .width(Length::Fill)
        .style(move |_theme, status| {
            if is_active {
                button::Style {
                    background: Some(accent_light.into()),
                    text_color: accent_pressed,
                    border: Border {
                        color: accent_primary,
                        width: 0.0,
                        radius: BORDER_RADIUS_SM.into(),
                    },
                    ..Default::default()
                }
            } else {
                let bg = match status {
                    button::Status::Hovered => Some(bg_secondary.into()),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    text_color: text_primary,
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
        });

        item_column = item_column.push(item_button);
    }

    container(
        column![
            container(header).padding(Padding::new(SPACING_SM).bottom(0.0)),
            scrollable(container(item_column).padding(Padding::new(SPACING_SM).top(0.0)))
                .height(Length::Fill),
        ]
        .spacing(SPACING_SM),
    )
    .width(Length::Fixed(width))
    .height(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(bg_primary.into()),
        border: Border {
            color: border_default,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}
