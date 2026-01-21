//! Semantic color system for accessibility-aware theming.
//!
//! This module defines semantic color roles that abstract away specific color values.
//! Each accessibility mode provides different color values for the same semantic roles.

use iced::Color;

/// Semantic color roles - defines the PURPOSE of a color, not its value.
///
/// This abstraction enables accessibility mode switching by mapping
/// semantic roles to different actual colors per palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum SemanticColor {
    // === Status Indicators ===
    /// Success state (validation passed, complete)
    StatusSuccess,
    /// Success state background
    StatusSuccessLight,
    /// Warning state (needs attention, incomplete)
    StatusWarning,
    /// Warning state background
    StatusWarningLight,
    /// Error state (failed, invalid)
    StatusError,
    /// Error state background
    StatusErrorLight,
    /// Informational state
    StatusInfo,
    /// Informational state background
    StatusInfoLight,

    // === Mapping Status ===
    /// Mapped/accepted column
    MappingMapped,
    /// Unmapped column (needs action)
    MappingUnmapped,
    /// Auto-suggested mapping
    MappingSuggested,
    /// Not collected (intentionally omitted)
    MappingNotCollected,
    /// In progress state
    MappingInProgress,

    // === Backgrounds ===
    /// Primary background (main app bg)
    BackgroundPrimary,
    /// Secondary background (cards, surfaces)
    BackgroundSecondary,
    /// Elevated surface (modals, dialogs)
    BackgroundElevated,
    /// Inset/recessed areas
    BackgroundInset,

    // === Text ===
    /// Primary text (headings, important)
    TextPrimary,
    /// Secondary text (body)
    TextSecondary,
    /// Muted text (descriptions, hints)
    TextMuted,
    /// Disabled text
    TextDisabled,
    /// Text on primary accent color
    TextOnAccent,

    // === Interactive (Accent) ===
    /// Primary accent (buttons, links)
    AccentPrimary,
    /// Accent hover state
    AccentHover,
    /// Accent pressed state
    AccentPressed,
    /// Accent disabled state
    AccentDisabled,
    /// Primary light tint for hover backgrounds
    AccentPrimaryLight,
    /// Primary medium tint
    AccentPrimaryMedium,

    // === Danger/Destructive ===
    /// Danger button hover
    DangerHover,
    /// Danger button pressed
    DangerPressed,

    // === Borders ===
    /// Default border
    BorderDefault,
    /// Subtle border (lighter)
    BorderSubtle,
    /// Focused element border
    BorderFocused,
    /// Error border
    BorderError,

    // === Special ===
    /// White (constant)
    White,
    /// Black (constant)
    Black,
    /// Transparent (constant)
    Transparent,
    /// Shadow color
    Shadow,
    /// Strong shadow color
    ShadowStrong,
    /// Modal backdrop overlay
    Backdrop,
}

/// Trait for color palette implementations.
///
/// Each accessibility mode implements this trait to provide
/// appropriate colors for all semantic roles.
pub trait Palette: Send + Sync {
    /// Resolve a semantic color to its actual color value.
    fn resolve(&self, color: SemanticColor) -> Color;
}
