//! Page header component.
//!
//! Consistent headers for views with back button, badge, title, and metadata.
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::PageHeader;
//!
//! PageHeader::new("Demographics")
//!     .back(Message::BackClicked)
//!     .badge("DM", PRIMARY_500)
//!     .meta("Rows", "150")
//!     .meta("Progress", "85%")
//!     .view()
//! ```

use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Border, Color, Element, Length};
use iced_fonts::lucide;

use crate::theme::{
    GRAY_100, GRAY_200, GRAY_500, GRAY_900, SPACING_LG, SPACING_MD, SPACING_SM, WHITE,
    button_secondary,
};

// =============================================================================
// PAGE HEADER
// =============================================================================

/// Page header with back button, badge, title, and metadata.
pub struct PageHeader<'a, M> {
    title: String,
    on_back: Option<M>,
    badge: Option<(String, Color)>,
    metadata: Vec<(String, String)>,
    trailing: Option<Element<'a, M>>,
}

impl<'a, M: Clone + 'a> PageHeader<'a, M> {
    /// Create a new page header with title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            on_back: None,
            badge: None,
            metadata: Vec::new(),
            trailing: None,
        }
    }

    /// Add a back button.
    pub fn back(mut self, message: M) -> Self {
        self.on_back = Some(message);
        self
    }

    /// Add a colored badge next to the title.
    pub fn badge(mut self, text: impl Into<String>, color: Color) -> Self {
        self.badge = Some((text.into(), color));
        self
    }

    /// Add a metadata key-value pair.
    pub fn meta(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((label.into(), value.into()));
        self
    }

    /// Add trailing element(s) on the right.
    pub fn trailing(mut self, element: impl Into<Element<'a, M>>) -> Self {
        self.trailing = Some(element.into());
        self
    }

    /// Build the element.
    pub fn view(self) -> Element<'a, M> {
        let mut header_row = row![].spacing(SPACING_SM).align_y(Alignment::Center);

        // Back button
        if let Some(on_back) = self.on_back {
            let back_btn = button(
                row![lucide::chevron_left().size(12), text("Back").size(14),]
                    .spacing(SPACING_SM)
                    .align_y(Alignment::Center),
            )
            .on_press(on_back)
            .padding([8.0, 16.0])
            .style(button_secondary);

            header_row = header_row.push(back_btn);
            header_row = header_row.push(Space::new().width(SPACING_MD));
        }

        // Badge
        if let Some((badge_text, badge_color)) = self.badge {
            let badge = container(text(badge_text).size(14).color(WHITE))
                .padding([4.0, 12.0])
                .style(move |_| container::Style {
                    background: Some(badge_color.into()),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            header_row = header_row.push(badge);
            header_row = header_row.push(Space::new().width(SPACING_SM));
        }

        // Title
        header_row = header_row.push(text(self.title).size(20).color(GRAY_900));

        // Fill space
        header_row = header_row.push(Space::new().width(Length::Fill));

        // Metadata items
        for (label, value) in self.metadata {
            let meta_item = text(format!("{}: {}", label, value))
                .size(12)
                .color(GRAY_500);
            header_row = header_row.push(meta_item);
            header_row = header_row.push(Space::new().width(SPACING_MD));
        }

        // Trailing element
        if let Some(trailing) = self.trailing {
            header_row = header_row.push(trailing);
        }

        // Container with background
        container(header_row)
            .width(Length::Fill)
            .padding([SPACING_MD, SPACING_LG])
            .style(|_| container::Style {
                background: Some(GRAY_100.into()),
                border: Border {
                    width: 0.0,
                    radius: 0.0.into(),
                    color: GRAY_200,
                },
                ..Default::default()
            })
            .into()
    }
}

// =============================================================================
// SIMPLE HEADER
// =============================================================================

/// Simple page header with just title and back button.
///
/// Convenience function for minimal headers.
pub fn page_header_simple<'a, M: Clone + 'a>(
    title: impl Into<String>,
    on_back: M,
) -> Element<'a, M> {
    PageHeader::new(title).back(on_back).view()
}
