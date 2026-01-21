//! Standard dark palette - default colors for normal color vision in dark mode.
//!
//! Uses inverted backgrounds with adjusted status colors for dark mode visibility.

use iced::Color;

use crate::theme::semantic::{Palette, SemanticColor};

/// Standard dark palette.
pub struct StandardDark;

#[allow(clippy::enum_glob_use)]
impl Palette for StandardDark {
    fn resolve(&self, color: SemanticColor) -> Color {
        use SemanticColor::*;
        match color {
            // Status colors - slightly brighter for dark backgrounds
            StatusSuccess => Color::from_rgb(0.35, 0.80, 0.55), // Brighter green
            StatusSuccessLight => Color::from_rgb(0.12, 0.22, 0.16), // Dark green
            StatusWarning => Color::from_rgb(1.0, 0.75, 0.20),  // Brighter yellow
            StatusWarningLight => Color::from_rgb(0.22, 0.18, 0.10), // Dark amber
            StatusError => Color::from_rgb(0.95, 0.40, 0.40),   // Brighter red
            StatusErrorLight => Color::from_rgb(0.25, 0.12, 0.12), // Dark red
            StatusInfo => Color::from_rgb(0.45, 0.70, 0.95),    // Brighter blue
            StatusInfoLight => Color::from_rgb(0.12, 0.18, 0.25), // Dark blue

            // Mapping status
            MappingMapped => Color::from_rgb(0.35, 0.80, 0.55),
            MappingUnmapped => Color::from_rgb(0.95, 0.40, 0.40),
            MappingSuggested => Color::from_rgb(1.0, 0.75, 0.20),
            MappingNotCollected => Color::from_rgb(0.50, 0.50, 0.55),
            MappingInProgress => Color::from_rgb(1.0, 0.75, 0.20),

            // Dark backgrounds
            BackgroundPrimary => Color::from_rgb(0.08, 0.08, 0.10), // Near black
            BackgroundSecondary => Color::from_rgb(0.12, 0.12, 0.14), // Slightly lighter
            BackgroundElevated => Color::from_rgb(0.16, 0.16, 0.18), // Card/modal bg
            BackgroundInset => Color::from_rgb(0.06, 0.06, 0.08),   // Recessed

            // Inverted text
            TextPrimary => Color::from_rgb(0.95, 0.95, 0.97), // Near white
            TextSecondary => Color::from_rgb(0.80, 0.80, 0.85),
            TextMuted => Color::from_rgb(0.60, 0.60, 0.65),
            TextDisabled => Color::from_rgb(0.40, 0.40, 0.45),
            TextOnAccent => Color::from_rgb(1.0, 1.0, 1.0),

            // Accent colors - clinical teal works well on dark
            AccentPrimary => Color::from_rgb(0.15, 0.75, 0.80), // Brighter teal
            AccentHover => Color::from_rgb(0.20, 0.85, 0.90),
            AccentPressed => Color::from_rgb(0.10, 0.65, 0.70),
            AccentDisabled => Color::from_rgb(0.30, 0.30, 0.35),
            AccentPrimaryLight => Color::from_rgb(0.10, 0.20, 0.22),
            AccentPrimaryMedium => Color::from_rgb(0.12, 0.28, 0.30),

            // Danger
            DangerHover => Color::from_rgb(1.0, 0.50, 0.50),
            DangerPressed => Color::from_rgb(0.85, 0.35, 0.35),

            // Borders - visible on dark
            BorderDefault => Color::from_rgb(0.25, 0.25, 0.28),
            BorderSubtle => Color::from_rgb(0.20, 0.20, 0.22),
            BorderFocused => Color::from_rgb(0.15, 0.75, 0.80),
            BorderError => Color::from_rgb(0.95, 0.40, 0.40),

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
