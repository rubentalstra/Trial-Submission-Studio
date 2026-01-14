//! Text field components with validation.
//!
//! Form text fields with character counting, validation errors,
//! and consistent styling.

use iced::widget::{Space, column, row, text, text_input};
use iced::{Border, Element, Length, Theme};

use crate::theme::{
    BORDER_RADIUS_SM, ERROR, GRAY_200, GRAY_400, GRAY_500, GRAY_600, GRAY_900, PRIMARY_100, WHITE,
};

// =============================================================================
// TEXT FIELD
// =============================================================================

/// A text input field with label, character count, and validation.
///
/// # Example
/// ```ignore
/// TextField::new("QNAM", &value, "Qualifier name", |s| Message::QnamChanged(s))
///     .max_length(8)
///     .required(true)
///     .error(if value.is_empty() { Some("Required") } else { None })
///     .view()
/// ```
pub struct TextField<M> {
    label: String,
    value: String,
    placeholder: String,
    on_change: Box<dyn Fn(String) -> M>,
    max_length: Option<usize>,
    required: bool,
    error: Option<String>,
}

impl<M: Clone + 'static> TextField<M> {
    /// Create a new text field.
    pub fn new(
        label: impl Into<String>,
        value: &str,
        placeholder: impl Into<String>,
        on_change: impl Fn(String) -> M + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            value: value.to_string(),
            placeholder: placeholder.into(),
            on_change: Box::new(on_change),
            max_length: None,
            required: false,
            error: None,
        }
    }

    /// Set maximum character length.
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Mark field as required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set an error message to display.
    pub fn error(mut self, error: Option<impl Into<String>>) -> Self {
        self.error = error.map(Into::into);
        self
    }

    /// Build the text field element.
    pub fn view(self) -> Element<'static, M> {
        let char_count = self.value.len();
        let is_over = self.max_length.is_some_and(|max| char_count > max);
        let has_error = self.error.is_some() || is_over;

        // Label with optional required indicator
        let label_text = if self.required {
            format!("{} *", self.label)
        } else {
            self.label.clone()
        };

        // Character count display
        let count_display: Element<'static, M> = if let Some(max) = self.max_length {
            text(format!("{}/{}", char_count, max))
                .size(11)
                .color(if is_over { ERROR } else { GRAY_400 })
                .into()
        } else {
            Space::new().width(0.0).into()
        };

        // Error message
        let error_el: Element<'static, M> = if let Some(err) = self.error {
            row![
                iced_fonts::lucide::circle_alert().size(12).color(ERROR),
                Space::new().width(4.0),
                text(err).size(11).color(ERROR),
            ]
            .into()
        } else if is_over {
            text("Character limit exceeded")
                .size(11)
                .color(ERROR)
                .into()
        } else {
            Space::new().height(0.0).into()
        };

        let value = self.value;
        let placeholder = self.placeholder;
        let on_change = self.on_change;

        column![
            row![
                text(label_text).size(12).color(GRAY_600),
                Space::new().width(Length::Fill),
                count_display,
            ],
            Space::new().height(4.0),
            text_input(&placeholder, &value)
                .on_input(move |s| on_change(s))
                .padding([10.0, 12.0])
                .size(14)
                .style(move |_: &Theme, _status| {
                    let border_color = if has_error { ERROR } else { GRAY_200 };
                    iced::widget::text_input::Style {
                        background: WHITE.into(),
                        border: Border {
                            color: border_color,
                            width: 1.0,
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        icon: GRAY_500,
                        placeholder: GRAY_400,
                        value: GRAY_900,
                        selection: PRIMARY_100,
                    }
                }),
            error_el,
        ]
        .into()
    }
}

// =============================================================================
// TEXTAREA FIELD (for longer text)
// =============================================================================

/// A multi-line text area field.
///
/// Note: Iced doesn't have a native textarea, so this uses text_input
/// with a larger size hint. For true multi-line editing, consider
/// using a custom widget.
pub struct TextAreaField<M> {
    label: String,
    value: String,
    placeholder: String,
    on_change: Box<dyn Fn(String) -> M>,
    max_length: Option<usize>,
    error: Option<String>,
}

impl<M: Clone + 'static> TextAreaField<M> {
    /// Create a new text area field.
    pub fn new(
        label: impl Into<String>,
        value: &str,
        placeholder: impl Into<String>,
        on_change: impl Fn(String) -> M + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            value: value.to_string(),
            placeholder: placeholder.into(),
            on_change: Box::new(on_change),
            max_length: None,
            error: None,
        }
    }

    /// Set maximum character length.
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Set an error message.
    pub fn error(mut self, error: Option<impl Into<String>>) -> Self {
        self.error = error.map(Into::into);
        self
    }

    /// Build the text area field element.
    pub fn view(self) -> Element<'static, M> {
        // Reuse TextField logic with slight modifications
        TextField::new(&self.label, &self.value, &self.placeholder, self.on_change)
            .max_length(self.max_length.unwrap_or(500))
            .error(self.error)
            .view()
    }
}
