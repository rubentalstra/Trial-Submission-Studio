//! Theme and styling constants
//!
//! Colors, spacing, and visual constants for the application.

#[allow(dead_code)]
use egui::Color32;

/// Light mode colors
pub mod light {
    use super::*;

    pub const BG_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
    pub const BG_SECONDARY: Color32 = Color32::from_rgb(249, 250, 251);
    pub const BG_HOVER: Color32 = Color32::from_rgb(243, 244, 246);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(17, 24, 39);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(107, 114, 128);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(156, 163, 175);

    pub const BORDER: Color32 = Color32::from_rgb(229, 231, 235);
    pub const ACCENT: Color32 = Color32::from_rgb(59, 130, 246);
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);
    pub const WARNING: Color32 = Color32::from_rgb(234, 179, 8);
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68);
}

/// Dark mode colors
pub mod dark {
    use super::*;

    pub const BG_PRIMARY: Color32 = Color32::from_rgb(24, 24, 27);
    pub const BG_SECONDARY: Color32 = Color32::from_rgb(39, 39, 42);
    pub const BG_HOVER: Color32 = Color32::from_rgb(63, 63, 70);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(250, 250, 250);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(161, 161, 170);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(113, 113, 122);

    pub const BORDER: Color32 = Color32::from_rgb(63, 63, 70);
    pub const ACCENT: Color32 = Color32::from_rgb(96, 165, 250);
    pub const SUCCESS: Color32 = Color32::from_rgb(74, 222, 128);
    pub const WARNING: Color32 = Color32::from_rgb(250, 204, 21);
    pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113);
}

/// Spacing constants
pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 32.0;
}

/// Get colors for current theme
pub fn colors(dark_mode: bool) -> ThemeColors {
    if dark_mode {
        ThemeColors {
            bg_primary: dark::BG_PRIMARY,
            bg_secondary: dark::BG_SECONDARY,
            bg_hover: dark::BG_HOVER,
            text_primary: dark::TEXT_PRIMARY,
            text_secondary: dark::TEXT_SECONDARY,
            text_muted: dark::TEXT_MUTED,
            border: dark::BORDER,
            accent: dark::ACCENT,
            success: dark::SUCCESS,
            warning: dark::WARNING,
            error: dark::ERROR,
        }
    } else {
        ThemeColors {
            bg_primary: light::BG_PRIMARY,
            bg_secondary: light::BG_SECONDARY,
            bg_hover: light::BG_HOVER,
            text_primary: light::TEXT_PRIMARY,
            text_secondary: light::TEXT_SECONDARY,
            text_muted: light::TEXT_MUTED,
            border: light::BORDER,
            accent: light::ACCENT,
            success: light::SUCCESS,
            warning: light::WARNING,
            error: light::ERROR,
        }
    }
}

/// Theme colors struct for easy access
#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_hover: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
}
