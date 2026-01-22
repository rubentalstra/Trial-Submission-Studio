//! Metadata card and row components.
//!
//! Components for displaying key-value metadata in styled cards.
//! Used in mapping, normalization, and SUPP detail panels.

use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Theme};

use crate::theme::{BORDER_RADIUS_SM, ClinicalColors, SPACING_MD, SPACING_SM};

// =============================================================================
// METADATA ROW
// =============================================================================

/// A single key-value metadata row with fixed label width.
///
/// # Example
/// ```ignore
/// metadata_row("Type", "Character")
/// ```
pub fn metadata_row<'a, M: 'a>(
    label: impl Into<String>,
    value: impl Into<String>,
) -> Element<'a, M> {
    let label_str: String = label.into();
    let value_str: String = value.into();

    row![
        text(label_str)
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted)
            })
            .width(Length::Fixed(80.0)),
        text(value_str).size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text)
        }),
    ]
    .align_y(Alignment::Center)
    .into()
}

/// A metadata row with custom label width.
pub fn metadata_row_wide<'a, M: 'a>(
    label: impl Into<String>,
    value: impl Into<String>,
    label_width: f32,
) -> Element<'a, M> {
    let label_str: String = label.into();
    let value_str: String = value.into();

    row![
        text(label_str)
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted)
            })
            .width(Length::Fixed(label_width)),
        text(value_str).size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text)
        }),
    ]
    .align_y(Alignment::Center)
    .into()
}

// =============================================================================
// METADATA CARD
// =============================================================================

/// A card container for displaying multiple metadata rows.
///
/// # Example
/// ```ignore
/// MetadataCard::new()
///     .row("Type", "Character")
///     .row("Length", "200")
///     .row("Core", "Required")
///     .view()
/// ```
pub struct MetadataCard {
    rows: Vec<(String, String)>,
    title: Option<String>,
}

impl MetadataCard {
    /// Create a new metadata card.
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            title: None,
        }
    }

    /// Set an optional title for the card.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add a key-value row to the card.
    pub fn row(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.rows.push((label.into(), value.into()));
        self
    }

    /// Add a row only if the value is Some.
    pub fn row_opt(mut self, label: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        if let Some(v) = value {
            self.rows.push((label.into(), v.into()));
        }
        self
    }

    /// Build the metadata card element.
    pub fn view<'a, M: 'a>(self) -> Element<'a, M> {
        let mut content = column![].spacing(SPACING_SM);

        if let Some(title) = self.title {
            content = content.push(text(title).size(14).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }));
            content = content.push(Space::new().height(SPACING_SM));
        }

        for (label, value) in self.rows {
            let row_el: Element<'a, M> = row![
                text(label)
                    .size(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted)
                    })
                    .width(Length::Fixed(80.0)),
                text(value).size(12).style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().background.base.text)
                }),
            ]
            .align_y(Alignment::Center)
            .into();
            content = content.push(row_el);
        }

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
}

impl Default for MetadataCard {
    fn default() -> Self {
        Self::new()
    }
}
