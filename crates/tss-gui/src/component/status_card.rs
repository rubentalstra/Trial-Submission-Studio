//! Status card component.
//!
//! A card for displaying status information with icon, title, description,
//! and colored border. Used extensively in mapping detail panels.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::theme::{BORDER_RADIUS_SM, SPACING_MD, SPACING_SM, SPACING_XS, button_primary, colors};

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
    background: Option<Color>,
    border_color: Option<Color>,
    title_color: Color,
    value_color: Color,
    description_color: Color,
}

impl<'a, M: Clone + 'a> StatusCard<'a, M> {
    /// Create a new status card with an icon.
    pub fn new(icon: impl Into<Element<'a, M>>) -> Self {
        let c = colors();
        Self {
            icon: icon.into(),
            title: None,
            value: None,
            description: None,
            action: None,
            background: Some(c.background_secondary),
            border_color: None,
            title_color: c.text_muted,
            value_color: c.text_primary,
            description_color: c.text_muted,
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

    /// Set the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Set the border color.
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Build the status card element.
    pub fn view(self) -> Element<'a, M> {
        let c = colors();
        let bg_secondary = c.background_secondary;

        let bg = self.background.unwrap_or(bg_secondary);
        let border = self.border_color;
        let title_color = self.title_color;
        let value_color = self.value_color;
        let description_color = self.description_color;

        // Build text content column
        let mut text_content = column![];

        if let Some(title) = self.title {
            text_content = text_content.push(text(title).size(12).color(title_color));
        }

        if let Some(value) = self.value {
            text_content = text_content.push(text(value).size(14).color(value_color));
        }

        if let Some(desc) = self.description {
            text_content = text_content.push(text(desc).size(11).color(description_color));
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
            .style(move |_theme: &Theme| container::Style {
                background: Some(bg.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    color: border.unwrap_or(Color::TRANSPARENT),
                    width: if border.is_some() { 1.0 } else { 0.0 },
                },
                ..Default::default()
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
    let c = colors();
    let success_color = c.status_success;
    let bg_color = c.status_success_light;

    StatusCard::new(
        iced_fonts::lucide::circle_check()
            .size(16)
            .color(success_color),
    )
    .title(title)
    .value(value)
    .description(description)
    .background(bg_color)
    .border_color(success_color)
    .view()
}

/// Convenience function for creating a warning/suggested status card.
pub fn status_card_warning<'a, M: Clone + 'a>(
    title: impl Into<String>,
    value: impl Into<String>,
    description: impl Into<String>,
    action_label: impl Into<String>,
    action_message: M,
) -> Element<'a, M> {
    let c = colors();
    let warning_color = c.status_warning;
    let bg_color = c.status_warning_light;

    StatusCard::new(
        iced_fonts::lucide::lightbulb()
            .size(16)
            .color(warning_color),
    )
    .title(title)
    .value(value)
    .description(description)
    .action(action_label, action_message)
    .background(bg_color)
    .border_color(warning_color)
    .view()
}

/// Convenience function for creating a neutral status card.
pub fn status_card_neutral<'a, M: Clone + 'a>(
    icon: impl Into<Element<'a, M>>,
    title: impl Into<String>,
    description: impl Into<String>,
) -> Element<'a, M> {
    let c = colors();
    let bg_color = c.background_secondary;

    StatusCard::new(icon)
        .value(title)
        .description(description)
        .background(bg_color)
        .view()
}

/// Convenience function for creating an unmapped status card.
pub fn status_card_unmapped<'a, M: Clone + 'a>() -> Element<'a, M> {
    let c = colors();
    let muted_color = c.text_muted;
    let bg_color = c.background_secondary;

    StatusCard::new(iced_fonts::lucide::circle().size(16).color(muted_color))
        .value("Not Mapped")
        .description("Select a source column below to map this variable")
        .background(bg_color)
        .view()
}
