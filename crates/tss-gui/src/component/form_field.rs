//! Form field components.
//!
//! Input fields with labels, validation, and error display.

use iced::widget::{column, container, text, text_input};
use iced::{Border, Element, Length};

use crate::theme::{
    BORDER_RADIUS_SM, ERROR, GRAY_300, GRAY_600, GRAY_900, SPACING_XS, text_input_default,
};

// =============================================================================
// FORM FIELD
// =============================================================================

/// Creates a form field with label and optional error message.
///
/// # Arguments
///
/// * `label` - Field label text
/// * `value` - Current field value
/// * `placeholder` - Placeholder text
/// * `on_change` - Message factory for value changes
/// * `error` - Optional error message to display
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::form_field;
///
/// let field = form_field(
///     "Study Name",
///     &state.study_name,
///     "Enter study name...",
///     Message::StudyNameChanged,
///     state.name_error.as_deref(),
/// );
/// ```
pub fn form_field<'a, M: Clone + 'a>(
    label: &'a str,
    value: &'a str,
    placeholder: &'a str,
    on_change: impl Fn(String) -> M + 'a,
    error: Option<&'a str>,
) -> Element<'a, M> {
    let label_text = text(label).size(13).color(GRAY_600);

    let input = text_input(placeholder, value)
        .on_input(on_change)
        .padding(10.0)
        .width(Length::Fill)
        .style(if error.is_some() {
            text_input_error_style
        } else {
            text_input_default
        });

    let mut content = column![label_text, input].spacing(SPACING_XS);

    if let Some(err) = error {
        let error_text = text(err).size(12).color(ERROR);
        content = content.push(error_text);
    }

    container(content).width(Length::Fill).into()
}

/// Creates a number input field.
///
/// Validates input to only allow numeric values within the specified range.
///
/// # Arguments
///
/// * `label` - Field label text
/// * `value` - Current numeric value
/// * `on_change` - Message factory for value changes
/// * `min` - Optional minimum value
/// * `max` - Optional maximum value
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::number_field;
///
/// let field = number_field(
///     "Header Rows",
///     state.header_rows,
///     Message::HeaderRowsChanged,
///     Some(0),
///     Some(10),
/// );
/// ```
pub fn number_field<'a, M: Clone + 'a>(
    label: &'a str,
    value: usize,
    on_change: impl Fn(usize) -> M + 'a,
    min: Option<usize>,
    max: Option<usize>,
) -> Element<'a, M> {
    let value_str = value.to_string();

    let label_text = text(label).size(13).color(GRAY_600);

    let input = text_input("0", &value_str)
        .on_input(move |s| {
            let parsed = s.parse().unwrap_or(value);
            let clamped = match (min, max) {
                (Some(lo), Some(hi)) => parsed.clamp(lo, hi),
                (Some(lo), None) => parsed.max(lo),
                (None, Some(hi)) => parsed.min(hi),
                (None, None) => parsed,
            };
            on_change(clamped)
        })
        .padding(10.0)
        .width(Length::Fixed(100.0))
        .style(text_input_default);

    column![label_text, input].spacing(SPACING_XS).into()
}

/// Creates a read-only display field.
///
/// Shows a value that cannot be edited (for display purposes).
pub fn display_field<'a, M: 'a>(label: &'a str, value: &'a str) -> Element<'a, M> {
    let label_text = text(label).size(13).color(GRAY_600);

    let value_text = container(text(value).size(14).color(GRAY_900))
        .padding(10.0)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(crate::theme::GRAY_100.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                color: GRAY_300,
                width: 1.0,
            },
            ..Default::default()
        });

    column![label_text, value_text].spacing(SPACING_XS).into()
}

/// Creates a text area field (multi-line input).
///
/// For longer text input like descriptions or notes.
pub fn text_area_field<'a, M: Clone + 'a>(
    label: &'a str,
    value: &'a str,
    placeholder: &'a str,
    on_change: impl Fn(String) -> M + 'a,
    rows: u16,
) -> Element<'a, M> {
    let label_text = text(label).size(13).color(GRAY_600);

    // Note: Iced doesn't have a native textarea, so we simulate with a taller text_input
    // For true multi-line, would need text_editor widget
    let input = text_input(placeholder, value)
        .on_input(on_change)
        .padding(10.0)
        .width(Length::Fill)
        .style(text_input_default);

    // Create container with minimum height based on rows
    let min_height = (rows as f32) * 20.0 + 20.0;

    column![
        label_text,
        container(input).height(Length::Fixed(min_height)),
    ]
    .spacing(SPACING_XS)
    .into()
}

// =============================================================================
// STYLES
// =============================================================================

/// Text input style for error state
fn text_input_error_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input_default(_theme, status);
    style.border.color = ERROR;
    style.border.width = 2.0;
    style
}
