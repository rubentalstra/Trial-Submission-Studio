//! Sidebar navigation component.
//!
//! A vertical sidebar for domain/feature navigation.

use iced::widget::{button, column, container, scrollable, space, text};
use iced::{Border, Element, Length, Padding, Theme};

use crate::theme::{BORDER_RADIUS_SM, ClinicalColors, SIDEBAR_WIDTH, SPACING_SM, SPACING_XS};

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
/// Uses Iced's theme system - colors are resolved inside style closures.
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
    let mut item_column = column![].spacing(SPACING_XS);

    for (index, item) in items.into_iter().enumerate() {
        let is_active = active_index == Some(index);
        let label = item.label.clone();

        // Item content with optional badge
        let item_content = if let Some(badge) = item.badge {
            iced::widget::row![
                text(label).size(14).style(move |theme: &Theme| {
                    let clinical = theme.clinical();
                    text::Style {
                        color: Some(if is_active {
                            clinical.accent_pressed
                        } else {
                            clinical.text_secondary
                        }),
                    }
                }),
                space::horizontal(),
                container(text(badge).size(11).style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_secondary),
                }))
                .padding([2.0, 6.0])
                .style(|theme: &Theme| {
                    let clinical = theme.clinical();
                    container::Style {
                        background: Some(clinical.border_default.into()),
                        border: Border {
                            radius: 10.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
            ]
            .align_y(iced::Alignment::Center)
        } else {
            iced::widget::row![text(label).size(14).style(move |theme: &Theme| {
                let clinical = theme.clinical();
                text::Style {
                    color: Some(if is_active {
                        clinical.accent_pressed
                    } else {
                        clinical.text_secondary
                    }),
                }
            })]
        };

        let item_button = button(
            container(item_content)
                .padding([SPACING_SM, 12.0])
                .width(Length::Fill),
        )
        .on_press(item.message)
        .width(Length::Fill)
        .style(move |theme: &Theme, status| {
            let palette = theme.extended_palette();
            let clinical = theme.clinical();

            if is_active {
                button::Style {
                    background: Some(clinical.accent_primary_light.into()),
                    text_color: clinical.accent_pressed,
                    border: Border {
                        color: palette.primary.base.color,
                        width: 0.0,
                        radius: BORDER_RADIUS_SM.into(),
                    },
                    ..Default::default()
                }
            } else {
                let bg = match status {
                    button::Status::Hovered => Some(clinical.background_secondary.into()),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    text_color: palette.background.base.text,
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
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            let clinical = theme.clinical();
            container::Style {
                background: Some(palette.background.base.color.into()),
                border: Border {
                    color: clinical.border_default,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Creates a sidebar with default width.
#[allow(dead_code)]
pub fn sidebar_default<'a, M: Clone + 'a>(
    items: Vec<SidebarItem<M>>,
    active_index: Option<usize>,
) -> Element<'a, M> {
    sidebar(items, active_index, SIDEBAR_WIDTH)
}

/// Creates a sidebar with a header.
#[allow(dead_code)]
pub fn sidebar_with_header<'a, M: Clone + 'a>(
    header: Element<'a, M>,
    items: Vec<SidebarItem<M>>,
    active_index: Option<usize>,
    width: f32,
) -> Element<'a, M> {
    let mut item_column = column![].spacing(SPACING_XS);

    for (index, item) in items.into_iter().enumerate() {
        let is_active = active_index == Some(index);
        let label = item.label.clone();

        let item_button = button(
            container(text(label).size(14).style(move |theme: &Theme| {
                let clinical = theme.clinical();
                text::Style {
                    color: Some(if is_active {
                        clinical.accent_pressed
                    } else {
                        clinical.text_secondary
                    }),
                }
            }))
            .padding([SPACING_SM, 12.0])
            .width(Length::Fill),
        )
        .on_press(item.message)
        .width(Length::Fill)
        .style(move |theme: &Theme, status| {
            let palette = theme.extended_palette();
            let clinical = theme.clinical();

            if is_active {
                button::Style {
                    background: Some(clinical.accent_primary_light.into()),
                    text_color: clinical.accent_pressed,
                    border: Border {
                        color: palette.primary.base.color,
                        width: 0.0,
                        radius: BORDER_RADIUS_SM.into(),
                    },
                    ..Default::default()
                }
            } else {
                let bg = match status {
                    button::Status::Hovered => Some(clinical.background_secondary.into()),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    text_color: palette.background.base.text,
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
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        let clinical = theme.clinical();
        container::Style {
            background: Some(palette.background.base.color.into()),
            border: Border {
                color: clinical.border_default,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}
