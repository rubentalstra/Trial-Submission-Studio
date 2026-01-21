//! Status badge component.
//!
//! Visual indicators for status, progress, and validation states.
//! Uses the semantic color system for accessibility mode support.

use iced::widget::{container, row, text};
use iced::{Border, Element, Length};
use iced_fonts::lucide;

use crate::theme::{BORDER_RADIUS_FULL, colors};

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
        let c = colors();
        match self {
            Status::Success => c.status_success,
            Status::Warning => c.status_warning,
            Status::Error => c.status_error,
            Status::Info => c.status_info,
            Status::Neutral => c.text_muted,
        }
    }

    /// Get the background color for this status.
    pub fn background(&self) -> iced::Color {
        let c = colors();
        match self {
            Status::Success => c.status_success_light,
            Status::Warning => c.status_warning_light,
            Status::Error => c.status_error_light,
            Status::Info => c.status_info_light,
            Status::Neutral => c.text_disabled,
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
    icon: impl Into<Element<'a, M>>,
    label: impl Into<String>,
    status: Status,
) -> Element<'a, M> {
    let bg_color = status.background();
    let text_color = status.color();
    let label_str = label.into();

    let label_text = text(label_str).size(12).color(text_color);

    container(
        row![icon.into(), label_text]
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
    let text_on_accent = colors().text_on_accent;
    let count_text = if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    };

    container(text(count_text).size(11).color(text_on_accent))
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
    let c = colors();

    if mapped {
        status_badge_with_icon(
            lucide::check().size(11).color(c.status_success),
            "Mapped",
            Status::Success,
        )
    } else if required {
        status_badge_with_icon(
            lucide::triangle_alert().size(11).color(c.status_error),
            "Required",
            Status::Error,
        )
    } else {
        status_badge("Unmapped", Status::Neutral)
    }
}

/// Creates a badge for validation status.
///
/// Shows validation state with issue count.
pub fn validation_badge<'a, M: 'a>(errors: usize, warnings: usize) -> Element<'a, M> {
    let c = colors();

    if errors > 0 {
        status_badge_with_icon(
            lucide::circle_x().size(11).color(c.status_error),
            format!("{} error{}", errors, if errors == 1 { "" } else { "s" }),
            Status::Error,
        )
    } else if warnings > 0 {
        status_badge_with_icon(
            lucide::triangle_alert().size(11).color(c.status_warning),
            format!(
                "{} warning{}",
                warnings,
                if warnings == 1 { "" } else { "s" }
            ),
            Status::Warning,
        )
    } else {
        status_badge_with_icon(
            lucide::check().size(11).color(c.status_success),
            "Valid",
            Status::Success,
        )
    }
}
