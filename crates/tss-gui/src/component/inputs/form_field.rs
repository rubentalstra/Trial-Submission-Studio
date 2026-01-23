//! Form field components.
//!
//! Input fields with labels, validation, and error display.

use iced::widget::{column, container, text, text_input};
use iced::{Element, Length, Theme};

use crate::theme::{ClinicalColors, SPACING_XS, text_input_default};

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
    let has_error = error.is_some();

    let label_text = text(label).size(13).style(|theme: &Theme| text::Style {
        color: Some(theme.clinical().text_muted),
    });

    let input = text_input(placeholder, value)
        .on_input(on_change)
        .padding(10.0)
        .width(Length::Fill)
        .style(if has_error {
            text_input_error_style
        } else {
            text_input_default
        });

    let mut content = column![label_text, input].spacing(SPACING_XS);

    if let Some(err) = error {
        let error_text = text(err).size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().danger.base.color),
        });
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

    let label_text = text(label).size(13).style(|theme: &Theme| text::Style {
        color: Some(theme.clinical().text_muted),
    });

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

// =============================================================================
// STYLES
// =============================================================================

/// Text input style for error state
fn text_input_error_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let clinical = theme.clinical();
    let error_color = clinical.border_error;

    let mut style = text_input_default(theme, status);
    style.border.color = error_color;
    style.border.width = 2.0;
    style
}
