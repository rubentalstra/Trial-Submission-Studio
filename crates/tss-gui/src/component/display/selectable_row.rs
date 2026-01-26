//! Selectable row component for master lists.
//!
//! Clickable list items with hover/selection states, commonly used
//! in master-detail layouts.
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::SelectableRow;
//! use iced_fonts::lucide;
//!
//! SelectableRow::new("STUDYID", Message::Selected(0))
//!     .secondary("Study Identifier")
//!     .leading(lucide::check().size(12).color(colors().status_success))
//!     .trailing(core_badge(CoreDesignation::Required))
//!     .selected(idx == selected_idx)
//!     .view()
//! ```

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::theme::{BORDER_RADIUS_SM, ClinicalColors, SPACING_SM, SPACING_XS};

// =============================================================================
// SELECTABLE ROW
// =============================================================================

/// A selectable row for master lists with hover/selection states and accessibility support.
pub struct SelectableRow<'a, M> {
    primary: String,
    secondary: Option<String>,
    leading: Option<Element<'a, M>>,
    trailing: Option<Element<'a, M>>,
    selected: bool,
    on_click: M,
}

impl<'a, M: Clone + 'a> SelectableRow<'a, M> {
    /// Create a new selectable row.
    pub fn new(primary: impl Into<String>, on_click: M) -> Self {
        Self {
            primary: primary.into(),
            secondary: None,
            leading: None,
            trailing: None,
            selected: false,
            on_click,
        }
    }

    /// Add secondary text below primary.
    pub fn secondary(mut self, text: impl Into<String>) -> Self {
        self.secondary = Some(text.into());
        self
    }

    /// Add a leading element (icon, status indicator).
    pub fn leading(mut self, element: impl Into<Element<'a, M>>) -> Self {
        self.leading = Some(element.into());
        self
    }

    /// Add a trailing element (badge, indicator).
    pub fn trailing(mut self, element: impl Into<Element<'a, M>>) -> Self {
        self.trailing = Some(element.into());
        self
    }

    /// Set selection state.
    pub fn selected(mut self, is_selected: bool) -> Self {
        self.selected = is_selected;
        self
    }

    /// Build the element.
    pub fn view(self) -> Element<'a, M> {
        let Self {
            primary,
            secondary,
            leading,
            trailing,
            on_click,
            selected,
        } = self;

        // Build content
        let mut content_row = row![].spacing(SPACING_SM).align_y(Alignment::Center);

        // Leading element
        if let Some(lead) = leading {
            content_row = content_row.push(lead);
        }

        // Text section
        let text_section: Element<'a, M> = if let Some(sec) = secondary {
            column![
                text(primary).size(13).style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                }),
                text(sec).size(11).style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
            ]
            .spacing(2.0)
            .into()
        } else {
            text(primary)
                .size(13)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                })
                .into()
        };
        content_row = content_row.push(text_section);

        // Fill space before trailing
        content_row = content_row.push(Space::new().width(Length::Fill));

        // Trailing element
        if let Some(trail) = trailing {
            content_row = content_row.push(trail);
        }

        // Button wrapper with styling
        button(content_row.padding([SPACING_SM, SPACING_SM]))
            .on_press(on_click)
            .width(Length::Fill)
            .style(move |theme: &Theme, status| {
                let clinical = theme.clinical();
                let bg = if selected {
                    Some(clinical.accent_primary_light.into())
                } else {
                    match status {
                        iced::widget::button::Status::Hovered => {
                            Some(clinical.background_secondary.into())
                        }
                        _ => None,
                    }
                };
                let border_color = if selected {
                    theme.extended_palette().primary.base.color
                } else {
                    clinical.border_default
                };

                iced::widget::button::Style {
                    background: bg,
                    text_color: theme.extended_palette().background.base.text,
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        color: border_color,
                        width: if selected { 1.0 } else { 0.0 },
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}

// =============================================================================
// DOMAIN LIST ITEM
// =============================================================================

/// Specialized list item for domain overview with accessibility support.
///
/// Combines domain badge + status icon + name + row count.
pub struct DomainListItem<M> {
    code: String,
    display_name: String,
    row_count: usize,
    is_complete: bool,
    is_touched: bool,
    on_click: M,
}

impl<M: Clone> DomainListItem<M> {
    /// Create a new domain list item.
    pub fn new(code: impl Into<String>, display_name: impl Into<String>, on_click: M) -> Self {
        Self {
            code: code.into(),
            display_name: display_name.into(),
            row_count: 0,
            is_complete: false,
            is_touched: false,
            on_click,
        }
    }

    /// Set row count.
    pub fn row_count(mut self, count: usize) -> Self {
        self.row_count = count;
        self
    }

    /// Set completion status.
    pub fn complete(mut self, is_complete: bool) -> Self {
        self.is_complete = is_complete;
        self
    }

    /// Set touched status (has been edited).
    pub fn touched(mut self, is_touched: bool) -> Self {
        self.is_touched = is_touched;
        self
    }

    /// Build the element.
    pub fn view<'a>(self) -> Element<'a, M>
    where
        M: 'a,
    {
        let is_complete = self.is_complete;
        let is_touched = self.is_touched;
        let code = self.code;
        let display_name = self.display_name;
        let row_count = self.row_count;

        // Status icon
        let status_icon: Element<'a, M> = if is_complete {
            container(lucide::circle_check().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().success.base.color),
                ..Default::default()
            })
        } else if is_touched {
            container(lucide::pencil().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().warning.base.color),
                ..Default::default()
            })
        } else {
            container(lucide::circle().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_muted),
                ..Default::default()
            })
        }
        .into();

        // Domain badge
        let badge: Element<'a, M> =
            container(text(code).size(14).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_on_accent),
            }))
            .padding([4.0, 12.0])
            .style(|theme: &Theme| container::Style {
                background: Some(theme.extended_palette().primary.base.color.into()),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into();

        // Build row
        let content = row![
            status_icon,
            Space::new().width(SPACING_XS),
            badge,
            Space::new().width(SPACING_SM),
            text(display_name)
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                }),
            Space::new().width(Length::Fill),
            text(format!("{} rows", row_count))
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
        ]
        .align_y(Alignment::Center)
        .padding([SPACING_SM, SPACING_SM]);

        button(content)
            .on_press(self.on_click)
            .width(Length::Fill)
            .style(|theme: &Theme, status| {
                let clinical = theme.clinical();
                let bg = match status {
                    iced::widget::button::Status::Hovered => {
                        Some(clinical.background_secondary.into())
                    }
                    _ => None,
                };
                iced::widget::button::Style {
                    background: bg,
                    text_color: theme.extended_palette().background.base.text,
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}
