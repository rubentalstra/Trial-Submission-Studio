//! Theme module for Trial Submission Studio.
//!
//! This module provides the Professional Clinical theme using idiomatic Iced 0.14 patterns:
//!
//! - **palette**: Clinical color palettes for all theme/accessibility combinations
//! - **colors**: ClinicalColors extension trait for app-specific colors
//! - **clinical**: Widget style functions that use the theme parameter
//! - **spacing**: Layout spacing constants
//! - **typography**: Text size constants
//!
//! # Architecture
//!
//! The theme system follows Iced's design patterns:
//! - Style functions receive `&Theme` and use it to access colors
//! - `theme.extended_palette()` provides standard widget colors
//! - `theme.clinical()` (via ClinicalColors trait) provides app-specific colors
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::theme::{clinical_theme, ClinicalColors, spacing};
//! use iced::Theme;
//!
//! // Create the theme (typically in App::theme())
//! let theme = clinical_theme(theme_mode, accessibility_mode, system_is_dark);
//!
//! // In style closures, use the theme parameter:
//! .style(|theme: &Theme| {
//!     let palette = theme.extended_palette();
//!     let clinical = theme.clinical();
//!
//!     container::Style {
//!         background: Some(palette.background.base.color.into()),
//!         border: Border {
//!             color: clinical.border_default,
//!             ..Default::default()
//!         },
//!         ..Default::default()
//!     }
//! })
//! ```

pub mod clinical;
pub mod colors;
pub mod palette;
pub mod spacing;
pub mod typography;

// Re-export ClinicalColors extension trait (main API for app-specific colors)
pub use colors::ClinicalColors;

// Re-export palette types
pub use palette::{AccessibilityMode, ThemeMode};

// Re-export theme creation function
pub use clinical::clinical_theme;

// Re-export spacing constants (only those currently used)
pub use spacing::{
    ALPHA_LIGHT, BORDER_RADIUS_FULL, BORDER_RADIUS_LG, BORDER_RADIUS_MD, BORDER_RADIUS_SM,
    MASTER_WIDTH, MODAL_WIDTH_MD, MODAL_WIDTH_SM, SIDEBAR_WIDTH, SPACING_LG, SPACING_MD,
    SPACING_SM, SPACING_XL, SPACING_XS, TAB_PADDING_X, TAB_PADDING_Y, TABLE_CELL_PADDING_X,
    TABLE_CELL_PADDING_Y,
};

// Re-export typography constants (only those currently used)
pub use typography::{MAX_CHARS_SHORT_LABEL, MAX_CHARS_VARIABLE_NAME};

// Re-export widget style functions (only those currently used)
pub use clinical::{
    button_ghost, button_primary, button_secondary, progress_bar_primary, text_input_default,
};

use iced::Theme;
use serde::{Deserialize, Serialize};

// =============================================================================
// THEME CONFIGURATION
// =============================================================================

/// Theme configuration for the application.
///
/// Combines appearance mode (light/dark/system) with accessibility mode.
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

    /// Create an Iced Theme from this configuration.
    ///
    /// This is useful for previewing themes (e.g., in settings dialogs)
    /// where you need to show colors for a different configuration than
    /// the currently active theme.
    ///
    /// # Arguments
    ///
    /// * `system_is_dark` - Whether the system is in dark mode (for System theme mode)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = ThemeConfig::new(ThemeMode::Dark, AccessibilityMode::Standard);
    /// let preview_theme = config.to_theme(false);
    /// let colors = preview_theme.clinical();
    /// ```
    pub fn to_theme(self, system_is_dark: bool) -> Theme {
        clinical_theme(self.theme_mode, self.accessibility_mode, system_is_dark)
    }
}
