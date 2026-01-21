//! Section card and panel components.
//!
//! Containers for grouping related content with consistent styling.
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::{SectionCard, panel, status_panel};
//! use iced_fonts::lucide;
//!
//! // Section with title and icon
//! SectionCard::new("Variable Information", content)
//!     .icon(lucide::info().size(14).color(PRIMARY_500))
//!     .view()
//!
//! // Simple panel wrapper
//! panel(my_content)
//!
//! // Status panel with colored border
//! status_panel(content, SUCCESS, Some(SUCCESS_LIGHT))
//! ```

use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length};

use crate::theme::{BORDER_RADIUS_SM, SPACING_MD, SPACING_SM, colors};

// =============================================================================
// SECTION CARD
// =============================================================================

/// A titled section card with optional icon.
///
/// Use for grouping related content with a header.
pub struct SectionCard<'a, M> {
    title: String,
    icon: Option<Element<'a, M>>,
    content: Element<'a, M>,
    title_color: Color,
    bg_color: Color,
    border_color: Color,
}

impl<'a, M: 'a> SectionCard<'a, M> {
    /// Create a new section card with title and content.
    pub fn new(title: impl Into<String>, content: impl Into<Element<'a, M>>) -> Self {
        let c = colors();
        Self {
            title: title.into(),
            icon: None,
            content: content.into(),
            title_color: c.text_secondary,
            bg_color: c.background_secondary,
            border_color: c.border_default,
        }
    }

    /// Add an icon to the header.
    pub fn icon(mut self, icon: impl Into<Element<'a, M>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Build the element.
    pub fn view(self) -> Element<'a, M> {
        let title_color = self.title_color;
        let bg_color = self.bg_color;
        let border_color = self.border_color;

        let header: Element<'a, M> = if let Some(icon) = self.icon {
            row![
                icon,
                Space::new().width(SPACING_SM),
                text(self.title).size(14).color(title_color),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            text(self.title).size(14).color(title_color).into()
        };

        container(
            column![header, Space::new().height(SPACING_SM), self.content,].width(Length::Fill),
        )
        .padding(SPACING_MD)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                color: border_color,
                width: 1.0,
            },
            ..Default::default()
        })
        .into()
    }
}

// =============================================================================
// PANEL
// =============================================================================

/// Simple panel wrapper with consistent styling.
///
/// Gray background, rounded corners, padding.
pub fn panel<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    let bg_color = colors().background_secondary;

    container(content)
        .padding(SPACING_MD)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

// =============================================================================
// STATUS PANEL
// =============================================================================

/// Panel with colored border for status indication.
///
/// Use for displaying content with visual status feedback.
///
/// # Arguments
/// * `content` - The panel content
/// * `border_color` - Border color for status indication
/// * `background` - Optional background color (defaults to light gray)
pub fn status_panel<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    border_color: Color,
    background: Option<Color>,
) -> Element<'a, M> {
    let bg = background.unwrap_or_else(|| colors().background_secondary);

    container(content)
        .padding(SPACING_MD)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                color: border_color,
                width: 2.0,
            },
            ..Default::default()
        })
        .into()
}
