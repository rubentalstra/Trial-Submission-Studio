//! Clinical color palettes for Trial Submission Studio.
//!
//! Provides 8 palettes (light/dark x 4 accessibility modes) that integrate
//! with Iced's theme system via the `Palette` type.
//!
//! These palettes are designed following:
//! - IBM Color Blind Safe palette guidelines
//! - Wong (Nature Methods, 2011) color-blind friendly colors
//! - WCAG 2.1 contrast requirements

use iced::Color;
use iced::theme::Palette;
use serde::{Deserialize, Serialize};

// =============================================================================
// ACCESSIBILITY MODE
// =============================================================================

/// Color vision accessibility modes.
///
/// Based on the three most common types of color vision deficiency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessibilityMode {
    /// Standard color vision (default clinical theme)
    #[default]
    Standard,
    /// Deuteranopia - red-green color blindness (green-weak)
    /// Most common form (~6% of males)
    Deuteranopia,
    /// Protanopia - red-green color blindness (red-blind)
    /// (~1% of males)
    Protanopia,
    /// Tritanopia - blue-yellow color blindness
    /// Rare (~0.01% of population)
    Tritanopia,
}

impl AccessibilityMode {
    /// Human-readable label for UI display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Deuteranopia => "Deuteranopia (Red-Green)",
            Self::Protanopia => "Protanopia (Red-Blind)",
            Self::Tritanopia => "Tritanopia (Blue-Yellow)",
        }
    }

    /// All available modes for UI picker.
    pub const ALL: [Self; 4] = [
        Self::Standard,
        Self::Deuteranopia,
        Self::Protanopia,
        Self::Tritanopia,
    ];
}

impl std::fmt::Display for AccessibilityMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// =============================================================================
// THEME MODE
// =============================================================================

/// Theme mode for light/dark appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
    System,
}

impl ThemeMode {
    /// Human-readable label for UI display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
            Self::System => "System",
        }
    }

    /// All available modes for UI picker.
    pub const ALL: [Self; 3] = [Self::Light, Self::Dark, Self::System];

    /// Check if this is a dark mode (or resolves to dark).
    pub fn is_dark(&self, system_is_dark: bool) -> bool {
        match self {
            Self::Light => false,
            Self::Dark => true,
            Self::System => system_is_dark,
        }
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// =============================================================================
// PALETTE CREATION
// =============================================================================

/// Create the Iced Palette for the given theme and accessibility configuration.
///
/// This returns a `Palette` that Iced uses to generate its `ExtendedPalette`
/// which provides all the color variations for widgets.
pub fn clinical_palette(
    theme_mode: ThemeMode,
    accessibility_mode: AccessibilityMode,
    system_is_dark: bool,
) -> Palette {
    let is_dark = theme_mode.is_dark(system_is_dark);

    match (is_dark, accessibility_mode) {
        // Light mode palettes
        (false, AccessibilityMode::Standard) => light_standard(),
        (false, AccessibilityMode::Deuteranopia) => light_deuteranopia(),
        (false, AccessibilityMode::Protanopia) => light_protanopia(),
        (false, AccessibilityMode::Tritanopia) => light_tritanopia(),
        // Dark mode palettes
        (true, AccessibilityMode::Standard) => dark_standard(),
        (true, AccessibilityMode::Deuteranopia) => dark_deuteranopia(),
        (true, AccessibilityMode::Protanopia) => dark_protanopia(),
        (true, AccessibilityMode::Tritanopia) => dark_tritanopia(),
    }
}

// =============================================================================
// LIGHT MODE PALETTES
// =============================================================================

/// Standard clinical light palette - default for normal color vision.
fn light_standard() -> Palette {
    Palette {
        background: Color::from_rgb(0.98, 0.98, 0.99), // GRAY_50
        text: Color::from_rgb(0.10, 0.10, 0.12),       // GRAY_900
        primary: Color::from_rgb(0.00, 0.61, 0.65),    // Clinical teal PRIMARY_500
        success: Color::from_rgb(0.20, 0.70, 0.40),    // Green
        warning: Color::from_rgb(0.95, 0.65, 0.05),    // Amber
        danger: Color::from_rgb(0.85, 0.25, 0.25),     // Red
    }
}

/// Deuteranopia (green-weak) light palette.
/// Uses blue-orange-purple scheme to maximize distinguishability.
fn light_deuteranopia() -> Palette {
    Palette {
        background: Color::from_rgb(0.98, 0.98, 0.99),
        text: Color::from_rgb(0.10, 0.10, 0.12),
        primary: Color::from_rgb(0.00, 0.45, 0.70), // Blue #0072B2
        success: Color::from_rgb(0.00, 0.45, 0.70), // Blue (same as primary)
        warning: Color::from_rgb(0.90, 0.62, 0.00), // Orange #E69F00
        danger: Color::from_rgb(0.80, 0.47, 0.65),  // Reddish purple #CC79A7
    }
}

/// Protanopia (red-blind) light palette.
/// Uses blue-yellow-magenta scheme.
fn light_protanopia() -> Palette {
    Palette {
        background: Color::from_rgb(0.98, 0.98, 0.99),
        text: Color::from_rgb(0.10, 0.10, 0.12),
        primary: Color::from_rgb(0.00, 0.62, 0.45), // Bluish green #009E73
        success: Color::from_rgb(0.00, 0.62, 0.45), // Bluish green
        warning: Color::from_rgb(0.94, 0.89, 0.26), // Yellow #F0E442
        danger: Color::from_rgb(0.86, 0.15, 0.50),  // Magenta #DC267F
    }
}

/// Tritanopia (blue-yellow blind) light palette.
/// Uses red-green-magenta spectrum to avoid blue-yellow confusion.
fn light_tritanopia() -> Palette {
    Palette {
        background: Color::from_rgb(0.98, 0.98, 0.99),
        text: Color::from_rgb(0.10, 0.10, 0.12),
        primary: Color::from_rgb(0.60, 0.20, 0.50), // Purple
        success: Color::from_rgb(0.20, 0.70, 0.40), // Green
        warning: Color::from_rgb(0.90, 0.38, 0.13), // Red-orange
        danger: Color::from_rgb(0.80, 0.20, 0.35),  // Dark red/magenta
    }
}

// =============================================================================
// DARK MODE PALETTES
// =============================================================================

/// Standard clinical dark palette.
fn dark_standard() -> Palette {
    Palette {
        background: Color::from_rgb(0.08, 0.08, 0.10), // Near black
        text: Color::from_rgb(0.95, 0.95, 0.97),       // Near white
        primary: Color::from_rgb(0.15, 0.75, 0.80),    // Brighter teal
        success: Color::from_rgb(0.35, 0.80, 0.55),    // Brighter green
        warning: Color::from_rgb(1.0, 0.75, 0.20),     // Brighter yellow
        danger: Color::from_rgb(0.95, 0.40, 0.40),     // Brighter red
    }
}

/// Deuteranopia dark palette.
fn dark_deuteranopia() -> Palette {
    Palette {
        background: Color::from_rgb(0.08, 0.08, 0.10),
        text: Color::from_rgb(0.95, 0.95, 0.97),
        primary: Color::from_rgb(0.40, 0.60, 0.90), // Brighter blue
        success: Color::from_rgb(0.40, 0.60, 0.90), // Blue
        warning: Color::from_rgb(1.0, 0.75, 0.20),  // Brighter orange
        danger: Color::from_rgb(0.90, 0.60, 0.75),  // Brighter purple
    }
}

/// Protanopia dark palette.
fn dark_protanopia() -> Palette {
    Palette {
        background: Color::from_rgb(0.08, 0.08, 0.10),
        text: Color::from_rgb(0.95, 0.95, 0.97),
        primary: Color::from_rgb(0.20, 0.80, 0.65), // Bright teal
        success: Color::from_rgb(0.20, 0.80, 0.65), // Bright teal
        warning: Color::from_rgb(1.0, 0.95, 0.40),  // Bright yellow
        danger: Color::from_rgb(0.95, 0.35, 0.65),  // Bright magenta
    }
}

/// Tritanopia dark palette.
fn dark_tritanopia() -> Palette {
    Palette {
        background: Color::from_rgb(0.08, 0.08, 0.10),
        text: Color::from_rgb(0.95, 0.95, 0.97),
        primary: Color::from_rgb(0.70, 0.35, 0.60), // Purple
        success: Color::from_rgb(0.35, 0.85, 0.55), // Bright green
        warning: Color::from_rgb(1.0, 0.55, 0.30),  // Red-orange
        danger: Color::from_rgb(0.95, 0.40, 0.55),  // Magenta-red
    }
}
