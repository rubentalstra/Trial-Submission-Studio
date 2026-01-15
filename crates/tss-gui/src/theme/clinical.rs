//! Professional Clinical theme implementation for Trial Submission Studio.
//!
//! This module provides the custom theme that gives the application its
//! distinctive clinical/regulatory aesthetic.

use iced::theme::Palette;
use iced::widget::{button, container, progress_bar, text_input};
use iced::{Border, Color, Shadow, Theme, Vector};

use super::palette;
use super::spacing;

// =============================================================================
// THEME CREATION
// =============================================================================

/// Creates the Professional Clinical light theme.
///
/// This is the primary theme for Trial Submission Studio, designed to:
/// - Convey trust and precision for FDA/regulatory work
/// - Provide excellent readability for long work sessions
/// - Use medical-inspired teal/cyan accent colors
pub fn clinical_light() -> Theme {
    let custom_palette = Palette {
        background: palette::GRAY_50,
        text: palette::GRAY_900,
        primary: palette::PRIMARY_500,
        success: palette::SUCCESS,
        warning: palette::WARNING,
        danger: palette::ERROR,
    };

    Theme::custom("Clinical Light".to_string(), custom_palette)
}

// =============================================================================
// BUTTON STYLES
// =============================================================================

/// Primary button style - main actions
pub fn button_primary(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(palette::PRIMARY_500.into()),
            text_color: palette::WHITE,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: palette::SHADOW,
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(palette::PRIMARY_600.into()),
            text_color: palette::WHITE,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: palette::SHADOW_STRONG,
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(palette::PRIMARY_700.into()),
            text_color: palette::WHITE,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(palette::GRAY_300.into()),
            text_color: palette::GRAY_500,
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
    match status {
        button::Status::Active => button::Style {
            background: Some(palette::WHITE.into()),
            text_color: palette::GRAY_700,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: palette::GRAY_300,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(palette::GRAY_50.into()),
            text_color: palette::GRAY_700,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: palette::GRAY_400,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(palette::GRAY_100.into()),
            text_color: palette::GRAY_700,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: palette::GRAY_300,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(palette::GRAY_100.into()),
            text_color: palette::GRAY_400,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: palette::GRAY_200,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
    }
}

/// Danger button style - destructive actions
pub fn button_danger(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(palette::ERROR.into()),
            text_color: palette::WHITE,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: palette::SHADOW,
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgb(0.75, 0.20, 0.20).into()),
            text_color: palette::WHITE,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: palette::SHADOW,
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.65, 0.15, 0.15).into()),
            text_color: palette::WHITE,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(palette::GRAY_300.into()),
            text_color: palette::GRAY_500,
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
    match status {
        button::Status::Active => button::Style {
            background: None,
            text_color: palette::PRIMARY_500,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(palette::PRIMARY_50.into()),
            text_color: palette::PRIMARY_500,
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(palette::PRIMARY_100.into()),
            text_color: palette::PRIMARY_700,
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
            text_color: palette::GRAY_400,
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
    container::Style {
        background: Some(palette::WHITE.into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_MD.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: palette::GRAY_200,
        },
        shadow: Shadow {
            color: palette::SHADOW,
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: None,
        ..Default::default()
    }
}

/// Modal container style - dialog overlay
pub fn container_modal(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::WHITE.into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_LG.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: palette::GRAY_200,
        },
        shadow: Shadow {
            color: palette::SHADOW_STRONG,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        text_color: None,
        ..Default::default()
    }
}

/// Sidebar container style - navigation panel
pub fn container_sidebar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::GRAY_100.into()),
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
    container::Style {
        background: Some(palette::GRAY_100.into()),
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
    container::Style {
        background: Some(palette::GRAY_50.into()),
        border: Border {
            radius: spacing::BORDER_RADIUS_SM.into(),
            width: spacing::BORDER_WIDTH_THIN,
            color: palette::GRAY_200,
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
    match status {
        text_input::Status::Active => text_input::Style {
            background: palette::WHITE.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: palette::GRAY_300,
            },
            icon: palette::GRAY_500,
            placeholder: palette::GRAY_400,
            value: palette::GRAY_900,
            selection: palette::PRIMARY_100,
        },
        text_input::Status::Hovered => text_input::Style {
            background: palette::WHITE.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: palette::GRAY_400,
            },
            icon: palette::GRAY_500,
            placeholder: palette::GRAY_400,
            value: palette::GRAY_900,
            selection: palette::PRIMARY_100,
        },
        text_input::Status::Focused { .. } => text_input::Style {
            background: palette::WHITE.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_MEDIUM,
                color: palette::PRIMARY_500,
            },
            icon: palette::GRAY_500,
            placeholder: palette::GRAY_400,
            value: palette::GRAY_900,
            selection: palette::PRIMARY_100,
        },
        text_input::Status::Disabled => text_input::Style {
            background: palette::GRAY_100.into(),
            border: Border {
                radius: spacing::BORDER_RADIUS_SM.into(),
                width: spacing::BORDER_WIDTH_THIN,
                color: palette::GRAY_300,
            },
            icon: palette::GRAY_400,
            placeholder: palette::GRAY_400,
            value: palette::GRAY_500,
            selection: palette::GRAY_200,
        },
    }
}

// =============================================================================
// PROGRESS BAR STYLES
// =============================================================================

/// Primary progress bar style
pub fn progress_bar_primary(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: palette::GRAY_200.into(),
        bar: palette::PRIMARY_500.into(),
        border: Border {
            radius: spacing::BORDER_RADIUS_FULL.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    }
}

/// Success progress bar style
pub fn progress_bar_success(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: palette::GRAY_200.into(),
        bar: palette::SUCCESS.into(),
        border: Border {
            radius: spacing::BORDER_RADIUS_FULL.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    }
}
