//! Color palette system with accessibility mode support.
//!
//! This module provides runtime-switchable color palettes optimized for
//! different types of color vision deficiency.

pub mod dark;
pub mod light;

use serde::{Deserialize, Serialize};

/// Color vision accessibility modes.
///
/// Based on the three most common types of color vision deficiency.
/// Color palettes follow IBM Color Blind Safe and Wong (Nature Methods) research.
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

    /// Description for settings UI.
    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Standard => "Default color scheme",
            Self::Deuteranopia => "Optimized for green-weak color vision",
            Self::Protanopia => "Optimized for red-blind color vision",
            Self::Tritanopia => "Optimized for blue-yellow color vision",
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

    /// Description for settings UI.
    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Light => "Light background with dark text",
            Self::Dark => "Dark background with light text",
            Self::System => "Follow operating system preference",
        }
    }

    /// All available modes for UI picker.
    pub const ALL: [Self; 3] = [Self::Light, Self::Dark, Self::System];

    /// Resolve System mode to actual Light/Dark based on OS preference.
    pub fn resolve(&self) -> Self {
        match self {
            Self::System => {
                if Self::detect_system_dark_mode() {
                    Self::Dark
                } else {
                    Self::Light
                }
            }
            mode => *mode,
        }
    }

    /// Detect if the system is in dark mode.
    ///
    /// Currently defaults to light mode. System theme detection can be
    /// added in the future using the `dark-light` crate.
    fn detect_system_dark_mode() -> bool {
        // TODO: Implement system theme detection using dark-light crate
        // when the feature is added to Cargo.toml
        false
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}
