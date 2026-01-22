//! Detail header component.
//!
//! A header for detail panels with title, subtitle, and optional badge.
//! Used in mapping, normalization, SUPP, and export detail panels.

use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::theme::{ClinicalColors, SPACING_XS};

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
}

impl<'a, M: 'a> DetailHeader<'a, M> {
    /// Create a new detail header with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            badge: None,
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
        let title_text = self.title.clone();
        let title_element = text(title_text)
            .size(20)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            });

        let subtitle_el: Element<'a, M> = if let Some(sub) = self.subtitle {
            column![
                Space::new().height(SPACING_XS),
                text(sub).size(14).style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
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
                    text(badge_text)
                        .size(11)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_on_accent),
                        }),
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
            row![title_element, Space::new().width(Length::Fill), badge_el,]
                .align_y(Alignment::Center),
            subtitle_el,
        ]
        .into()
    }
}
