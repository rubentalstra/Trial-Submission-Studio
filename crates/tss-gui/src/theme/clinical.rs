//! Professional Clinical theme implementation for Trial Submission Studio.
//!
//! This module provides the custom theme that gives the application its
//! distinctive clinical/regulatory aesthetic.

#![allow(dead_code)]

use iced::theme::Palette;
use iced::widget::{button, container, progress_bar, text_input};
use iced::{Border, Color, Shadow, Theme, Vector};

use super::spacing;
use super::{SemanticColor, ThemeConfig};

// =============================================================================
// THEME CREATION
// =============================================================================

/// Creates the Professional Clinical theme with the specified configuration.
///
/// This is the primary theme creation function that respects accessibility
/// settings and theme mode (light/dark).
pub fn clinical_theme(config: &ThemeConfig) -> Theme {
    let custom_palette = Palette {
        background: config.resolve(SemanticColor::BackgroundPrimary),
        text: config.resolve(SemanticColor::TextPrimary),
        primary: config.resolve(SemanticColor::AccentPrimary),
        success: config.resolve(SemanticColor::StatusSuccess),
        warning: config.resolve(SemanticColor::StatusWarning),
        danger: config.resolve(SemanticColor::StatusError),
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
    clinical_theme(&ThemeConfig::default())
}

// =============================================================================
// BUTTON STYLES
// =============================================================================

/// Primary button style - main actions
pub fn button_primary(_theme: &Theme, status: button::Status) -> button::Style {
    let config = ThemeConfig::default();
    button_primary_themed(&config, status)
}

/// Primary button style with theme configuration
pub fn button_primary_themed(config: &ThemeConfig, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(config.resolve(SemanticColor::AccentPrimary).into()),
            text_color: config.resolve(SemanticColor::TextOnAccent),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: config.resolve(SemanticColor::Shadow),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(config.resolve(SemanticColor::AccentHover).into()),
            text_color: config.resolve(SemanticColor::TextOnAccent),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: config.resolve(SemanticColor::ShadowStrong),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(config.resolve(SemanticColor::AccentPressed).into()),
            text_color: config.resolve(SemanticColor::TextOnAccent),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(config.resolve(SemanticColor::AccentDisabled).into()),
            text_color: config.resolve(SemanticColor::TextMuted),
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
    let config = ThemeConfig::default();
    button_secondary_themed(&config, status)
}

/// Secondary button style with theme configuration
pub fn button_secondary_themed(config: &ThemeConfig, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(config.resolve(SemanticColor::BackgroundElevated).into()),
            text_color: config.resolve(SemanticColor::TextSecondary),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: config.resolve(SemanticColor::BorderDefault),
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(config.resolve(SemanticColor::BackgroundPrimary).into()),
            text_color: config.resolve(SemanticColor::TextSecondary),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: config.resolve(SemanticColor::TextDisabled),
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(config.resolve(SemanticColor::BackgroundSecondary).into()),
            text_color: config.resolve(SemanticColor::TextSecondary),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: config.resolve(SemanticColor::BorderDefault),
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(config.resolve(SemanticColor::BackgroundSecondary).into()),
            text_color: config.resolve(SemanticColor::TextDisabled),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: config.resolve(SemanticColor::BorderSubtle),
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
    }
}

/// Danger button style - destructive actions
pub fn button_danger(_theme: &Theme, status: button::Status) -> button::Style {
    let config = ThemeConfig::default();
    button_danger_themed(&config, status)
}

/// Danger button style with theme configuration
pub fn button_danger_themed(config: &ThemeConfig, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(config.resolve(SemanticColor::StatusError).into()),
            text_color: config.resolve(SemanticColor::TextOnAccent),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: config.resolve(SemanticColor::Shadow),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(config.resolve(SemanticColor::DangerHover).into()),
            text_color: config.resolve(SemanticColor::TextOnAccent),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: config.resolve(SemanticColor::Shadow),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(config.resolve(SemanticColor::DangerPressed).into()),
            text_color: config.resolve(SemanticColor::TextOnAccent),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(config.resolve(SemanticColor::AccentDisabled).into()),
            text_color: config.resolve(SemanticColor::TextMuted),
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
    let config = ThemeConfig::default();
    button_ghost_themed(&config, status)
}

/// Ghost button style with theme configuration
pub fn button_ghost_themed(config: &ThemeConfig, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: None,
            text_color: config.resolve(SemanticColor::AccentPrimary),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(config.resolve(SemanticColor::AccentPrimaryLight).into()),
            text_color: config.resolve(SemanticColor::AccentPrimary),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(config.resolve(SemanticColor::AccentPrimaryMedium).into()),
            text_color: config.resolve(SemanticColor::AccentPressed),
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
            text_color: config.resolve(SemanticColor::TextDisabled),
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
    let config = ThemeConfig::default();
    container_card_themed(&config)
}

/// Card container style with theme configuration
pub fn container_card_themed(config: &ThemeConfig) -> container::Style {
    container::Style {
        background: Some(config.resolve(SemanticColor::BackgroundElevated).into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_MD.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: config.resolve(SemanticColor::BorderSubtle),
        },
        shadow: Shadow {
            color: config.resolve(SemanticColor::Shadow),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: None,
        ..Default::default()
    }
}

/// Modal container style - dialog overlay
pub fn container_modal(_theme: &Theme) -> container::Style {
    let config = ThemeConfig::default();
    container_modal_themed(&config)
}

/// Modal container style with theme configuration
pub fn container_modal_themed(config: &ThemeConfig) -> container::Style {
    container::Style {
        background: Some(config.resolve(SemanticColor::BackgroundElevated).into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_LG.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: config.resolve(SemanticColor::BorderSubtle),
        },
        shadow: Shadow {
            color: config.resolve(SemanticColor::ShadowStrong),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        text_color: None,
        ..Default::default()
    }
}

/// Sidebar container style - navigation panel
pub fn container_sidebar(_theme: &Theme) -> container::Style {
    let config = ThemeConfig::default();
    container_sidebar_themed(&config)
}

/// Sidebar container style with theme configuration
pub fn container_sidebar_themed(config: &ThemeConfig) -> container::Style {
    container::Style {
        background: Some(config.resolve(SemanticColor::BackgroundSecondary).into()),
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
    let config = ThemeConfig::default();
    container_surface_themed(&config)
}

/// Surface container style with theme configuration
pub fn container_surface_themed(config: &ThemeConfig) -> container::Style {
    container::Style {
        background: Some(config.resolve(SemanticColor::BackgroundSecondary).into()),
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
    let config = ThemeConfig::default();
    container_inset_themed(&config)
}

/// Inset container style with theme configuration
pub fn container_inset_themed(config: &ThemeConfig) -> container::Style {
    container::Style {
        background: Some(config.resolve(SemanticColor::BackgroundInset).into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_SM.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: config.resolve(SemanticColor::BorderSubtle),
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
    let config = ThemeConfig::default();
    text_input_default_themed(&config, status)
}

/// Default text input style with theme configuration
pub fn text_input_default_themed(
    config: &ThemeConfig,
    status: text_input::Status,
) -> text_input::Style {
    match status {
        text_input::Status::Active => text_input::Style {
            background: config.resolve(SemanticColor::BackgroundElevated).into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: config.resolve(SemanticColor::BorderDefault),
            },
            icon: config.resolve(SemanticColor::TextMuted),
            placeholder: config.resolve(SemanticColor::TextDisabled),
            value: config.resolve(SemanticColor::TextPrimary),
            selection: config.resolve(SemanticColor::AccentPrimaryMedium),
        },
        text_input::Status::Hovered => text_input::Style {
            background: config.resolve(SemanticColor::BackgroundElevated).into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: config.resolve(SemanticColor::TextDisabled),
            },
            icon: config.resolve(SemanticColor::TextMuted),
            placeholder: config.resolve(SemanticColor::TextDisabled),
            value: config.resolve(SemanticColor::TextPrimary),
            selection: config.resolve(SemanticColor::AccentPrimaryMedium),
        },
        text_input::Status::Focused { .. } => text_input::Style {
            background: config.resolve(SemanticColor::BackgroundElevated).into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_MEDIUM,
                color: config.resolve(SemanticColor::BorderFocused),
            },
            icon: config.resolve(SemanticColor::TextMuted),
            placeholder: config.resolve(SemanticColor::TextDisabled),
            value: config.resolve(SemanticColor::TextPrimary),
            selection: config.resolve(SemanticColor::AccentPrimaryMedium),
        },
        text_input::Status::Disabled => text_input::Style {
            background: config.resolve(SemanticColor::BackgroundSecondary).into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: config.resolve(SemanticColor::BorderDefault),
            },
            icon: config.resolve(SemanticColor::TextDisabled),
            placeholder: config.resolve(SemanticColor::TextDisabled),
            value: config.resolve(SemanticColor::TextMuted),
            selection: config.resolve(SemanticColor::BorderSubtle),
        },
    }
}

// =============================================================================
// PROGRESS BAR STYLES
// =============================================================================

/// Primary progress bar style
pub fn progress_bar_primary(_theme: &Theme) -> progress_bar::Style {
    let config = ThemeConfig::default();
    progress_bar_primary_themed(&config)
}

/// Primary progress bar style with theme configuration
pub fn progress_bar_primary_themed(config: &ThemeConfig) -> progress_bar::Style {
    progress_bar::Style {
        background: config.resolve(SemanticColor::BorderSubtle).into(),
        bar: config.resolve(SemanticColor::AccentPrimary).into(),
        border: Border {
            radius: spacing::BORDER_RADIUS_FULL.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    }
}

/// Success progress bar style
pub fn progress_bar_success(_theme: &Theme) -> progress_bar::Style {
    let config = ThemeConfig::default();
    progress_bar_success_themed(&config)
}

/// Success progress bar style with theme configuration
pub fn progress_bar_success_themed(config: &ThemeConfig) -> progress_bar::Style {
    progress_bar::Style {
        background: config.resolve(SemanticColor::BorderSubtle).into(),
        bar: config.resolve(SemanticColor::StatusSuccess).into(),
        border: Border {
            radius: spacing::BORDER_RADIUS_FULL.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    }
}
