//! Deuteranopia dark palette - red-green (green-weak) optimized for dark mode.

use iced::Color;

use crate::theme::semantic::{Palette, SemanticColor};

/// Deuteranopia-optimized dark palette.
pub struct DeuteranopiaDark;

#[allow(clippy::enum_glob_use)]
impl Palette for DeuteranopiaDark {
    fn resolve(&self, color: SemanticColor) -> Color {
        use SemanticColor::*;
        match color {
            // Blue/Orange/Purple scheme - brighter for dark mode
            StatusSuccess => Color::from_rgb(0.40, 0.60, 0.90), // Brighter blue
            StatusSuccessLight => Color::from_rgb(0.10, 0.15, 0.25), // Dark blue
            StatusWarning => Color::from_rgb(1.0, 0.75, 0.20),  // Brighter orange
            StatusWarningLight => Color::from_rgb(0.25, 0.18, 0.08), // Dark orange
            StatusError => Color::from_rgb(0.90, 0.60, 0.75),   // Brighter purple
            StatusErrorLight => Color::from_rgb(0.22, 0.14, 0.20), // Dark purple
            StatusInfo => Color::from_rgb(0.50, 0.80, 0.95),    // Sky blue
            StatusInfoLight => Color::from_rgb(0.12, 0.20, 0.28),

            // Mapping status
            MappingMapped => Color::from_rgb(0.40, 0.60, 0.90),
            MappingUnmapped => Color::from_rgb(0.90, 0.60, 0.75),
            MappingSuggested => Color::from_rgb(1.0, 0.75, 0.20),
            MappingNotCollected => Color::from_rgb(0.50, 0.50, 0.55),
            MappingInProgress => Color::from_rgb(1.0, 0.75, 0.20),

            // Backgrounds
            BackgroundPrimary => Color::from_rgb(0.08, 0.08, 0.10),
            BackgroundSecondary => Color::from_rgb(0.12, 0.12, 0.14),
            BackgroundElevated => Color::from_rgb(0.16, 0.16, 0.18),
            BackgroundInset => Color::from_rgb(0.06, 0.06, 0.08),

            // Text
            TextPrimary => Color::from_rgb(0.95, 0.95, 0.97),
            TextSecondary => Color::from_rgb(0.80, 0.80, 0.85),
            TextMuted => Color::from_rgb(0.60, 0.60, 0.65),
            TextDisabled => Color::from_rgb(0.40, 0.40, 0.45),
            TextOnAccent => Color::from_rgb(1.0, 1.0, 1.0),

            // Interactive - Blue accent
            AccentPrimary => Color::from_rgb(0.40, 0.60, 0.90),
            AccentHover => Color::from_rgb(0.50, 0.70, 0.95),
            AccentPressed => Color::from_rgb(0.30, 0.50, 0.80),
            AccentDisabled => Color::from_rgb(0.30, 0.30, 0.35),
            AccentPrimaryLight => Color::from_rgb(0.10, 0.15, 0.25),
            AccentPrimaryMedium => Color::from_rgb(0.15, 0.22, 0.35),

            // Danger - Purple
            DangerHover => Color::from_rgb(0.95, 0.65, 0.80),
            DangerPressed => Color::from_rgb(0.80, 0.50, 0.65),

            // Borders
            BorderDefault => Color::from_rgb(0.25, 0.25, 0.28),
            BorderSubtle => Color::from_rgb(0.20, 0.20, 0.22),
            BorderFocused => Color::from_rgb(0.40, 0.60, 0.90),
            BorderError => Color::from_rgb(0.90, 0.60, 0.75),

            // Special
            White => Color::from_rgb(1.0, 1.0, 1.0),
            Black => Color::from_rgb(0.0, 0.0, 0.0),
            Transparent => Color::from_rgba(0.0, 0.0, 0.0, 0.0),
            Shadow => Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            ShadowStrong => Color::from_rgba(0.0, 0.0, 0.0, 0.40),
            Backdrop => Color::from_rgba(0.0, 0.0, 0.0, 0.7),
        }
    }
}
