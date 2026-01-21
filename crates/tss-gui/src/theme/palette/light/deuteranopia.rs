//! Deuteranopia palette - optimized for red-green (green-weak) color blindness.
//!
//! Uses blue-orange-purple color scheme to maximize distinguishability.
//! Based on IBM Color Blind Safe and Wong (Nature Methods, 2011) palettes.

use iced::Color;

use crate::theme::semantic::{Palette, SemanticColor};

/// Deuteranopia-optimized light palette.
pub struct DeuteranopiaLight;

#[allow(clippy::enum_glob_use)]
impl Palette for DeuteranopiaLight {
    fn resolve(&self, color: SemanticColor) -> Color {
        use SemanticColor::*;
        match color {
            // Status - Blue/Orange/Purple scheme (avoids red-green)
            StatusSuccess => Color::from_rgb(0.00, 0.45, 0.70), // #0072B2 - Blue
            StatusSuccessLight => Color::from_rgb(0.85, 0.93, 0.98), // Light blue
            StatusWarning => Color::from_rgb(0.90, 0.62, 0.00), // #E69F00 - Orange
            StatusWarningLight => Color::from_rgb(0.99, 0.95, 0.85), // Light orange
            StatusError => Color::from_rgb(0.80, 0.47, 0.65),   // #CC79A7 - Reddish purple
            StatusErrorLight => Color::from_rgb(0.96, 0.90, 0.94), // Light purple
            StatusInfo => Color::from_rgb(0.34, 0.71, 0.91),    // #56B4E9 - Sky blue
            StatusInfoLight => Color::from_rgb(0.90, 0.96, 0.99), // Very light blue

            // Mapping status
            MappingMapped => Color::from_rgb(0.00, 0.45, 0.70), // Blue (success)
            MappingUnmapped => Color::from_rgb(0.80, 0.47, 0.65), // Purple (error)
            MappingSuggested => Color::from_rgb(0.90, 0.62, 0.00), // Orange (warning)
            MappingNotCollected => Color::from_rgb(0.60, 0.60, 0.60), // Gray
            MappingInProgress => Color::from_rgb(0.90, 0.62, 0.00), // Orange

            // Backgrounds (unchanged - gray scale)
            BackgroundPrimary => Color::from_rgb(0.98, 0.98, 0.99),
            BackgroundSecondary => Color::from_rgb(0.95, 0.95, 0.97),
            BackgroundElevated => Color::from_rgb(1.0, 1.0, 1.0),
            BackgroundInset => Color::from_rgb(0.98, 0.98, 0.99),

            // Text (unchanged - gray scale)
            TextPrimary => Color::from_rgb(0.10, 0.10, 0.12),
            TextSecondary => Color::from_rgb(0.30, 0.30, 0.35),
            TextMuted => Color::from_rgb(0.50, 0.50, 0.55),
            TextDisabled => Color::from_rgb(0.65, 0.65, 0.70),
            TextOnAccent => Color::from_rgb(1.0, 1.0, 1.0),

            // Interactive - Blue accent
            AccentPrimary => Color::from_rgb(0.00, 0.45, 0.70), // Blue
            AccentHover => Color::from_rgb(0.00, 0.38, 0.60),
            AccentPressed => Color::from_rgb(0.00, 0.30, 0.50),
            AccentDisabled => Color::from_rgb(0.82, 0.82, 0.86),
            AccentPrimaryLight => Color::from_rgb(0.85, 0.93, 0.98),
            AccentPrimaryMedium => Color::from_rgb(0.70, 0.88, 0.95),

            // Danger - Purple (distinct from blue)
            DangerHover => Color::from_rgb(0.70, 0.40, 0.55),
            DangerPressed => Color::from_rgb(0.60, 0.32, 0.45),

            // Borders
            BorderDefault => Color::from_rgb(0.82, 0.82, 0.86),
            BorderSubtle => Color::from_rgb(0.90, 0.90, 0.93),
            BorderFocused => Color::from_rgb(0.00, 0.45, 0.70),
            BorderError => Color::from_rgb(0.80, 0.47, 0.65),

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
