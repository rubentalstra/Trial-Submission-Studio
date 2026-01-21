//! Tritanopia dark palette - blue-yellow blind optimized for dark mode.

use iced::Color;

use crate::theme::semantic::{Palette, SemanticColor};

/// Tritanopia-optimized dark palette.
pub struct TritanopiaDark;

#[allow(clippy::enum_glob_use)]
impl Palette for TritanopiaDark {
    fn resolve(&self, color: SemanticColor) -> Color {
        use SemanticColor::*;
        match color {
            // Green/Red-Orange/Magenta scheme (avoids blue-yellow)
            StatusSuccess => Color::from_rgb(0.35, 0.85, 0.55), // Bright green
            StatusSuccessLight => Color::from_rgb(0.10, 0.22, 0.14),
            StatusWarning => Color::from_rgb(1.0, 0.55, 0.30), // Red-orange
            StatusWarningLight => Color::from_rgb(0.28, 0.15, 0.10),
            StatusError => Color::from_rgb(0.95, 0.40, 0.55), // Magenta-red
            StatusErrorLight => Color::from_rgb(0.25, 0.12, 0.15),
            StatusInfo => Color::from_rgb(0.70, 0.70, 0.70), // Neutral gray
            StatusInfoLight => Color::from_rgb(0.18, 0.18, 0.18),

            // Mapping status
            MappingMapped => Color::from_rgb(0.35, 0.85, 0.55),
            MappingUnmapped => Color::from_rgb(0.95, 0.40, 0.55),
            MappingSuggested => Color::from_rgb(1.0, 0.55, 0.30),
            MappingNotCollected => Color::from_rgb(0.50, 0.50, 0.55),
            MappingInProgress => Color::from_rgb(1.0, 0.55, 0.30),

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

            // Interactive - Purple accent
            AccentPrimary => Color::from_rgb(0.70, 0.35, 0.60),
            AccentHover => Color::from_rgb(0.80, 0.45, 0.70),
            AccentPressed => Color::from_rgb(0.60, 0.25, 0.50),
            AccentDisabled => Color::from_rgb(0.30, 0.30, 0.35),
            AccentPrimaryLight => Color::from_rgb(0.18, 0.10, 0.15),
            AccentPrimaryMedium => Color::from_rgb(0.25, 0.15, 0.22),

            // Danger - Red-magenta
            DangerHover => Color::from_rgb(1.0, 0.50, 0.62),
            DangerPressed => Color::from_rgb(0.85, 0.35, 0.48),

            // Borders
            BorderDefault => Color::from_rgb(0.25, 0.25, 0.28),
            BorderSubtle => Color::from_rgb(0.20, 0.20, 0.22),
            BorderFocused => Color::from_rgb(0.70, 0.35, 0.60),
            BorderError => Color::from_rgb(0.95, 0.40, 0.55),

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
