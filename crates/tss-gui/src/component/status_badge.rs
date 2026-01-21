//! Status badge component.
//!
//! Visual indicators for status, progress, and validation states.
//! Uses the semantic color system for accessibility mode support.

use iced::widget::{container, row, text};
use iced::{Border, Element, Length};
use iced_fonts::lucide;

use crate::theme::{BORDER_RADIUS_FULL, SemanticColor, ThemeConfig};

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
    pub fn color(&self, config: &ThemeConfig) -> iced::Color {
        match self {
            Status::Success => config.resolve(SemanticColor::StatusSuccess),
            Status::Warning => config.resolve(SemanticColor::StatusWarning),
            Status::Error => config.resolve(SemanticColor::StatusError),
            Status::Info => config.resolve(SemanticColor::StatusInfo),
            Status::Neutral => config.resolve(SemanticColor::TextMuted),
        }
    }

    /// Get the background color for this status.
    pub fn background(&self, config: &ThemeConfig) -> iced::Color {
        match self {
            Status::Success => config.resolve(SemanticColor::StatusSuccessLight),
            Status::Warning => config.resolve(SemanticColor::StatusWarningLight),
            Status::Error => config.resolve(SemanticColor::StatusErrorLight),
            Status::Info => config.resolve(SemanticColor::StatusInfoLight),
            Status::Neutral => config.resolve(SemanticColor::TextDisabled),
        }
    }

    /// Get the semantic color for this status (foreground).
    pub fn semantic_color(&self) -> SemanticColor {
        match self {
            Status::Success => SemanticColor::StatusSuccess,
            Status::Warning => SemanticColor::StatusWarning,
            Status::Error => SemanticColor::StatusError,
            Status::Info => SemanticColor::StatusInfo,
            Status::Neutral => SemanticColor::TextMuted,
        }
    }
}

// =============================================================================
// STATUS BADGE
// =============================================================================

/// Creates a status badge with colored indicator.
///
/// A pill-shaped badge with background color based on status.
/// Uses the default theme configuration.
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
    status_badge_themed(&ThemeConfig::default(), label, status)
}

/// Creates a status badge with colored indicator using specific theme config.
pub fn status_badge_themed<'a, M: 'a>(
    config: &ThemeConfig,
    label: impl Into<String>,
    status: Status,
) -> Element<'a, M> {
    let bg_color = status.background(config);
    let text_color = status.color(config);
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
/// This function now accepts a lucide icon element directly.
pub fn status_badge_with_icon<'a, M: 'a>(
    icon: impl Into<Element<'a, M>>,
    label: impl Into<String>,
    status: Status,
) -> Element<'a, M> {
    status_badge_with_icon_themed(&ThemeConfig::default(), icon, label, status)
}

/// Creates a status badge with an icon using specific theme config.
pub fn status_badge_with_icon_themed<'a, M: 'a>(
    config: &ThemeConfig,
    icon: impl Into<Element<'a, M>>,
    label: impl Into<String>,
    status: Status,
) -> Element<'a, M> {
    let bg_color = status.background(config);
    let text_color = status.color(config);
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
    status_dot_themed(&ThemeConfig::default(), status)
}

/// Creates a small dot indicator using specific theme config.
pub fn status_dot_themed<'a, M: 'a>(config: &ThemeConfig, status: Status) -> Element<'a, M> {
    let color = status.color(config);

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
    count_badge_themed(&ThemeConfig::default(), count, status)
}

/// Creates a count badge using specific theme config.
pub fn count_badge_themed<'a, M: 'a>(
    config: &ThemeConfig,
    count: usize,
    status: Status,
) -> Element<'a, M> {
    let bg_color = status.color(config);
    let text_on_accent = config.resolve(SemanticColor::TextOnAccent);
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
    mapping_status_badge_themed(&ThemeConfig::default(), mapped, required)
}

/// Creates a badge for mapping status using specific theme config.
pub fn mapping_status_badge_themed<'a, M: 'a>(
    config: &ThemeConfig,
    mapped: bool,
    required: bool,
) -> Element<'a, M> {
    let success_color = config.resolve(SemanticColor::StatusSuccess);
    let error_color = config.resolve(SemanticColor::StatusError);

    if mapped {
        status_badge_with_icon_themed(
            config,
            lucide::check().size(11).color(success_color),
            "Mapped",
            Status::Success,
        )
    } else if required {
        status_badge_with_icon_themed(
            config,
            lucide::triangle_alert().size(11).color(error_color),
            "Required",
            Status::Error,
        )
    } else {
        status_badge_themed(config, "Unmapped", Status::Neutral)
    }
}

/// Creates a badge for validation status.
///
/// Shows validation state with issue count.
pub fn validation_badge<'a, M: 'a>(errors: usize, warnings: usize) -> Element<'a, M> {
    validation_badge_themed(&ThemeConfig::default(), errors, warnings)
}

/// Creates a badge for validation status using specific theme config.
pub fn validation_badge_themed<'a, M: 'a>(
    config: &ThemeConfig,
    errors: usize,
    warnings: usize,
) -> Element<'a, M> {
    let error_color = config.resolve(SemanticColor::StatusError);
    let warning_color = config.resolve(SemanticColor::StatusWarning);
    let success_color = config.resolve(SemanticColor::StatusSuccess);

    if errors > 0 {
        status_badge_with_icon_themed(
            config,
            lucide::circle_x().size(11).color(error_color),
            format!("{} error{}", errors, if errors == 1 { "" } else { "s" }),
            Status::Error,
        )
    } else if warnings > 0 {
        status_badge_with_icon_themed(
            config,
            lucide::triangle_alert().size(11).color(warning_color),
            format!(
                "{} warning{}",
                warnings,
                if warnings == 1 { "" } else { "s" }
            ),
            Status::Warning,
        )
    } else {
        status_badge_with_icon_themed(
            config,
            lucide::check().size(11).color(success_color),
            "Valid",
            Status::Success,
        )
    }
}
