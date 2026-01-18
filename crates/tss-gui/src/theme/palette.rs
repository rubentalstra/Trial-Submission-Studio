//! Color palette definitions for the Professional Clinical theme.
//!
//! This module defines all colors used throughout the application.
//! Colors follow a 10-shade scale pattern (50-900) for flexibility.

#![allow(dead_code)]

use iced::Color;

/// Helper function to create Color from RGB values (0.0-1.0 range)
const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}

/// Helper function to create Color from RGB values with alpha
const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color { r, g, b, a }
}

// =============================================================================
// PRIMARY - Teal/Cyan (Professional Clinical)
// =============================================================================

/// Lightest primary tint - backgrounds, hover states
pub const PRIMARY_50: Color = rgb(0.88, 0.97, 0.98); // #E0F7FA

/// Light primary - subtle highlights
pub const PRIMARY_100: Color = rgb(0.70, 0.92, 0.95); // #B3EBF2

/// Light-medium primary
pub const PRIMARY_200: Color = rgb(0.50, 0.85, 0.90); // #80D9E6

/// Medium-light primary
pub const PRIMARY_300: Color = rgb(0.30, 0.78, 0.82); // #4DC7D1

/// Medium primary
pub const PRIMARY_400: Color = rgb(0.15, 0.70, 0.75); // #26B3BF

/// **Main accent color** - buttons, links, active states
pub const PRIMARY_500: Color = rgb(0.00, 0.61, 0.65); // #009BA6

/// Darker primary - hover on primary buttons
pub const PRIMARY_600: Color = rgb(0.00, 0.52, 0.56); // #00858E

/// Dark primary - active/pressed states
pub const PRIMARY_700: Color = rgb(0.00, 0.44, 0.47); // #007078

/// Darker primary - rarely used
pub const PRIMARY_800: Color = rgb(0.00, 0.35, 0.38); // #005A61

/// Darkest primary shade
pub const PRIMARY_900: Color = rgb(0.00, 0.27, 0.29); // #00454A

// =============================================================================
// SEMANTIC COLORS
// =============================================================================

/// Success color - validation passed, complete states
pub const SUCCESS: Color = rgb(0.20, 0.70, 0.40); // #33B366

/// Success light - success backgrounds
pub const SUCCESS_LIGHT: Color = rgb(0.85, 0.95, 0.88); // #D9F2E0

/// Warning color - attention needed, incomplete states
pub const WARNING: Color = rgb(0.95, 0.65, 0.05); // #F2A60D

/// Warning light - warning backgrounds
pub const WARNING_LIGHT: Color = rgb(0.99, 0.95, 0.85); // #FDF2D9

/// Error/Danger color - errors, destructive actions
pub const ERROR: Color = rgb(0.85, 0.25, 0.25); // #D94040

/// Error light - error backgrounds
pub const ERROR_LIGHT: Color = rgb(0.98, 0.90, 0.90); // #FAE6E6

/// Info color - informational messages
pub const INFO: Color = rgb(0.25, 0.55, 0.85); // #408CD9

/// Info light - info backgrounds
pub const INFO_LIGHT: Color = rgb(0.90, 0.95, 0.99); // #E6F2FC

// =============================================================================
// NEUTRAL GRAYS
// =============================================================================

/// Lightest gray - main background
pub const GRAY_50: Color = rgb(0.98, 0.98, 0.99); // #FAFAFE

/// Very light gray - card/surface backgrounds
pub const GRAY_100: Color = rgb(0.95, 0.95, 0.97); // #F2F2F7

/// Light gray - borders, dividers
pub const GRAY_200: Color = rgb(0.90, 0.90, 0.93); // #E6E6ED

/// Medium-light gray - dividers, separators
pub const GRAY_300: Color = rgb(0.82, 0.82, 0.86); // #D1D1DB

/// Medium gray - placeholder text, icons
pub const GRAY_400: Color = rgb(0.65, 0.65, 0.70); // #A6A6B3

/// Medium gray - secondary text
pub const GRAY_500: Color = rgb(0.50, 0.50, 0.55); // #80808C

/// Medium-dark gray - muted text
pub const GRAY_600: Color = rgb(0.40, 0.40, 0.45); // #666673

/// Dark gray - body text
pub const GRAY_700: Color = rgb(0.30, 0.30, 0.35); // #4D4D59

/// Darker gray - headings
pub const GRAY_800: Color = rgb(0.20, 0.20, 0.24); // #33333D

/// Darkest gray - primary text
pub const GRAY_900: Color = rgb(0.10, 0.10, 0.12); // #1A1A1F

// =============================================================================
// SPECIAL PURPOSE
// =============================================================================

/// Pure white
pub const WHITE: Color = rgb(1.0, 1.0, 1.0); // #FFFFFF

/// Pure black
pub const BLACK: Color = rgb(0.0, 0.0, 0.0); // #000000

/// Transparent
pub const TRANSPARENT: Color = rgba(0.0, 0.0, 0.0, 0.0);

/// Modal backdrop overlay
pub const BACKDROP: Color = rgba(0.0, 0.0, 0.0, 0.5);

/// Shadow color
pub const SHADOW: Color = rgba(0.0, 0.0, 0.0, 0.08);

/// Strong shadow color
pub const SHADOW_STRONG: Color = rgba(0.0, 0.0, 0.0, 0.16);

// =============================================================================
// MAPPING STATUS COLORS
// =============================================================================

/// Mapped variable - complete
pub const STATUS_MAPPED: Color = SUCCESS;

/// Not mapped - needs attention
pub const STATUS_NOT_MAPPED: Color = ERROR;

/// Unmapped/Skipped - intentionally omitted
pub const STATUS_UNMAPPED: Color = GRAY_400;

/// In progress - partially configured
pub const STATUS_IN_PROGRESS: Color = WARNING;

// =============================================================================
// EXPORT STATUS COLORS
// =============================================================================

/// Ready to export
pub const EXPORT_READY: Color = SUCCESS;

/// Export incomplete - missing required fields
pub const EXPORT_INCOMPLETE: Color = WARNING;

/// Export has errors - cannot export
pub const EXPORT_HAS_ERRORS: Color = ERROR;
