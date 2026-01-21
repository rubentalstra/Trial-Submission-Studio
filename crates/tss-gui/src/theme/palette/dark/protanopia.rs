//! Protanopia dark palette - red-blind optimized for dark mode.

use iced::Color;

use crate::theme::semantic::{Palette, SemanticColor};

/// Protanopia-optimized dark palette.
pub struct ProtanopiaDark;

#[allow(clippy::enum_glob_use)]
impl Palette for ProtanopiaDark {
    fn resolve(&self, color: SemanticColor) -> Color {
        use SemanticColor::*;
        match color {
            // Teal/Yellow/Magenta scheme
            StatusSuccess => Color::from_rgb(0.20, 0.80, 0.65), // Bright teal
            StatusSuccessLight => Color::from_rgb(0.08, 0.20, 0.16),
            StatusWarning => Color::from_rgb(1.0, 0.95, 0.40), // Bright yellow
            StatusWarningLight => Color::from_rgb(0.25, 0.22, 0.10),
            StatusError => Color::from_rgb(0.95, 0.35, 0.65), // Bright magenta
            StatusErrorLight => Color::from_rgb(0.25, 0.10, 0.18),
            StatusInfo => Color::from_rgb(0.50, 0.80, 0.95),
            StatusInfoLight => Color::from_rgb(0.12, 0.20, 0.28),

            // Mapping status
            MappingMapped => Color::from_rgb(0.20, 0.80, 0.65),
            MappingUnmapped => Color::from_rgb(0.95, 0.35, 0.65),
            MappingSuggested => Color::from_rgb(1.0, 0.95, 0.40),
            MappingNotCollected => Color::from_rgb(0.50, 0.50, 0.55),
            MappingInProgress => Color::from_rgb(1.0, 0.95, 0.40),

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

            // Interactive - Teal accent
            AccentPrimary => Color::from_rgb(0.20, 0.80, 0.65),
            AccentHover => Color::from_rgb(0.30, 0.90, 0.75),
            AccentPressed => Color::from_rgb(0.15, 0.70, 0.55),
            AccentDisabled => Color::from_rgb(0.30, 0.30, 0.35),
            AccentPrimaryLight => Color::from_rgb(0.08, 0.20, 0.16),
            AccentPrimaryMedium => Color::from_rgb(0.12, 0.30, 0.24),

            // Danger - Magenta
            DangerHover => Color::from_rgb(1.0, 0.45, 0.72),
            DangerPressed => Color::from_rgb(0.85, 0.28, 0.55),

            // Borders
            BorderDefault => Color::from_rgb(0.25, 0.25, 0.28),
            BorderSubtle => Color::from_rgb(0.20, 0.20, 0.22),
            BorderFocused => Color::from_rgb(0.20, 0.80, 0.65),
            BorderError => Color::from_rgb(0.95, 0.35, 0.65),

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
