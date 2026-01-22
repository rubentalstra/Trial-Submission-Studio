//! Professional Clinical theme implementation for Trial Submission Studio.
//!
//! This module provides the custom theme and widget style functions that give
//! the application its distinctive clinical/regulatory aesthetic.
//!
//! # Usage
//!
//! Style functions receive `&Theme` and use it to access colors:
//!
//! ```rust,ignore
//! use crate::theme::{button_primary, ClinicalColors};
//!
//! // Use a pre-defined style function
//! button(text("Save")).style(button_primary)
//!
//! // Or create custom styles inside closures
//! container(content).style(|theme: &Theme| {
//!     let palette = theme.extended_palette();
//!     let clinical = theme.clinical();
//!     container::Style {
//!         background: Some(palette.background.base.color.into()),
//!         border: Border { color: clinical.border_default, ..Default::default() },
//!         ..Default::default()
//!     }
//! })
//! ```

#![allow(dead_code)]

use iced::widget::{button, container, progress_bar, text_input};
use iced::{Border, Color, Shadow, Theme, Vector};

use super::colors::ClinicalColors;
use super::palette::{AccessibilityMode, ThemeMode, clinical_palette};
use super::spacing;

// =============================================================================
// THEME CREATION
// =============================================================================

/// Creates the Professional Clinical theme with the specified configuration.
///
/// This is the primary theme creation function that respects accessibility
/// settings and theme mode (light/dark/system).
///
/// # Arguments
///
/// * `theme_mode` - Light, Dark, or System mode
/// * `accessibility_mode` - Color vision accessibility mode
/// * `system_is_dark` - Whether the system is in dark mode (for System theme mode)
pub fn clinical_theme(
    theme_mode: ThemeMode,
    accessibility_mode: AccessibilityMode,
    system_is_dark: bool,
) -> Theme {
    let palette = clinical_palette(theme_mode, accessibility_mode, system_is_dark);
    let is_dark = theme_mode.is_dark(system_is_dark);

    let theme_name = format!(
        "Clinical {} ({})",
        if is_dark { "Dark" } else { "Light" },
        accessibility_mode.label()
    );

    Theme::custom(theme_name, palette)
}

// =============================================================================
// BUTTON STYLES
// =============================================================================

/// Primary button style - main actions.
///
/// Uses the theme's primary color with appropriate text contrast.
pub fn button_primary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let clinical = theme.clinical();

    match status {
        button::Status::Active => button::Style {
            background: Some(palette.primary.base.color.into()),
            text_color: clinical.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: clinical.shadow,
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(clinical.accent_hover.into()),
            text_color: clinical.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: clinical.shadow_strong,
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(clinical.accent_pressed.into()),
            text_color: clinical.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(clinical.accent_disabled.into()),
            text_color: clinical.text_muted,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
    }
}

/// Secondary button style - alternative actions.
///
/// Uses a subtle background with border emphasis.
pub fn button_secondary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let clinical = theme.clinical();

    match status {
        button::Status::Active => button::Style {
            background: Some(clinical.background_elevated.into()),
            text_color: clinical.text_secondary,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: clinical.border_default,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(palette.background.base.color.into()),
            text_color: clinical.text_secondary,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: clinical.text_disabled,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(clinical.background_secondary.into()),
            text_color: clinical.text_secondary,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: clinical.border_default,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(clinical.background_secondary.into()),
            text_color: clinical.text_disabled,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: clinical.border_subtle,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
    }
}

/// Danger button style - destructive actions.
///
/// Uses the danger color with states for hover/pressed.
pub fn button_danger(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let clinical = theme.clinical();

    match status {
        button::Status::Active => button::Style {
            background: Some(palette.danger.base.color.into()),
            text_color: clinical.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: clinical.shadow,
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(clinical.danger_hover.into()),
            text_color: clinical.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: clinical.shadow,
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(clinical.danger_pressed.into()),
            text_color: clinical.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(clinical.accent_disabled.into()),
            text_color: clinical.text_muted,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
    }
}

/// Ghost button style - minimal visual weight.
///
/// Transparent background with text-only appearance.
pub fn button_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let clinical = theme.clinical();

    match status {
        button::Status::Active => button::Style {
            background: None,
            text_color: palette.primary.base.color,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(clinical.accent_primary_light.into()),
            text_color: palette.primary.base.color,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(clinical.accent_primary_medium.into()),
            text_color: clinical.accent_pressed,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: None,
            text_color: clinical.text_disabled,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
    }
}

// =============================================================================
// CONTAINER STYLES
// =============================================================================

/// Card container style - elevated surface.
pub fn container_card(theme: &Theme) -> container::Style {
    let clinical = theme.clinical();

    container::Style {
        background: Some(clinical.background_elevated.into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_MD.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: clinical.border_subtle,
        },
        shadow: Shadow {
            color: clinical.shadow,
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: None,
        ..Default::default()
    }
}

/// Modal container style - dialog overlay.
pub fn container_modal(theme: &Theme) -> container::Style {
    let clinical = theme.clinical();

    container::Style {
        background: Some(clinical.background_elevated.into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_LG.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: clinical.border_subtle,
        },
        shadow: Shadow {
            color: clinical.shadow_strong,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        text_color: None,
        ..Default::default()
    }
}

/// Sidebar container style - navigation panel.
pub fn container_sidebar(theme: &Theme) -> container::Style {
    let clinical = theme.clinical();

    container::Style {
        background: Some(clinical.background_secondary.into()),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        text_color: None,
        ..Default::default()
    }
}

/// Surface container style - subtle elevation.
pub fn container_surface(theme: &Theme) -> container::Style {
    let clinical = theme.clinical();

    container::Style {
        background: Some(clinical.background_secondary.into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_SM.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        text_color: None,
        ..Default::default()
    }
}

/// Inset container style - recessed area.
pub fn container_inset(theme: &Theme) -> container::Style {
    let clinical = theme.clinical();

    container::Style {
        background: Some(clinical.background_inset.into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_SM.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: clinical.border_subtle,
        },
        shadow: Shadow::default(),
        text_color: None,
        ..Default::default()
    }
}

// =============================================================================
// TEXT INPUT STYLES
// =============================================================================

/// Default text input style.
pub fn text_input_default(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.extended_palette();
    let clinical = theme.clinical();

    match status {
        text_input::Status::Active => text_input::Style {
            background: clinical.background_elevated.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: clinical.border_default,
            },
            icon: clinical.text_muted,
            placeholder: clinical.text_disabled,
            value: palette.background.base.text,
            selection: clinical.accent_primary_medium,
        },
        text_input::Status::Hovered => text_input::Style {
            background: clinical.background_elevated.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: clinical.text_disabled,
            },
            icon: clinical.text_muted,
            placeholder: clinical.text_disabled,
            value: palette.background.base.text,
            selection: clinical.accent_primary_medium,
        },
        text_input::Status::Focused { .. } => text_input::Style {
            background: clinical.background_elevated.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_MEDIUM,
                color: clinical.border_focused,
            },
            icon: clinical.text_muted,
            placeholder: clinical.text_disabled,
            value: palette.background.base.text,
            selection: clinical.accent_primary_medium,
        },
        text_input::Status::Disabled => text_input::Style {
            background: clinical.background_secondary.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: clinical.border_default,
            },
            icon: clinical.text_disabled,
            placeholder: clinical.text_disabled,
            value: clinical.text_muted,
            selection: clinical.border_subtle,
        },
    }
}

// =============================================================================
// PROGRESS BAR STYLES
// =============================================================================

/// Primary progress bar style.
pub fn progress_bar_primary(theme: &Theme) -> progress_bar::Style {
    let palette = theme.extended_palette();
    let clinical = theme.clinical();

    progress_bar::Style {
        background: clinical.border_subtle.into(),
        bar: palette.primary.base.color.into(),
        border: Border {
            radius: spacing::BORDER_RADIUS_FULL.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    }
}

/// Success progress bar style.
pub fn progress_bar_success(theme: &Theme) -> progress_bar::Style {
    let palette = theme.extended_palette();
    let clinical = theme.clinical();

    progress_bar::Style {
        background: clinical.border_subtle.into(),
        bar: palette.success.base.color.into(),
        border: Border {
            radius: spacing::BORDER_RADIUS_FULL.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    }
}
