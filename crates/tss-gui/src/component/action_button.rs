//! Action button components.
//!
//! Buttons and button lists for action sections in detail panels.
//! Provides consistent styling for primary and secondary actions.

use iced::widget::{Space, button, column, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::theme::{SPACING_SM, SPACING_XS, button_primary, button_secondary, colors};

// =============================================================================
// ACTION BUTTON STYLE
// =============================================================================

/// Button style variants.
#[derive(Clone)]
pub enum ActionButtonStyle {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

// =============================================================================
// ACTION BUTTON
// =============================================================================

/// A styled action button with icon and label.
///
/// # Example
/// ```ignore
/// ActionButton::secondary(lucide::x(), "Clear Mapping", Message::ClearMapping)
///     .view()
/// ```
pub struct ActionButton<'a, M> {
    icon: Option<Element<'a, M>>,
    label: String,
    on_press: M,
    style: ActionButtonStyle,
    full_width: bool,
    // Theme colors for different styles
    error_color: Color,
    error_hover: Color,
    error_pressed: Color,
    ghost_color: Color,
    ghost_hover: Color,
    text_on_accent: Color,
}

impl<'a, M: Clone + 'a> ActionButton<'a, M> {
    fn new_with_style(
        icon: Option<Element<'a, M>>,
        label: impl Into<String>,
        on_press: M,
        style: ActionButtonStyle,
    ) -> Self {
        let c = colors();
        Self {
            icon,
            label: label.into(),
            on_press,
            style,
            full_width: false,
            error_color: c.status_error,
            error_hover: c.danger_hover,
            error_pressed: c.danger_pressed,
            ghost_color: c.text_muted,
            ghost_hover: c.text_secondary,
            text_on_accent: c.text_on_accent,
        }
    }

    /// Create a primary action button.
    pub fn primary(icon: impl Into<Element<'a, M>>, label: impl Into<String>, on_press: M) -> Self {
        Self::new_with_style(
            Some(icon.into()),
            label,
            on_press,
            ActionButtonStyle::Primary,
        )
    }

    /// Create a secondary action button.
    pub fn secondary(
        icon: impl Into<Element<'a, M>>,
        label: impl Into<String>,
        on_press: M,
    ) -> Self {
        Self::new_with_style(
            Some(icon.into()),
            label,
            on_press,
            ActionButtonStyle::Secondary,
        )
    }

    /// Create a danger action button.
    pub fn danger(icon: impl Into<Element<'a, M>>, label: impl Into<String>, on_press: M) -> Self {
        Self::new_with_style(
            Some(icon.into()),
            label,
            on_press,
            ActionButtonStyle::Danger,
        )
    }

    /// Create a ghost action button (text only, minimal styling).
    pub fn ghost(label: impl Into<String>, on_press: M) -> Self {
        Self::new_with_style(None, label, on_press, ActionButtonStyle::Ghost)
    }

    /// Create a primary button without icon.
    pub fn primary_text(label: impl Into<String>, on_press: M) -> Self {
        Self::new_with_style(None, label, on_press, ActionButtonStyle::Primary)
    }

    /// Create a secondary button without icon.
    pub fn secondary_text(label: impl Into<String>, on_press: M) -> Self {
        Self::new_with_style(None, label, on_press, ActionButtonStyle::Secondary)
    }

    /// Make the button full width.
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    /// Build the action button element.
    pub fn view(self) -> Element<'a, M> {
        let label = self.label.clone();
        let label2 = self.label;
        let content: Element<'a, M> = if let Some(icon) = self.icon {
            row![icon, Space::new().width(SPACING_XS), text(label).size(13),]
                .align_y(Alignment::Center)
                .into()
        } else {
            text(label2).size(13).into()
        };

        let style = self.style.clone();
        let error_color = self.error_color;
        let error_hover = self.error_hover;
        let error_pressed = self.error_pressed;
        let ghost_color = self.ghost_color;
        let ghost_hover = self.ghost_hover;
        let text_on_accent = self.text_on_accent;

        let mut btn = button(content).on_press(self.on_press).padding([8.0, 16.0]);

        if self.full_width {
            btn = btn.width(Length::Fill);
        }

        btn = btn.style(move |theme: &Theme, status| match style {
            ActionButtonStyle::Primary => button_primary(theme, status),
            ActionButtonStyle::Secondary => button_secondary(theme, status),
            ActionButtonStyle::Danger => {
                let bg = match status {
                    iced::widget::button::Status::Hovered => error_hover,
                    iced::widget::button::Status::Pressed => error_pressed,
                    _ => error_color,
                };
                iced::widget::button::Style {
                    background: Some(bg.into()),
                    text_color: text_on_accent,
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            ActionButtonStyle::Ghost => {
                let text_color = match status {
                    iced::widget::button::Status::Hovered => ghost_hover,
                    _ => ghost_color,
                };
                iced::widget::button::Style {
                    background: None,
                    text_color,
                    ..Default::default()
                }
            }
        });

        btn.into()
    }
}

// =============================================================================
// ACTION BUTTON LIST
// =============================================================================

/// A vertical list of action buttons with consistent spacing.
///
/// # Example
/// ```ignore
/// ActionButtonList::new()
///     .button(ActionButton::secondary(lucide::x(), "Clear Mapping", msg1))
///     .button(ActionButton::secondary(lucide::ban(), "Mark Not Collected", msg2))
///     .view()
/// ```
pub struct ActionButtonList<'a, M> {
    title: Option<String>,
    buttons: Vec<Element<'a, M>>,
    title_color: Color,
}

impl<'a, M: Clone + 'a> ActionButtonList<'a, M> {
    /// Create a new empty action button list.
    pub fn new() -> Self {
        let c = colors();
        Self {
            title: None,
            buttons: Vec::new(),
            title_color: c.text_secondary,
        }
    }

    /// Set an optional title for the action section.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add an action button to the list.
    pub fn button(mut self, button: ActionButton<'a, M>) -> Self {
        self.buttons.push(button.view());
        self
    }

    /// Add a pre-built element to the list.
    pub fn element(mut self, element: impl Into<Element<'a, M>>) -> Self {
        self.buttons.push(element.into());
        self
    }

    /// Build the action button list element.
    /// Returns None if the list is empty.
    pub fn view(self) -> Option<Element<'a, M>> {
        if self.buttons.is_empty() {
            return None;
        }

        let mut content = column![].spacing(SPACING_SM);

        if let Some(title) = self.title {
            content = content.push(text(title).size(14).color(self.title_color));
            content = content.push(Space::new().height(SPACING_SM));
        }

        for btn in self.buttons {
            content = content.push(btn);
        }

        Some(content.into())
    }

    /// Build the action button list, returning an empty element if no buttons.
    pub fn view_or_empty(self) -> Element<'a, M> {
        self.view()
            .unwrap_or_else(|| Space::new().height(0.0).into())
    }
}

impl<'a, M: Clone + 'a> Default for ActionButtonList<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// INLINE EDIT BUTTONS
// =============================================================================

/// Save and cancel button pair for inline editing.
///
/// # Example
/// ```ignore
/// edit_buttons(can_save, Message::Save, Message::Cancel)
/// ```
pub fn edit_buttons<'a, M: Clone + 'a>(
    can_save: bool,
    save_message: M,
    cancel_message: M,
) -> Element<'a, M> {
    let save_btn = button(
        row![
            iced_fonts::lucide::check().size(12),
            Space::new().width(SPACING_XS),
            text("Save").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press_maybe(if can_save { Some(save_message) } else { None })
    .padding([8.0, 16.0])
    .style(button_primary);

    let cancel_btn = button(text("Cancel").size(13))
        .on_press(cancel_message)
        .padding([8.0, 16.0])
        .style(button_secondary);

    row![save_btn, Space::new().width(SPACING_SM), cancel_btn,].into()
}
