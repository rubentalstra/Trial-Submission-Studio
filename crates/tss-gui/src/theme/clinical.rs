//! Professional Clinical theme implementation for Trial Submission Studio.
//!
//! This module provides the custom theme that gives the application its
//! distinctive clinical/regulatory aesthetic.

#![allow(dead_code)]

use iced::theme::Palette;
use iced::widget::{button, container, progress_bar, text_input};
use iced::{Border, Color, Shadow, Theme, Vector};

use super::ThemeConfig;
use super::context::{colors, current_config};
use super::spacing;

// =============================================================================
// THEME CREATION
// =============================================================================

/// Creates the Professional Clinical theme with the specified configuration.
///
/// This is the primary theme creation function that respects accessibility
/// settings and theme mode (light/dark).
pub fn clinical_theme(config: &ThemeConfig) -> Theme {
    let c = colors();

    let custom_palette = Palette {
        background: c.background_primary,
        text: c.text_primary,
        primary: c.accent_primary,
        success: c.status_success,
        warning: c.status_warning,
        danger: c.status_error,
    };

    let theme_name = format!(
        "Clinical {} ({})",
        if config.is_dark() { "Dark" } else { "Light" },
        config.accessibility_mode.label()
    );

    Theme::custom(theme_name, custom_palette)
}

/// Creates the Professional Clinical light theme with default settings.
///
/// This is a convenience function that creates the standard light theme
/// with no accessibility modifications. For customized themes, use
/// `clinical_theme()` with a `ThemeConfig`.
pub fn clinical_light() -> Theme {
    clinical_theme(&current_config())
}

// =============================================================================
// BUTTON STYLES
// =============================================================================

/// Primary button style - main actions
pub fn button_primary(_theme: &Theme, status: button::Status) -> button::Style {
    let c = colors();

    match status {
        button::Status::Active => button::Style {
            background: Some(c.accent_primary.into()),
            text_color: c.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: c.shadow,
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(c.accent_hover.into()),
            text_color: c.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: c.shadow_strong,
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(c.accent_pressed.into()),
            text_color: c.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(c.accent_disabled.into()),
            text_color: c.text_muted,
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

/// Secondary button style - alternative actions
pub fn button_secondary(_theme: &Theme, status: button::Status) -> button::Style {
    let c = colors();

    match status {
        button::Status::Active => button::Style {
            background: Some(c.background_elevated.into()),
            text_color: c.text_secondary,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: c.border_default,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(c.background_primary.into()),
            text_color: c.text_secondary,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: c.text_disabled,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(c.background_secondary.into()),
            text_color: c.text_secondary,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: c.border_default,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(c.background_secondary.into()),
            text_color: c.text_disabled,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: c.border_subtle,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
    }
}

/// Danger button style - destructive actions
pub fn button_danger(_theme: &Theme, status: button::Status) -> button::Style {
    let c = colors();

    match status {
        button::Status::Active => button::Style {
            background: Some(c.status_error.into()),
            text_color: c.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: c.shadow,
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(c.danger_hover.into()),
            text_color: c.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: c.shadow,
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(c.danger_pressed.into()),
            text_color: c.text_on_accent,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(c.accent_disabled.into()),
            text_color: c.text_muted,
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

/// Ghost button style - minimal visual weight
pub fn button_ghost(_theme: &Theme, status: button::Status) -> button::Style {
    let c = colors();

    match status {
        button::Status::Active => button::Style {
            background: None,
            text_color: c.accent_primary,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(c.accent_primary_light.into()),
            text_color: c.accent_primary,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(c.accent_primary_medium.into()),
            text_color: c.accent_pressed,
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
            text_color: c.text_disabled,
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

/// Card container style - elevated surface
pub fn container_card(_theme: &Theme) -> container::Style {
    let c = colors();

    container::Style {
        background: Some(c.background_elevated.into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_MD.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: c.border_subtle,
        },
        shadow: Shadow {
            color: c.shadow,
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: None,
        ..Default::default()
    }
}

/// Modal container style - dialog overlay
pub fn container_modal(_theme: &Theme) -> container::Style {
    let c = colors();

    container::Style {
        background: Some(c.background_elevated.into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_LG.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: c.border_subtle,
        },
        shadow: Shadow {
            color: c.shadow_strong,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        text_color: None,
        ..Default::default()
    }
}

/// Sidebar container style - navigation panel
pub fn container_sidebar(_theme: &Theme) -> container::Style {
    let c = colors();

    container::Style {
        background: Some(c.background_secondary.into()),
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

/// Surface container style - subtle elevation
pub fn container_surface(_theme: &Theme) -> container::Style {
    let c = colors();

    container::Style {
        background: Some(c.background_secondary.into()),
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

/// Inset container style - recessed area
pub fn container_inset(_theme: &Theme) -> container::Style {
    let c = colors();

    container::Style {
        background: Some(c.background_inset.into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_SM.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: c.border_subtle,
        },
        shadow: Shadow::default(),
        text_color: None,
        ..Default::default()
    }
}

// =============================================================================
// TEXT INPUT STYLES
// =============================================================================

/// Default text input style
pub fn text_input_default(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let c = colors();

    match status {
        text_input::Status::Active => text_input::Style {
            background: c.background_elevated.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: c.border_default,
            },
            icon: c.text_muted,
            placeholder: c.text_disabled,
            value: c.text_primary,
            selection: c.accent_primary_medium,
        },
        text_input::Status::Hovered => text_input::Style {
            background: c.background_elevated.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: c.text_disabled,
            },
            icon: c.text_muted,
            placeholder: c.text_disabled,
            value: c.text_primary,
            selection: c.accent_primary_medium,
        },
        text_input::Status::Focused { .. } => text_input::Style {
            background: c.background_elevated.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_MEDIUM,
                color: c.border_focused,
            },
            icon: c.text_muted,
            placeholder: c.text_disabled,
            value: c.text_primary,
            selection: c.accent_primary_medium,
        },
        text_input::Status::Disabled => text_input::Style {
            background: c.background_secondary.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: c.border_default,
            },
            icon: c.text_disabled,
            placeholder: c.text_disabled,
            value: c.text_muted,
            selection: c.border_subtle,
        },
    }
}

// =============================================================================
// PROGRESS BAR STYLES
// =============================================================================

/// Primary progress bar style
pub fn progress_bar_primary(_theme: &Theme) -> progress_bar::Style {
    let c = colors();

    progress_bar::Style {
        background: c.border_subtle.into(),
        bar: c.accent_primary.into(),
        border: Border {
            radius: spacing::BORDER_RADIUS_FULL.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    }
}

/// Success progress bar style
pub fn progress_bar_success(_theme: &Theme) -> progress_bar::Style {
    let c = colors();

    progress_bar::Style {
        background: c.border_subtle.into(),
        bar: c.status_success.into(),
        border: Border {
            radius: spacing::BORDER_RADIUS_FULL.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    }
}
