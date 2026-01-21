//! Detail header component.
//!
//! A header for detail panels with title, subtitle, and optional badge.
//! Used in mapping, normalization, SUPP, and export detail panels.

use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::theme::{SPACING_XS, colors};

// =============================================================================
// DETAIL HEADER
// =============================================================================

/// Detail panel header with title, subtitle, and optional badge.
///
/// # Example
/// ```ignore
/// DetailHeader::new("STUDYID")
///     .subtitle("Study Identifier")
///     .badge("Constant Value", GRAY_600)
///     .view()
/// ```
pub struct DetailHeader<'a, M> {
    title: String,
    subtitle: Option<String>,
    badge: Option<(Element<'a, M>, String, Color)>, // (icon, text, bg_color)
    title_color: Color,
    subtitle_color: Color,
    badge_text_color: Color,
}

impl<'a, M: 'a> DetailHeader<'a, M> {
    /// Create a new detail header with a title.
    pub fn new(title: impl Into<String>) -> Self {
        let c = colors();
        Self {
            title: title.into(),
            subtitle: None,
            badge: None,
            title_color: c.text_primary,
            subtitle_color: c.text_muted,
            badge_text_color: c.text_on_accent,
        }
    }

    /// Set the subtitle text.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set a badge with icon, text, and background color.
    pub fn badge(
        mut self,
        icon: impl Into<Element<'a, M>>,
        text: impl Into<String>,
        color: Color,
    ) -> Self {
        self.badge = Some((icon.into(), text.into(), color));
        self
    }

    /// Set a simple badge with just text and color (no icon).
    pub fn badge_simple(mut self, text: impl Into<String>, color: Color) -> Self {
        self.badge = Some((Space::new().width(0.0).into(), text.into(), color));
        self
    }

    /// Build the detail header element.
    pub fn view(self) -> Element<'a, M> {
        let title_color = self.title_color;
        let subtitle_color = self.subtitle_color;
        let badge_text_color = self.badge_text_color;

        let title_text = text(self.title).size(20).color(title_color);

        let subtitle_el: Element<'a, M> = if let Some(sub) = self.subtitle {
            column![
                Space::new().height(SPACING_XS),
                text(sub).size(14).color(subtitle_color),
            ]
            .into()
        } else {
            Space::new().height(0.0).into()
        };

        let badge_el: Element<'a, M> = if let Some((icon, badge_text, bg_color)) = self.badge {
            container(
                row![
                    icon,
                    Space::new().width(SPACING_XS),
                    text(badge_text).size(11).color(badge_text_color),
                ]
                .align_y(Alignment::Center),
            )
            .padding([4.0, 10.0])
            .style(move |_theme: &Theme| container::Style {
                background: Some(bg_color.into()),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
        } else {
            Space::new().width(0.0).into()
        };

        column![
            row![title_text, Space::new().width(Length::Fill), badge_el,]
                .align_y(Alignment::Center),
            subtitle_el,
        ]
        .into()
    }
}
