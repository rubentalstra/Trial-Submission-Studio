//! Standard clinical palette - default colors for normal color vision.

use iced::Color;

use crate::theme::semantic::{Palette, SemanticColor};

/// Standard palette matching current clinical theme.
pub struct StandardLight;

#[allow(clippy::enum_glob_use)]
impl Palette for StandardLight {
    fn resolve(&self, color: SemanticColor) -> Color {
        use SemanticColor::*;
        match color {
            // Status
            StatusSuccess => Color::from_rgb(0.20, 0.70, 0.40), // #33B366
            StatusSuccessLight => Color::from_rgb(0.85, 0.95, 0.88), // #D9F2E0
            StatusWarning => Color::from_rgb(0.95, 0.65, 0.05), // #F2A60D
            StatusWarningLight => Color::from_rgb(0.99, 0.95, 0.85), // #FDF2D9
            StatusError => Color::from_rgb(0.85, 0.25, 0.25),   // #D94040
            StatusErrorLight => Color::from_rgb(0.98, 0.90, 0.90), // #FAE6E6
            StatusInfo => Color::from_rgb(0.25, 0.55, 0.85),    // #408CD9
            StatusInfoLight => Color::from_rgb(0.90, 0.95, 0.99), // #E6F2FC

            // Mapping status
            MappingMapped => Color::from_rgb(0.20, 0.70, 0.40),
            MappingUnmapped => Color::from_rgb(0.85, 0.25, 0.25),
            MappingSuggested => Color::from_rgb(0.95, 0.65, 0.05),
            MappingNotCollected => Color::from_rgb(0.65, 0.65, 0.70), // GRAY_400
            MappingInProgress => Color::from_rgb(0.95, 0.65, 0.05),

            // Backgrounds
            BackgroundPrimary => Color::from_rgb(0.98, 0.98, 0.99), // GRAY_50
            BackgroundSecondary => Color::from_rgb(0.95, 0.95, 0.97), // GRAY_100
            BackgroundElevated => Color::from_rgb(1.0, 1.0, 1.0),   // WHITE
            BackgroundInset => Color::from_rgb(0.98, 0.98, 0.99),   // GRAY_50

            // Text
            TextPrimary => Color::from_rgb(0.10, 0.10, 0.12), // GRAY_900
            TextSecondary => Color::from_rgb(0.30, 0.30, 0.35), // GRAY_700
            TextMuted => Color::from_rgb(0.50, 0.50, 0.55),   // GRAY_500
            TextDisabled => Color::from_rgb(0.65, 0.65, 0.70), // GRAY_400
            TextOnAccent => Color::from_rgb(1.0, 1.0, 1.0),   // WHITE

            // Interactive
            AccentPrimary => Color::from_rgb(0.00, 0.61, 0.65), // PRIMARY_500
            AccentHover => Color::from_rgb(0.00, 0.52, 0.56),   // PRIMARY_600
            AccentPressed => Color::from_rgb(0.00, 0.44, 0.47), // PRIMARY_700
            AccentDisabled => Color::from_rgb(0.82, 0.82, 0.86), // GRAY_300
            AccentPrimaryLight => Color::from_rgb(0.88, 0.97, 0.98), // PRIMARY_50
            AccentPrimaryMedium => Color::from_rgb(0.70, 0.92, 0.95), // PRIMARY_100

            // Danger
            DangerHover => Color::from_rgb(0.75, 0.20, 0.20),
            DangerPressed => Color::from_rgb(0.65, 0.15, 0.15),

            // Borders
            BorderDefault => Color::from_rgb(0.82, 0.82, 0.86), // GRAY_300
            BorderSubtle => Color::from_rgb(0.90, 0.90, 0.93),  // GRAY_200
            BorderFocused => Color::from_rgb(0.00, 0.61, 0.65), // PRIMARY_500
            BorderError => Color::from_rgb(0.85, 0.25, 0.25),   // ERROR

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
