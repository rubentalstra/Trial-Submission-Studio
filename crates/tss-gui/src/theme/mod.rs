//! Theme module for Trial Submission Studio.
//!
//! This module provides the Professional Clinical theme with:
//! - Semantic color system with accessibility mode support (`semantic`)
//! - Color palettes for different accessibility modes (`palette`)
//! - Spacing constants (`spacing`)
//! - Typography definitions (`typography`)
//! - Custom widget styles (`clinical`)
//!
//! # Architecture
//!
//! The theme system uses a semantic color approach where colors are defined
//! by their purpose (e.g., `StatusSuccess`) rather than their value. This
//! enables runtime switching between accessibility modes without code changes.
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::theme::{ThemeConfig, SemanticColor, spacing, typography};
//!
//! // Create theme configuration
//! let config = ThemeConfig::default();
//!
//! // Resolve semantic colors
//! let success_color = config.resolve(SemanticColor::StatusSuccess);
//!
//! // Use spacing constants
//! let padding = spacing::SPACING_MD;
//! ```

pub mod clinical;
pub mod palette;
pub mod semantic;
pub mod spacing;
pub mod typography;

// Re-export semantic color types
pub use semantic::{Palette as PaletteTrait, SemanticColor};

// Re-export palette mode enums
pub use palette::{AccessibilityMode, ThemeMode};

// Re-export palette implementations
pub use palette::dark::{DeuteranopiaDark, ProtanopiaDark, StandardDark, TritanopiaDark};
pub use palette::light::{DeuteranopiaLight, ProtanopiaLight, StandardLight, TritanopiaLight};

// Re-export theme creation function
pub use clinical::clinical_theme;

// Re-export spacing constants (only those currently used)
pub use spacing::{
    BORDER_RADIUS_FULL, BORDER_RADIUS_LG, BORDER_RADIUS_MD, BORDER_RADIUS_SM, MASTER_WIDTH,
    MODAL_WIDTH_MD, MODAL_WIDTH_SM, SIDEBAR_WIDTH, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL,
    SPACING_XS, TAB_PADDING_X, TAB_PADDING_Y, TABLE_CELL_PADDING_X, TABLE_CELL_PADDING_Y,
};

// Re-export typography constants (only those currently used)
pub use typography::{MAX_CHARS_SHORT_LABEL, MAX_CHARS_VARIABLE_NAME};

// Re-export widget style functions (only those currently used)
pub use clinical::{
    button_ghost, button_primary, button_secondary, progress_bar_primary, text_input_default,
};

use iced::Color;
use serde::{Deserialize, Serialize};

// =============================================================================
// THEME CONFIGURATION
// =============================================================================

/// Theme configuration for the application.
///
/// Combines appearance mode (light/dark) with accessibility mode.
/// Changes apply immediately without app restart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Appearance mode (light/dark/system)
    pub theme_mode: ThemeMode,
    /// Color vision accessibility mode
    pub accessibility_mode: AccessibilityMode,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Light,
            accessibility_mode: AccessibilityMode::Standard,
        }
    }
}

impl ThemeConfig {
    /// Create with both theme mode and accessibility mode.
    pub fn new(theme_mode: ThemeMode, accessibility_mode: AccessibilityMode) -> Self {
        Self {
            theme_mode,
            accessibility_mode,
        }
    }

    /// Get the effective theme mode (resolves System to Light/Dark).
    pub fn effective_mode(&self) -> ThemeMode {
        self.theme_mode.resolve()
    }

    /// Check if currently in dark mode.
    pub fn is_dark(&self) -> bool {
        matches!(self.effective_mode(), ThemeMode::Dark)
    }

    /// Resolve a semantic color using the current palette combination.
    pub fn resolve(&self, color: SemanticColor) -> Color {
        match (self.effective_mode(), self.accessibility_mode) {
            // Light mode palettes
            (ThemeMode::Light, AccessibilityMode::Standard) => StandardLight.resolve(color),
            (ThemeMode::Light, AccessibilityMode::Deuteranopia) => DeuteranopiaLight.resolve(color),
            (ThemeMode::Light, AccessibilityMode::Protanopia) => ProtanopiaLight.resolve(color),
            (ThemeMode::Light, AccessibilityMode::Tritanopia) => TritanopiaLight.resolve(color),
            // Dark mode palettes
            (ThemeMode::Dark, AccessibilityMode::Standard) => StandardDark.resolve(color),
            (ThemeMode::Dark, AccessibilityMode::Deuteranopia) => DeuteranopiaDark.resolve(color),
            (ThemeMode::Dark, AccessibilityMode::Protanopia) => ProtanopiaDark.resolve(color),
            (ThemeMode::Dark, AccessibilityMode::Tritanopia) => TritanopiaDark.resolve(color),
            // System resolves to Light or Dark via effective_mode()
            (ThemeMode::System, _) => {
                // This branch should never be reached because effective_mode()
                // always resolves System to Light or Dark
                StandardLight.resolve(color)
            }
        }
    }
}

// =============================================================================
// LEGACY COLOR CONSTANTS (for backwards compatibility during migration)
// =============================================================================
// These will be removed after all files are migrated to use ThemeConfig.resolve()
// TODO: Remove these once migration is complete

use iced::color;

// Primary colors - allow dead_code during migration
#[allow(dead_code)]
pub const PRIMARY_50: Color = color!(0xE0F7FA);
pub const PRIMARY_100: Color = color!(0xB3EBF2);
#[allow(dead_code)]
pub const PRIMARY_200: Color = color!(0x80D9E6);
#[allow(dead_code)]
pub const PRIMARY_300: Color = color!(0x4DC7D1);
#[allow(dead_code)]
pub const PRIMARY_400: Color = color!(0x26B3BF);
pub const PRIMARY_500: Color = color!(0x009BA6);
pub const PRIMARY_600: Color = color!(0x00858E);
pub const PRIMARY_700: Color = color!(0x007078);
#[allow(dead_code)]
pub const PRIMARY_800: Color = color!(0x005A61);
#[allow(dead_code)]
pub const PRIMARY_900: Color = color!(0x00454A);
#[allow(dead_code)]
pub const PRIMARY: Color = PRIMARY_500;

// Semantic colors
pub const SUCCESS: Color = color!(0x33B366);
#[allow(dead_code)]
pub const SUCCESS_LIGHT: Color = color!(0xD9F2E0);
pub const WARNING: Color = color!(0xF2A60D);
#[allow(dead_code)]
pub const WARNING_LIGHT: Color = color!(0xFDF2D9);
pub const ERROR: Color = color!(0xD94040);
#[allow(dead_code)]
pub const ERROR_LIGHT: Color = color!(0xFAE6E6);
#[allow(dead_code)]
pub const INFO: Color = color!(0x408CD9);
#[allow(dead_code)]
pub const INFO_LIGHT: Color = color!(0xE6F2FC);

// Grays
#[allow(dead_code)]
pub const GRAY_50: Color = color!(0xFAFAFE);
pub const GRAY_100: Color = color!(0xF2F2F7);
pub const GRAY_200: Color = color!(0xE6E6ED);
#[allow(dead_code)]
pub const GRAY_300: Color = color!(0xD1D1DB);
pub const GRAY_400: Color = color!(0xA6A6B3);
pub const GRAY_500: Color = color!(0x80808C);
pub const GRAY_600: Color = color!(0x666673);
pub const GRAY_700: Color = color!(0x4D4D59);
pub const GRAY_800: Color = color!(0x33333D);
pub const GRAY_900: Color = color!(0x1A1A1F);

// Special colors
pub const WHITE: Color = Color::WHITE;
#[allow(dead_code)]
pub const BLACK: Color = Color::BLACK;
#[allow(dead_code)]
pub const TRANSPARENT: Color = Color::TRANSPARENT;
pub const BACKDROP: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.5,
};
#[allow(dead_code)]
pub const SHADOW: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.08,
};
#[allow(dead_code)]
pub const SHADOW_STRONG: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.16,
};

// Mapping status colors
#[allow(dead_code)]
pub const STATUS_MAPPED: Color = SUCCESS;
#[allow(dead_code)]
pub const STATUS_NOT_MAPPED: Color = ERROR;
#[allow(dead_code)]
pub const STATUS_UNMAPPED: Color = GRAY_400;
#[allow(dead_code)]
pub const STATUS_IN_PROGRESS: Color = WARNING;

// Export status colors
#[allow(dead_code)]
pub const EXPORT_READY: Color = SUCCESS;
#[allow(dead_code)]
pub const EXPORT_INCOMPLETE: Color = WARNING;
#[allow(dead_code)]
pub const EXPORT_HAS_ERRORS: Color = ERROR;
