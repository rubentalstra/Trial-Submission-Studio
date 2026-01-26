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
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::theme::{BORDER_RADIUS_SM, ClinicalColors, SPACING_MD, SPACING_SM};

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
}

impl<'a, M: 'a> SectionCard<'a, M> {
    /// Create a new section card with title and content.
    pub fn new(title: impl Into<String>, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            content: content.into(),
        }
    }

    /// Add an icon to the header.
    pub fn icon(mut self, icon: impl Into<Element<'a, M>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Build the element.
    pub fn view(self) -> Element<'a, M> {
        let Self {
            title,
            icon,
            content,
        } = self;

        let header: Element<'a, M> = if let Some(ic) = icon {
            row![
                ic,
                Space::new().width(SPACING_SM),
                text(title).size(14).style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_secondary),
                }),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            text(title)
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_secondary),
                })
                .into()
        };

        container(column![header, Space::new().height(SPACING_SM), content,].width(Length::Fill))
            .padding(SPACING_MD)
            .width(Length::Fill)
            .style(|theme: &Theme| {
                let clinical = theme.clinical();
                container::Style {
                    background: Some(clinical.background_secondary.into()),
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        color: clinical.border_default,
                        width: 1.0,
                    },
                    ..Default::default()
                }
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
    container(content)
        .padding(SPACING_MD)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
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
    container(content)
        .padding(SPACING_MD)
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            let bg = background.unwrap_or_else(|| theme.clinical().background_secondary);
            container::Style {
                background: Some(bg.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    color: border_color,
                    width: 2.0,
                },
                ..Default::default()
            }
        })
        .into()
}
