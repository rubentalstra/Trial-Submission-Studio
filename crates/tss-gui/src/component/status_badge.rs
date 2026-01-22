//! Status badge component.
//!
//! Visual indicators for status, progress, and validation states.
//! Uses the semantic color system for accessibility mode support.

use iced::widget::{container, row, text};
use iced::{Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::theme::{BORDER_RADIUS_FULL, ClinicalColors};

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
    let label_str = label.into();

    container(text(label_str).size(12).style(move |theme: &Theme| {
        let palette = theme.extended_palette();
        let clinical = theme.clinical();
        let color = match status {
            Status::Success => palette.success.base.color,
            Status::Warning => palette.warning.base.color,
            Status::Error => palette.danger.base.color,
            Status::Info => palette.primary.base.color,
            Status::Neutral => clinical.text_muted,
        };
        text::Style { color: Some(color) }
    }))
    .padding([4.0, 10.0])
    .style(move |theme: &Theme| {
        let clinical = theme.clinical();
        let bg_color = match status {
            Status::Success => clinical.status_success_light,
            Status::Warning => clinical.status_warning_light,
            Status::Error => clinical.status_error_light,
            Status::Info => clinical.status_info_light,
            Status::Neutral => clinical.text_disabled,
        };
        container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: BORDER_RADIUS_FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        }
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
    let label_str = label.into();

    let label_text = text(label_str).size(12).style(move |theme: &Theme| {
        let palette = theme.extended_palette();
        let clinical = theme.clinical();
        let color = match status {
            Status::Success => palette.success.base.color,
            Status::Warning => palette.warning.base.color,
            Status::Error => palette.danger.base.color,
            Status::Info => palette.primary.base.color,
            Status::Neutral => clinical.text_muted,
        };
        text::Style { color: Some(color) }
    });

    container(
        row![icon.into(), label_text]
            .spacing(6.0)
            .align_y(iced::Alignment::Center),
    )
    .padding([4.0, 10.0])
    .style(move |theme: &Theme| {
        let clinical = theme.clinical();
        let bg_color = match status {
            Status::Success => clinical.status_success_light,
            Status::Warning => clinical.status_warning_light,
            Status::Error => clinical.status_error_light,
            Status::Info => clinical.status_info_light,
            Status::Neutral => clinical.text_disabled,
        };
        container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: BORDER_RADIUS_FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}

/// Creates a small dot indicator.
///
/// A minimal status indicator without text, just a colored dot.
pub fn status_dot<'a, M: 'a>(status: Status) -> Element<'a, M> {
    container(text(""))
        .width(Length::Fixed(8.0))
        .height(Length::Fixed(8.0))
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let clinical = theme.clinical();
            let color = match status {
                Status::Success => palette.success.base.color,
                Status::Warning => palette.warning.base.color,
                Status::Error => palette.danger.base.color,
                Status::Info => palette.primary.base.color,
                Status::Neutral => clinical.text_muted,
            };
            container::Style {
                background: Some(color.into()),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

/// Creates a count badge (for notifications/errors).
///
/// A small circular badge typically used to show counts.
pub fn count_badge<'a, M: 'a>(count: usize, status: Status) -> Element<'a, M> {
    let count_text = if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    };

    container(text(count_text).size(11).style(|theme: &Theme| {
        let clinical = theme.clinical();
        text::Style {
            color: Some(clinical.text_on_accent),
        }
    }))
    .padding([2.0, 6.0])
    .style(move |theme: &Theme| {
        let palette = theme.extended_palette();
        let clinical = theme.clinical();
        let bg_color = match status {
            Status::Success => palette.success.base.color,
            Status::Warning => palette.warning.base.color,
            Status::Error => palette.danger.base.color,
            Status::Info => palette.primary.base.color,
            Status::Neutral => clinical.text_muted,
        };
        container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: BORDER_RADIUS_FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        }
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
        status_badge_with_icon(
            container(lucide::check().size(11)).style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    text_color: Some(palette.success.base.color),
                    ..Default::default()
                }
            }),
            "Mapped",
            Status::Success,
        )
    } else if required {
        status_badge_with_icon(
            container(lucide::triangle_alert().size(11)).style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    text_color: Some(palette.danger.base.color),
                    ..Default::default()
                }
            }),
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
    if errors > 0 {
        status_badge_with_icon(
            container(lucide::circle_x().size(11)).style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    text_color: Some(palette.danger.base.color),
                    ..Default::default()
                }
            }),
            format!("{} error{}", errors, if errors == 1 { "" } else { "s" }),
            Status::Error,
        )
    } else if warnings > 0 {
        status_badge_with_icon(
            container(lucide::triangle_alert().size(11)).style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    text_color: Some(palette.warning.base.color),
                    ..Default::default()
                }
            }),
            format!(
                "{} warning{}",
                warnings,
                if warnings == 1 { "" } else { "s" }
            ),
            Status::Warning,
        )
    } else {
        status_badge_with_icon(
            container(lucide::check().size(11)).style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    text_color: Some(palette.success.base.color),
                    ..Default::default()
                }
            }),
            "Valid",
            Status::Success,
        )
    }
}
