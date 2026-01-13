//! Status badge component.
//!
//! Visual indicators for status, progress, and validation states.

use iced::widget::{container, row, text};
use iced::{Border, Element, Length};

use crate::theme::{
    BORDER_RADIUS_FULL, ERROR, ERROR_LIGHT, GRAY_400, GRAY_500, INFO, INFO_LIGHT, SUCCESS,
    SUCCESS_LIGHT, WARNING, WARNING_LIGHT,
};

// =============================================================================
// STATUS ENUM
// =============================================================================

/// Status type for badges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Success/complete state (green)
    Success,
    /// Warning/attention state (amber)
    Warning,
    /// Error/failure state (red)
    Error,
    /// Informational state (blue)
    Info,
    /// Neutral/inactive state (gray)
    Neutral,
}

impl Status {
    /// Get the foreground color for this status.
    pub fn color(&self) -> iced::Color {
        match self {
            Status::Success => SUCCESS,
            Status::Warning => WARNING,
            Status::Error => ERROR,
            Status::Info => INFO,
            Status::Neutral => GRAY_500,
        }
    }

    /// Get the background color for this status.
    pub fn background(&self) -> iced::Color {
        match self {
            Status::Success => SUCCESS_LIGHT,
            Status::Warning => WARNING_LIGHT,
            Status::Error => ERROR_LIGHT,
            Status::Info => INFO_LIGHT,
            Status::Neutral => GRAY_400,
        }
    }
}

// =============================================================================
// STATUS BADGE
// =============================================================================

/// Creates a status badge with colored indicator.
///
/// A pill-shaped badge with background color based on status.
///
/// # Arguments
///
/// * `label` - Badge text
/// * `status` - The status type (Success, Warning, Error, Info, Neutral)
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::{status_badge, Status};
///
/// let badge = status_badge("Mapped", Status::Success);
/// let warning = status_badge("3 Issues", Status::Warning);
/// ```
pub fn status_badge<'a, M: 'a>(label: impl Into<String>, status: Status) -> Element<'a, M> {
    let bg_color = status.background();
    let text_color = status.color();
    let label_str = label.into();

    container(text(label_str).size(12).color(text_color))
        .padding([4.0, 10.0])
        .style(move |_theme| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: BORDER_RADIUS_FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// Creates a status badge with an icon.
///
/// Includes an icon before the label text.
pub fn status_badge_with_icon<'a, M: 'a>(
    icon_char: char,
    label: impl Into<String>,
    status: Status,
) -> Element<'a, M> {
    let bg_color = status.background();
    let text_color = status.color();
    let label_str = label.into();

    let icon = text(icon_char.to_string())
        .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
        .size(11)
        .color(text_color);

    let label_text = text(label_str).size(12).color(text_color);

    container(
        row![icon, label_text]
            .spacing(6.0)
            .align_y(iced::Alignment::Center),
    )
    .padding([4.0, 10.0])
    .style(move |_theme| container::Style {
        background: Some(bg_color.into()),
        border: Border {
            radius: BORDER_RADIUS_FULL.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

/// Creates a small dot indicator.
///
/// A minimal status indicator without text, just a colored dot.
pub fn status_dot<'a, M: 'a>(status: Status) -> Element<'a, M> {
    let color = status.color();

    container(text(""))
        .width(Length::Fixed(8.0))
        .height(Length::Fixed(8.0))
        .style(move |_theme| container::Style {
            background: Some(color.into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// Creates a count badge (for notifications/errors).
///
/// A small circular badge typically used to show counts.
pub fn count_badge<'a, M: 'a>(count: usize, status: Status) -> Element<'a, M> {
    let bg_color = status.color();
    let count_text = if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    };

    container(text(count_text).size(11).color(iced::Color::WHITE))
        .padding([2.0, 6.0])
        .style(move |_theme| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: BORDER_RADIUS_FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

// =============================================================================
// MAPPING STATUS BADGES
// =============================================================================

/// Creates a badge for mapping status.
///
/// Specialized badge for variable mapping states.
pub fn mapping_status_badge<'a, M: 'a>(mapped: bool, required: bool) -> Element<'a, M> {
    if mapped {
        status_badge_with_icon('\u{f00c}', "Mapped", Status::Success)
    } else if required {
        status_badge_with_icon('\u{f071}', "Required", Status::Error)
    } else {
        status_badge("Unmapped", Status::Neutral)
    }
}

/// Creates a badge for validation status.
///
/// Shows validation state with issue count.
pub fn validation_badge<'a, M: 'a>(errors: usize, warnings: usize) -> Element<'a, M> {
    if errors > 0 {
        status_badge_with_icon(
            '\u{f057}', // times-circle
            format!("{} error{}", errors, if errors == 1 { "" } else { "s" }),
            Status::Error,
        )
    } else if warnings > 0 {
        status_badge_with_icon(
            '\u{f071}', // exclamation-triangle
            format!(
                "{} warning{}",
                warnings,
                if warnings == 1 { "" } else { "s" }
            ),
            Status::Warning,
        )
    } else {
        status_badge_with_icon('\u{f00c}', "Valid", Status::Success)
    }
}
