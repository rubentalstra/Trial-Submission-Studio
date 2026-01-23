//! Clinical color extension trait for app-specific colors.
//!
//! Provides an extension trait `ClinicalColors` that adds clinical-specific
//! color methods to Iced's `Theme`. These are colors not covered by Iced's
//! built-in `ExtendedPalette`.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::theme::ClinicalColors;
//!
//! // In a style closure that receives &Theme:
//! .style(|theme: &Theme| {
//!     let clinical = theme.clinical();
//!     container::Style {
//!         background: Some(clinical.mapping_mapped.into()),
//!         ..Default::default()
//!     }
//! })
//! ```

use iced::{Color, Theme};

// =============================================================================
// CLINICAL COLOR SET
// =============================================================================

/// Clinical-specific colors not covered by Iced's ExtendedPalette.
///
/// These are app-specific semantic colors for:
/// - Mapping status indicators
/// - Danger button states
/// - Specialized borders and shadows
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Some fields reserved for future use
pub struct ClinicalColorSet {
    // === Mapping Status ===
    /// Mapped/accepted variable (typically success color)
    pub mapping_mapped: Color,
    /// Unmapped variable requiring action (typically danger color)
    pub mapping_unmapped: Color,
    /// Auto-suggested mapping (typically warning color)
    pub mapping_suggested: Color,
    /// Not collected - intentionally omitted (neutral gray)
    pub mapping_not_collected: Color,
    /// Mapping in progress (typically warning color)
    pub mapping_in_progress: Color,

    // === Danger Button States ===
    /// Danger button hover color
    pub danger_hover: Color,
    /// Danger button pressed color
    pub danger_pressed: Color,

    // === Accent Tints ===
    /// Light tint of primary accent (for hover backgrounds)
    pub accent_primary_light: Color,
    /// Medium tint of primary accent (for selections)
    pub accent_primary_medium: Color,

    // === Status Colors ===
    /// Info color (blue) - not in Iced's extended palette
    pub info: Color,
    /// Success status light background
    pub status_success_light: Color,
    /// Warning status light background
    pub status_warning_light: Color,
    /// Error status light background
    pub status_error_light: Color,
    /// Info status light background
    pub status_info_light: Color,

    // === Borders ===
    /// Default border color
    pub border_default: Color,
    /// Subtle/lighter border
    pub border_subtle: Color,
    /// Focused element border (typically accent color)
    pub border_focused: Color,
    /// Error border color
    pub border_error: Color,

    // === Backgrounds ===
    /// Secondary background (cards, surfaces)
    pub background_secondary: Color,
    /// Elevated surface (modals, dialogs) - white in light mode
    pub background_elevated: Color,
    /// Inset/recessed areas
    pub background_inset: Color,

    // === Text ===
    /// Secondary text color
    pub text_secondary: Color,
    /// Muted text (descriptions, hints)
    pub text_muted: Color,
    /// Disabled text
    pub text_disabled: Color,
    /// Text on accent color backgrounds
    pub text_on_accent: Color,

    // === Interactive ===
    /// Accent hover color
    pub accent_hover: Color,
    /// Accent pressed color
    pub accent_pressed: Color,
    /// Accent disabled color
    pub accent_disabled: Color,

    // === Special ===
    /// Shadow color for elevation
    pub shadow: Color,
    /// Strong shadow for higher elevation
    pub shadow_strong: Color,
    /// Modal backdrop overlay
    pub backdrop: Color,
}

// =============================================================================
// EXTENSION TRAIT
// =============================================================================

/// Extension trait for clinical-specific colors.
///
/// This trait provides access to app-specific colors that aren't part of
/// Iced's ExtendedPalette. Use it inside style closures that receive a `&Theme`.
pub trait ClinicalColors {
    /// Get the clinical color set for this theme.
    fn clinical(&self) -> ClinicalColorSet;
}

impl ClinicalColors for Theme {
    fn clinical(&self) -> ClinicalColorSet {
        let palette = self.extended_palette();
        let is_dark = palette.is_dark;

        // Map clinical concepts to the extended palette where possible,
        // otherwise provide calculated colors based on the palette

        ClinicalColorSet {
            // Mapping status uses semantic palette colors
            mapping_mapped: palette.success.base.color,
            mapping_unmapped: palette.danger.base.color,
            mapping_suggested: palette.warning.base.color,
            mapping_not_collected: if is_dark {
                Color::from_rgb(0.50, 0.50, 0.55)
            } else {
                Color::from_rgb(0.65, 0.65, 0.70)
            },
            mapping_in_progress: palette.warning.base.color,

            // Danger button states - derived from danger color
            danger_hover: if is_dark {
                // Lighter in dark mode
                blend_color(palette.danger.base.color, Color::WHITE, 0.15)
            } else {
                // Darker in light mode
                blend_color(palette.danger.base.color, Color::BLACK, 0.12)
            },
            danger_pressed: if is_dark {
                blend_color(palette.danger.base.color, Color::BLACK, 0.15)
            } else {
                blend_color(palette.danger.base.color, Color::BLACK, 0.25)
            },

            // Accent tints
            accent_primary_light: if is_dark {
                Color::from_rgba(
                    palette.primary.base.color.r,
                    palette.primary.base.color.g,
                    palette.primary.base.color.b,
                    0.15,
                )
            } else {
                blend_color(palette.primary.base.color, Color::WHITE, 0.88)
            },
            accent_primary_medium: if is_dark {
                Color::from_rgba(
                    palette.primary.base.color.r,
                    palette.primary.base.color.g,
                    palette.primary.base.color.b,
                    0.25,
                )
            } else {
                blend_color(palette.primary.base.color, Color::WHITE, 0.70)
            },

            // Info color (not in Iced's extended palette)
            info: Color::from_rgb(0.25, 0.55, 0.85),

            // Status light backgrounds
            status_success_light: if is_dark {
                Color::from_rgba(
                    palette.success.base.color.r,
                    palette.success.base.color.g,
                    palette.success.base.color.b,
                    0.15,
                )
            } else {
                blend_color(palette.success.base.color, Color::WHITE, 0.85)
            },
            status_warning_light: if is_dark {
                Color::from_rgba(
                    palette.warning.base.color.r,
                    palette.warning.base.color.g,
                    palette.warning.base.color.b,
                    0.15,
                )
            } else {
                blend_color(palette.warning.base.color, Color::WHITE, 0.85)
            },
            status_error_light: if is_dark {
                Color::from_rgba(
                    palette.danger.base.color.r,
                    palette.danger.base.color.g,
                    palette.danger.base.color.b,
                    0.15,
                )
            } else {
                blend_color(palette.danger.base.color, Color::WHITE, 0.85)
            },
            status_info_light: if is_dark {
                Color::from_rgba(0.25, 0.55, 0.85, 0.15)
            } else {
                Color::from_rgb(0.90, 0.95, 0.99)
            },

            // Borders
            border_default: palette.background.strong.color,
            border_subtle: if is_dark {
                Color::from_rgb(0.20, 0.20, 0.22)
            } else {
                Color::from_rgb(0.90, 0.90, 0.93)
            },
            border_focused: palette.primary.base.color,
            border_error: palette.danger.base.color,

            // Backgrounds
            background_secondary: palette.background.weak.color,
            background_elevated: if is_dark {
                Color::from_rgb(0.16, 0.16, 0.18)
            } else {
                Color::WHITE
            },
            background_inset: if is_dark {
                Color::from_rgb(0.06, 0.06, 0.08)
            } else {
                Color::from_rgb(0.98, 0.98, 0.99)
            },

            // Text
            text_secondary: if is_dark {
                Color::from_rgb(0.80, 0.80, 0.85)
            } else {
                Color::from_rgb(0.30, 0.30, 0.35)
            },
            text_muted: if is_dark {
                Color::from_rgb(0.60, 0.60, 0.65)
            } else {
                Color::from_rgb(0.50, 0.50, 0.55)
            },
            text_disabled: if is_dark {
                Color::from_rgb(0.40, 0.40, 0.45)
            } else {
                Color::from_rgb(0.65, 0.65, 0.70)
            },
            text_on_accent: Color::WHITE,

            // Interactive accent states
            accent_hover: palette.primary.strong.color,
            accent_pressed: if is_dark {
                blend_color(palette.primary.base.color, Color::BLACK, 0.20)
            } else {
                blend_color(palette.primary.base.color, Color::BLACK, 0.15)
            },
            accent_disabled: if is_dark {
                Color::from_rgb(0.30, 0.30, 0.35)
            } else {
                Color::from_rgb(0.82, 0.82, 0.86)
            },

            // Shadows and overlays
            shadow: Color::from_rgba(0.0, 0.0, 0.0, if is_dark { 0.25 } else { 0.08 }),
            shadow_strong: Color::from_rgba(0.0, 0.0, 0.0, if is_dark { 0.40 } else { 0.16 }),
            backdrop: Color::from_rgba(0.0, 0.0, 0.0, if is_dark { 0.70 } else { 0.50 }),
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Blend two colors together.
///
/// `factor` of 0.0 returns `base`, 1.0 returns `blend`.
fn blend_color(base: Color, blend: Color, factor: f32) -> Color {
    Color::from_rgb(
        base.r + (blend.r - base.r) * factor,
        base.g + (blend.g - base.g) * factor,
        base.b + (blend.b - base.b) * factor,
    )
}
