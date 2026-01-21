//! Theme module for Trial Submission Studio.
//!
//! This module provides the Professional Clinical theme with:
//! - Pre-resolved color cache (`resolved`) - eliminates config parameter passing
//! - Thread-local context (`context`) - global color access via `colors()`
//! - Color palettes for different accessibility modes (`palette`)
//! - Spacing constants (`spacing`)
//! - Typography definitions (`typography`)
//! - Custom widget styles (`clinical`)
//!
//! # Architecture
//!
//! The theme system uses a thread-local context to store pre-resolved colors,
//! eliminating the need to pass `&ThemeConfig` through the call hierarchy.
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::theme::{set_theme, colors, spacing};
//!
//! // Initialize theme (call in App::new and on settings change)
//! set_theme(config);
//!
//! // Access colors anywhere - no parameters needed
//! let c = colors();
//! let background = c.background_primary;
//! let success = c.status_success;
//!
//! // Use spacing constants
//! let padding = spacing::SPACING_MD;
//! ```

pub mod clinical;
pub mod context;
pub mod palette;
pub mod resolved;
pub mod semantic;
pub mod spacing;
pub mod typography;

// Re-export thread-local context functions (main API)
pub use context::{colors, is_dark, set_theme};

// Re-export semantic color types (kept for palette implementations)
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
