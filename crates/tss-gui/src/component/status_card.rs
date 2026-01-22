//! Status card component.
//!
//! A card for displaying status information with icon, title, description,
//! and colored border. Used extensively in mapping detail panels.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_MD, SPACING_SM, SPACING_XS, button_primary,
};

// =============================================================================
// COLOR ENUM
// =============================================================================

/// Color specification - either a direct color or a theme-derived closure.
enum ColorSpec {
    Direct(Color),
    Themed(fn(&Theme) -> Color),
}

impl ColorSpec {
    fn resolve(&self, theme: &Theme) -> Color {
        match self {
            ColorSpec::Direct(c) => *c,
            ColorSpec::Themed(f) => f(theme),
        }
    }
}

// =============================================================================
// STATUS CARD
// =============================================================================

/// A status display card with icon, title, description, and colored border.
///
/// Uses semantic colors that adapt to the current accessibility mode.
///
/// # Example
/// ```ignore
/// StatusCard::new(lucide::circle_check().size(16).color(success_color))
///     .title("Mapped to:")
///     .value("AGE_SOURCE")
///     .description("95% confidence")
///     .background(success_bg)
///     .border_color(success_color)
///     .view()
/// ```
pub struct StatusCard<'a, M> {
    icon: Element<'a, M>,
    title: Option<String>,
    value: Option<String>,
    description: Option<String>,
    action: Option<(String, M)>,
    background: Option<ColorSpec>,
    border_color: Option<ColorSpec>,
}

impl<'a, M: Clone + 'a> StatusCard<'a, M> {
    /// Create a new status card with an icon.
    pub fn new(icon: impl Into<Element<'a, M>>) -> Self {
        Self {
            icon: icon.into(),
            title: None,
            value: None,
            description: None,
            action: None,
            background: None,
            border_color: None,
        }
    }

    /// Set the title text (small, secondary color).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the main value text (larger, primary color).
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set the description text (small, tertiary color).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add an action button inside the card.
    pub fn action(mut self, label: impl Into<String>, message: M) -> Self {
        self.action = Some((label.into(), message));
        self
    }

    /// Set the background color directly.
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(ColorSpec::Direct(color));
        self
    }

    /// Set the background color from a theme closure.
    pub fn background_themed(mut self, color_fn: fn(&Theme) -> Color) -> Self {
        self.background = Some(ColorSpec::Themed(color_fn));
        self
    }

    /// Set the border color directly.
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(ColorSpec::Direct(color));
        self
    }

    /// Set the border color from a theme closure.
    pub fn border_color_themed(mut self, color_fn: fn(&Theme) -> Color) -> Self {
        self.border_color = Some(ColorSpec::Themed(color_fn));
        self
    }

    /// Build the status card element.
    pub fn view(self) -> Element<'a, M> {
        let bg_spec = self.background;
        let border_spec = self.border_color;

        // Build text content column
        let mut text_content = column![];

        if let Some(title) = self.title {
            text_content = text_content.push(text(title).size(12).style(|theme: &Theme| {
                let clinical = theme.clinical();
                text::Style {
                    color: Some(clinical.text_muted),
                }
            }));
        }

        if let Some(value) = self.value {
            text_content = text_content.push(text(value).size(14).style(|theme: &Theme| {
                let palette = theme.extended_palette();
                text::Style {
                    color: Some(palette.background.base.text),
                }
            }));
        }

        if let Some(desc) = self.description {
            text_content = text_content.push(text(desc).size(11).style(|theme: &Theme| {
                let clinical = theme.clinical();
                text::Style {
                    color: Some(clinical.text_muted),
                }
            }));
        }

        // Main content row with icon and text
        let main_content = row![self.icon, Space::new().width(SPACING_SM), text_content,]
            .align_y(Alignment::Center);

        // Build final content with optional action button
        let final_content: Element<'a, M> = if let Some((label, message)) = self.action {
            let action_btn = button(
                row![
                    iced_fonts::lucide::check().size(12),
                    Space::new().width(SPACING_XS),
                    text(label).size(13),
                ]
                .align_y(Alignment::Center),
            )
            .on_press(message)
            .padding([8.0, 16.0])
            .style(button_primary);

            column![main_content, Space::new().height(SPACING_SM), action_btn,].into()
        } else {
            main_content.into()
        };

        container(final_content)
            .padding(SPACING_MD)
            .width(Length::Fill)
            .style(move |theme: &Theme| {
                let clinical = theme.clinical();
                let bg_color = bg_spec
                    .as_ref()
                    .map(|s| s.resolve(theme))
                    .unwrap_or(clinical.background_secondary);
                let border_col = border_spec.as_ref().map(|s| s.resolve(theme));
                container::Style {
                    background: Some(bg_color.into()),
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        color: border_col.unwrap_or(Color::TRANSPARENT),
                        width: if border_col.is_some() { 1.0 } else { 0.0 },
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}

// =============================================================================
// PRESET STATUS CARDS
// =============================================================================

/// Convenience function for creating a success status card.
pub fn status_card_success<'a, M: Clone + 'a>(
    title: impl Into<String>,
    value: impl Into<String>,
    description: impl Into<String>,
) -> Element<'a, M> {
    let title_str = title.into();
    let value_str = value.into();
    let desc_str = description.into();

    // We need to create the icon with a style closure
    let icon = container(iced_fonts::lucide::circle_check().size(16)).style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            text_color: Some(palette.success.base.color),
            ..Default::default()
        }
    });

    // Create a custom card that applies colors via theme
    container(
        row![
            icon,
            Space::new().width(SPACING_SM),
            column![
                text(title_str).size(12).style(|theme: &Theme| {
                    let clinical = theme.clinical();
                    text::Style {
                        color: Some(clinical.text_muted),
                    }
                }),
                text(value_str).size(14).style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(palette.background.base.text),
                    }
                }),
                text(desc_str).size(11).style(|theme: &Theme| {
                    let clinical = theme.clinical();
                    text::Style {
                        color: Some(clinical.text_muted),
                    }
                }),
            ],
        ]
        .align_y(Alignment::Center),
    )
    .padding(SPACING_MD)
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        let clinical = theme.clinical();
        container::Style {
            background: Some(clinical.status_success_light.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                color: palette.success.base.color,
                width: 1.0,
            },
            ..Default::default()
        }
    })
    .into()
}

/// Convenience function for creating a warning/suggested status card.
pub fn status_card_warning<'a, M: Clone + 'a>(
    title: impl Into<String>,
    value: impl Into<String>,
    description: impl Into<String>,
    action_label: impl Into<String>,
    action_message: M,
) -> Element<'a, M> {
    let title_str = title.into();
    let value_str = value.into();
    let desc_str = description.into();
    let action_str = action_label.into();

    let icon = container(iced_fonts::lucide::lightbulb().size(16)).style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            text_color: Some(palette.warning.base.color),
            ..Default::default()
        }
    });

    let action_btn = button(
        row![
            iced_fonts::lucide::check().size(12),
            Space::new().width(SPACING_XS),
            text(action_str).size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(action_message)
    .padding([8.0, 16.0])
    .style(button_primary);

    container(column![
        row![
            icon,
            Space::new().width(SPACING_SM),
            column![
                text(title_str).size(12).style(|theme: &Theme| {
                    let clinical = theme.clinical();
                    text::Style {
                        color: Some(clinical.text_muted),
                    }
                }),
                text(value_str).size(14).style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(palette.background.base.text),
                    }
                }),
                text(desc_str).size(11).style(|theme: &Theme| {
                    let clinical = theme.clinical();
                    text::Style {
                        color: Some(clinical.text_muted),
                    }
                }),
            ],
        ]
        .align_y(Alignment::Center),
        Space::new().height(SPACING_SM),
        action_btn,
    ])
    .padding(SPACING_MD)
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        let clinical = theme.clinical();
        container::Style {
            background: Some(clinical.status_warning_light.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                color: palette.warning.base.color,
                width: 1.0,
            },
            ..Default::default()
        }
    })
    .into()
}

/// Convenience function for creating a neutral status card.
pub fn status_card_neutral<'a, M: Clone + 'a>(
    icon: impl Into<Element<'a, M>>,
    title: impl Into<String>,
    description: impl Into<String>,
) -> Element<'a, M> {
    let title_str = title.into();
    let desc_str = description.into();

    container(
        row![
            icon.into(),
            Space::new().width(SPACING_SM),
            column![
                text(title_str).size(14).style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(palette.background.base.text),
                    }
                }),
                text(desc_str).size(11).style(|theme: &Theme| {
                    let clinical = theme.clinical();
                    text::Style {
                        color: Some(clinical.text_muted),
                    }
                }),
            ],
        ]
        .align_y(Alignment::Center),
    )
    .padding(SPACING_MD)
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let clinical = theme.clinical();
        container::Style {
            background: Some(clinical.background_secondary.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}

/// Convenience function for creating an unmapped status card.
pub fn status_card_unmapped<'a, M: Clone + 'a>() -> Element<'a, M> {
    let icon = container(iced_fonts::lucide::circle().size(16)).style(|theme: &Theme| {
        let clinical = theme.clinical();
        container::Style {
            text_color: Some(clinical.text_muted),
            ..Default::default()
        }
    });

    container(
        row![
            icon,
            Space::new().width(SPACING_SM),
            column![
                text("Not Mapped").size(14).style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(palette.background.base.text),
                    }
                }),
                text("Select a source column below to map this variable")
                    .size(11)
                    .style(|theme: &Theme| {
                        let clinical = theme.clinical();
                        text::Style {
                            color: Some(clinical.text_muted),
                        }
                    }),
            ],
        ]
        .align_y(Alignment::Center),
    )
    .padding(SPACING_MD)
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let clinical = theme.clinical();
        container::Style {
            background: Some(clinical.background_secondary.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}
