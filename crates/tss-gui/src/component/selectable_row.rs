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
//!     .leading(lucide::check().size(12).color(SUCCESS))
//!     .trailing(core_badge(CoreDesignation::Required))
//!     .selected(idx == selected_idx)
//!     .view()
//! ```

use iced::widget::{Space, button, column, row, text};
use iced::{Alignment, Border, Element, Length};

use crate::theme::{
    BORDER_RADIUS_SM, GRAY_100, GRAY_200, GRAY_500, GRAY_800, GRAY_900, PRIMARY_100, PRIMARY_500,
    SPACING_SM, SPACING_XS,
};

// =============================================================================
// SELECTABLE ROW
// =============================================================================

/// A selectable row for master lists with hover/selection states.
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
        let is_selected = self.selected;

        // Build content
        let mut content_row = row![].spacing(SPACING_SM).align_y(Alignment::Center);

        // Leading element
        if let Some(leading) = self.leading {
            content_row = content_row.push(leading);
        }

        // Text section
        let text_section: Element<'a, M> = if let Some(secondary) = self.secondary {
            column![
                text(self.primary).size(13).color(GRAY_900),
                text(secondary).size(11).color(GRAY_500),
            ]
            .spacing(2.0)
            .into()
        } else {
            text(self.primary).size(13).color(GRAY_900).into()
        };
        content_row = content_row.push(text_section);

        // Fill space before trailing
        content_row = content_row.push(Space::new().width(Length::Fill));

        // Trailing element
        if let Some(trailing) = self.trailing {
            content_row = content_row.push(trailing);
        }

        // Button wrapper with styling
        button(content_row.padding([SPACING_SM, SPACING_SM]))
            .on_press(self.on_click)
            .width(Length::Fill)
            .style(move |_, status| {
                let bg = if is_selected {
                    Some(PRIMARY_100.into())
                } else {
                    match status {
                        iced::widget::button::Status::Hovered => Some(GRAY_100.into()),
                        _ => None,
                    }
                };
                let border_color = if is_selected { PRIMARY_500 } else { GRAY_200 };

                iced::widget::button::Style {
                    background: bg,
                    text_color: GRAY_800,
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        color: border_color,
                        width: if is_selected { 1.0 } else { 0.0 },
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

/// Specialized list item for domain overview.
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
        use crate::theme::{PRIMARY_500, SUCCESS, WARNING, WHITE};
        use iced::widget::container;
        use iced_fonts::lucide;

        // Status icon
        let status_icon: Element<'a, M> = if self.is_complete {
            lucide::circle_check().size(14).color(SUCCESS).into()
        } else if self.is_touched {
            lucide::pencil().size(14).color(WARNING).into()
        } else {
            lucide::circle().size(14).color(GRAY_500).into()
        };

        // Domain badge (inline to avoid lifetime issues with owned string)
        let badge: Element<'a, M> = container(text(self.code).size(14).color(WHITE))
            .padding([4.0, 12.0])
            .style(|_| container::Style {
                background: Some(PRIMARY_500.into()),
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
            text(self.display_name).size(14).color(GRAY_800),
            Space::new().width(Length::Fill),
            text(format!("{} rows", self.row_count))
                .size(12)
                .color(GRAY_500),
        ]
        .align_y(Alignment::Center)
        .padding([SPACING_SM, SPACING_SM]);

        button(content)
            .on_press(self.on_click)
            .width(Length::Fill)
            .style(|_, status| {
                let bg = match status {
                    iced::widget::button::Status::Hovered => Some(GRAY_100.into()),
                    _ => None,
                };
                iced::widget::button::Style {
                    background: bg,
                    text_color: GRAY_800,
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
