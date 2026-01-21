//! Tritanopia light palette - optimized for blue-yellow color blindness.
//!
//! Avoids blue-yellow confusion by using red-green-magenta spectrum.

use iced::Color;

use crate::theme::semantic::{Palette, SemanticColor};

/// Tritanopia-optimized light palette.
pub struct TritanopiaLight;

#[allow(clippy::enum_glob_use)]
impl Palette for TritanopiaLight {
    fn resolve(&self, color: SemanticColor) -> Color {
        use SemanticColor::*;
        match color {
            // Status - Red/Green/Magenta scheme (avoids blue-yellow)
            StatusSuccess => Color::from_rgb(0.20, 0.70, 0.40),
            StatusSuccessLight => Color::from_rgb(0.85, 0.95, 0.88),
            StatusWarning => Color::from_rgb(0.90, 0.38, 0.13),
            StatusWarningLight => Color::from_rgb(0.98, 0.90, 0.86),
            StatusError => Color::from_rgb(0.80, 0.20, 0.35),
            StatusErrorLight => Color::from_rgb(0.96, 0.88, 0.90),
            StatusInfo => Color::from_rgb(0.60, 0.60, 0.60),
            StatusInfoLight => Color::from_rgb(0.94, 0.94, 0.94),

            // Mapping status
            MappingMapped => Color::from_rgb(0.20, 0.70, 0.40),
            MappingUnmapped => Color::from_rgb(0.80, 0.20, 0.35),
            MappingSuggested => Color::from_rgb(0.90, 0.38, 0.13),
            MappingNotCollected => Color::from_rgb(0.60, 0.60, 0.60),
            MappingInProgress => Color::from_rgb(0.90, 0.38, 0.13),

            // Backgrounds
            BackgroundPrimary => Color::from_rgb(0.98, 0.98, 0.99),
            BackgroundSecondary => Color::from_rgb(0.95, 0.95, 0.97),
            BackgroundElevated => Color::from_rgb(1.0, 1.0, 1.0),
            BackgroundInset => Color::from_rgb(0.98, 0.98, 0.99),

            // Text
            TextPrimary => Color::from_rgb(0.10, 0.10, 0.12),
            TextSecondary => Color::from_rgb(0.30, 0.30, 0.35),
            TextMuted => Color::from_rgb(0.50, 0.50, 0.55),
            TextDisabled => Color::from_rgb(0.65, 0.65, 0.70),
            TextOnAccent => Color::from_rgb(1.0, 1.0, 1.0),

            // Interactive - Purple accent
            AccentPrimary => Color::from_rgb(0.60, 0.20, 0.50),
            AccentHover => Color::from_rgb(0.50, 0.17, 0.42),
            AccentPressed => Color::from_rgb(0.40, 0.13, 0.33),
            AccentDisabled => Color::from_rgb(0.82, 0.82, 0.86),
            AccentPrimaryLight => Color::from_rgb(0.94, 0.88, 0.92),
            AccentPrimaryMedium => Color::from_rgb(0.88, 0.78, 0.85),

            // Danger - Dark red/magenta
            DangerHover => Color::from_rgb(0.70, 0.15, 0.28),
            DangerPressed => Color::from_rgb(0.60, 0.10, 0.22),

            // Borders
            BorderDefault => Color::from_rgb(0.82, 0.82, 0.86),
            BorderSubtle => Color::from_rgb(0.90, 0.90, 0.93),
            BorderFocused => Color::from_rgb(0.60, 0.20, 0.50),
            BorderError => Color::from_rgb(0.80, 0.20, 0.35),

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
