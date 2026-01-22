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
use iced::{Alignment, Border, Color, Element, Length};

use crate::theme::{BORDER_RADIUS_SM, SPACING_SM, SPACING_XS, colors};

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
    text_primary_color: Color,
    text_muted_color: Color,
    selected_bg: Color,
    hover_bg: Color,
    selected_border: Color,
    default_border: Color,
}

impl<'a, M: Clone + 'a> SelectableRow<'a, M> {
    /// Create a new selectable row.
    pub fn new(primary: impl Into<String>, on_click: M) -> Self {
        let c = colors();
        Self {
            primary: primary.into(),
            secondary: None,
            leading: None,
            trailing: None,
            selected: false,
            on_click,
            text_primary_color: c.text_primary,
            text_muted_color: c.text_muted,
            selected_bg: c.accent_primary_light,
            hover_bg: c.background_secondary,
            selected_border: c.accent_primary,
            default_border: c.border_default,
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
        let text_primary = self.text_primary_color;
        let text_muted = self.text_muted_color;
        let selected_bg = self.selected_bg;
        let hover_bg = self.hover_bg;
        let selected_border = self.selected_border;
        let default_border = self.default_border;

        // Build content
        let mut content_row = row![].spacing(SPACING_SM).align_y(Alignment::Center);

        // Leading element
        if let Some(leading) = self.leading {
            content_row = content_row.push(leading);
        }

        // Text section
        let text_section: Element<'a, M> = if let Some(secondary) = self.secondary {
            column![
                text(self.primary).size(13).color(text_primary),
                text(secondary).size(11).color(text_muted),
            ]
            .spacing(2.0)
            .into()
        } else {
            text(self.primary).size(13).color(text_primary).into()
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
                    Some(selected_bg.into())
                } else {
                    match status {
                        iced::widget::button::Status::Hovered => Some(hover_bg.into()),
                        _ => None,
                    }
                };
                let border_color = if is_selected {
                    selected_border
                } else {
                    default_border
                };

                iced::widget::button::Style {
                    background: bg,
                    text_color: text_primary,
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
    // Theme colors
    success_color: Color,
    warning_color: Color,
    muted_color: Color,
    accent_color: Color,
    text_on_accent: Color,
    text_primary: Color,
    hover_bg: Color,
}

impl<M: Clone> DomainListItem<M> {
    /// Create a new domain list item.
    pub fn new(code: impl Into<String>, display_name: impl Into<String>, on_click: M) -> Self {
        let c = colors();
        Self {
            code: code.into(),
            display_name: display_name.into(),
            row_count: 0,
            is_complete: false,
            is_touched: false,
            on_click,
            success_color: c.status_success,
            warning_color: c.status_warning,
            muted_color: c.text_muted,
            accent_color: c.accent_primary,
            text_on_accent: c.text_on_accent,
            text_primary: c.text_primary,
            hover_bg: c.background_secondary,
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
        use iced_fonts::lucide;

        let success = self.success_color;
        let warning = self.warning_color;
        let muted = self.muted_color;
        let accent = self.accent_color;
        let text_on_accent = self.text_on_accent;
        let text_primary = self.text_primary;
        let hover_bg = self.hover_bg;

        // Status icon
        let status_icon: Element<'a, M> = if self.is_complete {
            lucide::circle_check().size(14).color(success).into()
        } else if self.is_touched {
            lucide::pencil().size(14).color(warning).into()
        } else {
            lucide::circle().size(14).color(muted).into()
        };

        // Domain badge
        let badge: Element<'a, M> = container(text(self.code).size(14).color(text_on_accent))
            .padding([4.0, 12.0])
            .style(move |_| container::Style {
                background: Some(accent.into()),
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
            text(self.display_name).size(14).color(text_primary),
            Space::new().width(Length::Fill),
            text(format!("{} rows", self.row_count))
                .size(12)
                .color(muted),
        ]
        .align_y(Alignment::Center)
        .padding([SPACING_SM, SPACING_SM]);

        button(content)
            .on_press(self.on_click)
            .width(Length::Fill)
            .style(move |_, status| {
                let bg = match status {
                    iced::widget::button::Status::Hovered => Some(hover_bg.into()),
                    _ => None,
                };
                iced::widget::button::Style {
                    background: bg,
                    text_color: text_primary,
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
