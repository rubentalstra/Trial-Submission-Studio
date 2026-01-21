//! Protanopia palette - optimized for red-blind color vision.
//!
//! Similar to deuteranopia but with adjusted brightness for red-blind perception.

use iced::Color;

use crate::theme::semantic::{Palette, SemanticColor};

/// Protanopia-optimized light palette.
pub struct ProtanopiaLight;

#[allow(clippy::enum_glob_use)]
impl Palette for ProtanopiaLight {
    fn resolve(&self, color: SemanticColor) -> Color {
        use SemanticColor::*;
        match color {
            // Status - Blue/Yellow-Orange/Magenta scheme
            StatusSuccess => Color::from_rgb(0.00, 0.62, 0.45), // #009E73 - Bluish green
            StatusSuccessLight => Color::from_rgb(0.85, 0.96, 0.92), // Light teal
            StatusWarning => Color::from_rgb(0.94, 0.89, 0.26), // #F0E442 - Yellow
            StatusWarningLight => Color::from_rgb(0.99, 0.98, 0.88), // Light yellow
            StatusError => Color::from_rgb(0.86, 0.15, 0.50),   // #DC267F - Magenta
            StatusErrorLight => Color::from_rgb(0.98, 0.88, 0.93), // Light magenta
            StatusInfo => Color::from_rgb(0.34, 0.71, 0.91),    // #56B4E9 - Sky blue
            StatusInfoLight => Color::from_rgb(0.90, 0.96, 0.99),

            // Mapping status
            MappingMapped => Color::from_rgb(0.00, 0.62, 0.45),
            MappingUnmapped => Color::from_rgb(0.86, 0.15, 0.50),
            MappingSuggested => Color::from_rgb(0.94, 0.89, 0.26),
            MappingNotCollected => Color::from_rgb(0.60, 0.60, 0.60),
            MappingInProgress => Color::from_rgb(0.94, 0.89, 0.26),

            // Backgrounds, Text - same as standard
            BackgroundPrimary => Color::from_rgb(0.98, 0.98, 0.99),
            BackgroundSecondary => Color::from_rgb(0.95, 0.95, 0.97),
            BackgroundElevated => Color::from_rgb(1.0, 1.0, 1.0),
            BackgroundInset => Color::from_rgb(0.98, 0.98, 0.99),

            TextPrimary => Color::from_rgb(0.10, 0.10, 0.12),
            TextSecondary => Color::from_rgb(0.30, 0.30, 0.35),
            TextMuted => Color::from_rgb(0.50, 0.50, 0.55),
            TextDisabled => Color::from_rgb(0.65, 0.65, 0.70),
            TextOnAccent => Color::from_rgb(1.0, 1.0, 1.0),

            // Interactive - Teal accent
            AccentPrimary => Color::from_rgb(0.00, 0.62, 0.45),
            AccentHover => Color::from_rgb(0.00, 0.52, 0.38),
            AccentPressed => Color::from_rgb(0.00, 0.42, 0.30),
            AccentDisabled => Color::from_rgb(0.82, 0.82, 0.86),
            AccentPrimaryLight => Color::from_rgb(0.85, 0.96, 0.92),
            AccentPrimaryMedium => Color::from_rgb(0.70, 0.92, 0.86),

            // Danger - Magenta
            DangerHover => Color::from_rgb(0.76, 0.10, 0.42),
            DangerPressed => Color::from_rgb(0.66, 0.05, 0.35),

            // Borders
            BorderDefault => Color::from_rgb(0.82, 0.82, 0.86),
            BorderSubtle => Color::from_rgb(0.90, 0.90, 0.93),
            BorderFocused => Color::from_rgb(0.00, 0.62, 0.45),
            BorderError => Color::from_rgb(0.86, 0.15, 0.50),

            // Special
            White => Color::from_rgb(1.0, 1.0, 1.0),
            Black => Color::from_rgb(0.0, 0.0, 0.0),
            Transparent => Color::from_rgba(0.0, 0.0, 0.0, 0.0),
            Shadow => Color::from_rgba(0.0, 0.0, 0.0, 0.08),
            ShadowStrong => Color::from_rgba(0.0, 0.0, 0.0, 0.16),
            Backdrop => Color::from_rgba(0.0, 0.0, 0.0, 0.5),
        }
    }
}
