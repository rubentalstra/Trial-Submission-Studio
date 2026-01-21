//! Pre-resolved color cache for the theme system.
//!
//! This module provides a struct with all semantic colors pre-resolved,
//! eliminating the need to pass `&ThemeConfig` through the call hierarchy.

use iced::Color;

use super::ThemeConfig;
use super::semantic::SemanticColor;

/// Pre-resolved colors for direct access.
///
/// All colors are resolved once when the theme changes, then accessed
/// via the thread-local `colors()` function. This eliminates:
/// - 176 functions with `&ThemeConfig` parameters
/// - 620 `config.resolve()` calls
///
/// # Example
///
/// ```rust,ignore
/// use crate::theme::colors;
///
/// let c = colors();
/// let background = c.background_primary;
/// let text = c.text_primary;
/// ```
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Fields are intentionally available for future use
pub struct ResolvedColors {
    // === Status Indicators (8) ===
    pub status_success: Color,
    pub status_success_light: Color,
    pub status_warning: Color,
    pub status_warning_light: Color,
    pub status_error: Color,
    pub status_error_light: Color,
    pub status_info: Color,
    pub status_info_light: Color,

    // === Mapping Status (5) ===
    pub mapping_mapped: Color,
    pub mapping_unmapped: Color,
    pub mapping_suggested: Color,
    pub mapping_not_collected: Color,
    pub mapping_in_progress: Color,

    // === Backgrounds (4) ===
    pub background_primary: Color,
    pub background_secondary: Color,
    pub background_elevated: Color,
    pub background_inset: Color,

    // === Text (5) ===
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_disabled: Color,
    pub text_on_accent: Color,

    // === Interactive/Accent (6) ===
    pub accent_primary: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub accent_disabled: Color,
    pub accent_primary_light: Color,
    pub accent_primary_medium: Color,

    // === Danger/Destructive (2) ===
    pub danger_hover: Color,
    pub danger_pressed: Color,

    // === Borders (4) ===
    pub border_default: Color,
    pub border_subtle: Color,
    pub border_focused: Color,
    pub border_error: Color,

    // === Special (6) ===
    pub white: Color,
    pub black: Color,
    pub transparent: Color,
    pub shadow: Color,
    pub shadow_strong: Color,
    pub backdrop: Color,
}

impl ResolvedColors {
    /// Create resolved colors from a theme configuration.
    ///
    /// This resolves all 40 semantic colors once, caching the results
    /// for efficient access throughout the UI.
    pub fn from_config(config: &ThemeConfig) -> Self {
        Self {
            // Status
            status_success: config.resolve(SemanticColor::StatusSuccess),
            status_success_light: config.resolve(SemanticColor::StatusSuccessLight),
            status_warning: config.resolve(SemanticColor::StatusWarning),
            status_warning_light: config.resolve(SemanticColor::StatusWarningLight),
            status_error: config.resolve(SemanticColor::StatusError),
            status_error_light: config.resolve(SemanticColor::StatusErrorLight),
            status_info: config.resolve(SemanticColor::StatusInfo),
            status_info_light: config.resolve(SemanticColor::StatusInfoLight),

            // Mapping
            mapping_mapped: config.resolve(SemanticColor::MappingMapped),
            mapping_unmapped: config.resolve(SemanticColor::MappingUnmapped),
            mapping_suggested: config.resolve(SemanticColor::MappingSuggested),
            mapping_not_collected: config.resolve(SemanticColor::MappingNotCollected),
            mapping_in_progress: config.resolve(SemanticColor::MappingInProgress),

            // Backgrounds
            background_primary: config.resolve(SemanticColor::BackgroundPrimary),
            background_secondary: config.resolve(SemanticColor::BackgroundSecondary),
            background_elevated: config.resolve(SemanticColor::BackgroundElevated),
            background_inset: config.resolve(SemanticColor::BackgroundInset),

            // Text
            text_primary: config.resolve(SemanticColor::TextPrimary),
            text_secondary: config.resolve(SemanticColor::TextSecondary),
            text_muted: config.resolve(SemanticColor::TextMuted),
            text_disabled: config.resolve(SemanticColor::TextDisabled),
            text_on_accent: config.resolve(SemanticColor::TextOnAccent),

            // Interactive
            accent_primary: config.resolve(SemanticColor::AccentPrimary),
            accent_hover: config.resolve(SemanticColor::AccentHover),
            accent_pressed: config.resolve(SemanticColor::AccentPressed),
            accent_disabled: config.resolve(SemanticColor::AccentDisabled),
            accent_primary_light: config.resolve(SemanticColor::AccentPrimaryLight),
            accent_primary_medium: config.resolve(SemanticColor::AccentPrimaryMedium),

            // Danger
            danger_hover: config.resolve(SemanticColor::DangerHover),
            danger_pressed: config.resolve(SemanticColor::DangerPressed),

            // Borders
            border_default: config.resolve(SemanticColor::BorderDefault),
            border_subtle: config.resolve(SemanticColor::BorderSubtle),
            border_focused: config.resolve(SemanticColor::BorderFocused),
            border_error: config.resolve(SemanticColor::BorderError),

            // Special
            white: config.resolve(SemanticColor::White),
            black: config.resolve(SemanticColor::Black),
            transparent: config.resolve(SemanticColor::Transparent),
            shadow: config.resolve(SemanticColor::Shadow),
            shadow_strong: config.resolve(SemanticColor::ShadowStrong),
            backdrop: config.resolve(SemanticColor::Backdrop),
        }
    }
}

impl Default for ResolvedColors {
    fn default() -> Self {
        Self::from_config(&ThemeConfig::default())
    }
}
