//! Helper functions and types for the Preview tab.
//!
//! Contains constants, colors, and value formatting utilities.

use iced::Color;

use crate::theme::ThemeConfig;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Minimum column width
pub(super) const MIN_COL_WIDTH: f32 = 60.0;

/// Maximum column width
pub(super) const MAX_COL_WIDTH: f32 = 300.0;

/// Padding inside cells
pub(super) const CELL_PADDING_X: f32 = 12.0;
pub(super) const CELL_PADDING_Y: f32 = 8.0;

/// Approximate character width for calculating column widths
pub(super) const CHAR_WIDTH: f32 = 7.5;

// =============================================================================
// NOT COLLECTED COLUMN COLORS
// =============================================================================

/// Colors for "Not Collected" columns.
pub(super) struct NotCollectedColors {
    pub(super) bg: Color,
    pub(super) bg_alt: Color,
    pub(super) header_bg: Color,
    pub(super) badge_bg: Color,
}

/// Get the Not Collected colors for theming.
pub(super) fn not_collected_colors(config: &ThemeConfig) -> NotCollectedColors {
    use crate::theme::ClinicalColors;

    // Use warning colors as base for "Not Collected" columns
    let theme = config.to_theme(false);
    let warning = theme.extended_palette().warning.base.color;
    let warning_light = theme.clinical().status_warning_light;

    NotCollectedColors {
        bg: warning_light,
        bg_alt: Color {
            a: warning_light.a * 1.1,
            ..warning_light
        },
        header_bg: Color {
            a: warning_light.a * 1.2,
            ..warning_light
        },
        badge_bg: warning,
    }
}

// =============================================================================
// VALUE FORMATTING
// =============================================================================

/// Format a Polars AnyValue for display.
pub(super) fn format_anyvalue(value: &polars::prelude::AnyValue) -> String {
    use polars::prelude::AnyValue;

    match value {
        AnyValue::Null => String::new(),
        AnyValue::Boolean(b) => if *b { "Y" } else { "N" }.to_string(),
        AnyValue::Int8(n) => n.to_string(),
        AnyValue::Int16(n) => n.to_string(),
        AnyValue::Int32(n) => n.to_string(),
        AnyValue::Int64(n) => n.to_string(),
        AnyValue::UInt8(n) => n.to_string(),
        AnyValue::UInt16(n) => n.to_string(),
        AnyValue::UInt32(n) => n.to_string(),
        AnyValue::UInt64(n) => n.to_string(),
        AnyValue::Float32(n) => {
            if n.fract() == 0.0 {
                format!("{:.0}", n)
            } else {
                format!("{:.4}", n)
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            }
        }
        AnyValue::Float64(n) => {
            if n.fract() == 0.0 {
                format!("{:.0}", n)
            } else {
                format!("{:.4}", n)
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            }
        }
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        _ => format!("{}", value),
    }
}
