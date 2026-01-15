//! Typography definitions for consistent text styling.
//!
//! All font sizes are in pixels (f32).

// =============================================================================
// FONT SIZES
// =============================================================================

/// Caption text - labels, hints, footnotes
pub const FONT_SIZE_CAPTION: f32 = 11.0;

/// Small text - secondary information, metadata
pub const FONT_SIZE_SMALL: f32 = 12.0;

/// Body text - default text size
pub const FONT_SIZE_BODY: f32 = 14.0;

/// Subtitle text - emphasized body text
pub const FONT_SIZE_SUBTITLE: f32 = 16.0;

/// Title text - section headers
pub const FONT_SIZE_TITLE: f32 = 20.0;

/// Heading text - page headers
pub const FONT_SIZE_HEADING: f32 = 24.0;

/// Display text - hero sections, large headings
pub const FONT_SIZE_DISPLAY: f32 = 32.0;

// =============================================================================
// LINE HEIGHTS
// =============================================================================

/// Tight line height - headings, single-line elements
pub const LINE_HEIGHT_TIGHT: f32 = 1.2;

/// Normal line height - body text
pub const LINE_HEIGHT_NORMAL: f32 = 1.5;

/// Relaxed line height - readable paragraphs
pub const LINE_HEIGHT_RELAXED: f32 = 1.75;

// =============================================================================
// FONT WEIGHTS (for reference, Iced uses font families)
// =============================================================================

// Note: Iced handles font weights through Font families.
// These constants are for documentation and future use.

/// Light weight
pub const FONT_WEIGHT_LIGHT: u16 = 300;

/// Normal/Regular weight
pub const FONT_WEIGHT_NORMAL: u16 = 400;

/// Medium weight
pub const FONT_WEIGHT_MEDIUM: u16 = 500;

/// Semi-bold weight
pub const FONT_WEIGHT_SEMIBOLD: u16 = 600;

/// Bold weight
pub const FONT_WEIGHT_BOLD: u16 = 700;

// =============================================================================
// TEXT STYLES (helper functions can be added here)
// =============================================================================

/// Maximum characters for single-line truncation
pub const MAX_CHARS_SINGLE_LINE: usize = 80;

/// Maximum characters for variable names (CDISC limit)
pub const MAX_CHARS_VARIABLE_NAME: usize = 8;

/// Maximum characters for labels (CDISC limit)
pub const MAX_CHARS_LABEL: usize = 200;

/// Maximum characters for short labels
pub const MAX_CHARS_SHORT_LABEL: usize = 40;
