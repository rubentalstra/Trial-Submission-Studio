//! Theme module for Trial Submission Studio.
//!
//! This module provides the Professional Clinical theme with:
//! - Color palette (`palette`)
//! - Spacing constants (`spacing`)
//! - Typography definitions (`typography`)
//! - Custom widget styles (`clinical`)
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::theme::{clinical_light, palette, spacing, typography};
//!
//! // Create the theme
//! let theme = clinical_light();
//!
//! // Use palette colors
//! let primary_color = palette::PRIMARY_500;
//!
//! // Use spacing constants
//! let padding = spacing::SPACING_MD;
//!
//! // Use typography
//! let font_size = typography::FONT_SIZE_BODY;
//! ```

// Allow unused imports - these are public API re-exports
#![allow(unused_imports)]

pub mod clinical;
pub mod palette;
pub mod spacing;
pub mod typography;

// Re-export commonly used items
pub use clinical::clinical_light;
pub use palette::{
    BACKDROP,
    BLACK,
    ERROR,
    ERROR_LIGHT,
    EXPORT_HAS_ERRORS,
    EXPORT_INCOMPLETE,
    EXPORT_READY,
    GRAY_50,
    // Grays
    GRAY_100,
    GRAY_200,
    GRAY_300,
    GRAY_400,
    GRAY_500,
    GRAY_600,
    GRAY_700,
    GRAY_800,
    GRAY_900,
    INFO,
    INFO_LIGHT,
    PRIMARY_50,
    PRIMARY_100,
    PRIMARY_200,
    PRIMARY_300,
    PRIMARY_400,
    // Primary colors
    PRIMARY_500,
    PRIMARY_500 as PRIMARY,
    PRIMARY_600,
    PRIMARY_700,
    PRIMARY_800,
    PRIMARY_900,
    SHADOW,
    SHADOW_STRONG,
    STATUS_IN_PROGRESS,
    // Status colors
    STATUS_MAPPED,
    STATUS_NOT_MAPPED,
    STATUS_UNMAPPED,
    // Semantic colors
    SUCCESS,
    SUCCESS_LIGHT,
    TRANSPARENT,
    WARNING,
    WARNING_LIGHT,
    // Special
    WHITE,
};

pub use spacing::{
    BORDER_RADIUS_FULL,
    BORDER_RADIUS_LG,
    BORDER_RADIUS_MD,
    // Border radius
    BORDER_RADIUS_SM,
    BORDER_RADIUS_XL,
    BORDER_WIDTH_MEDIUM,
    BORDER_WIDTH_THICK,
    // Border widths
    BORDER_WIDTH_THIN,
    BUTTON_HEIGHT_LG,
    BUTTON_HEIGHT_MD,
    // Button heights
    BUTTON_HEIGHT_SM,
    ICON_SIZE_LG,
    ICON_SIZE_MD,
    // Icon sizes
    ICON_SIZE_SM,
    ICON_SIZE_XL,
    // Input heights
    INPUT_HEIGHT,
    INPUT_HEIGHT_LG,
    // Layout widths
    MASTER_WIDTH,
    MODAL_WIDTH_LG,
    MODAL_WIDTH_MD,
    MODAL_WIDTH_SM,
    MODAL_WIDTH_XL,
    SCROLLBAR_RADIUS,
    // Scrollbar
    SCROLLBAR_WIDTH,
    SETTINGS_WIDTH,
    SIDEBAR_WIDTH,
    SIDEBAR_WIDTH_NARROW,
    SIDEBAR_WIDTH_WIDE,
    SPACING_LG,
    SPACING_MD,
    SPACING_SM,
    SPACING_XL,
    // Spacing
    SPACING_XS,
    SPACING_XXL,
    TAB_BAR_HEIGHT,
    TAB_INDICATOR_HEIGHT,
    // Table
    TAB_PADDING_X,
    TAB_PADDING_Y,
    // Tab bar
    TABLE_CELL_PADDING_X,
    TABLE_CELL_PADDING_Y,
    TABLE_HEADER_HEIGHT,
    TABLE_ROW_HEIGHT_COMFORTABLE,
    TABLE_ROW_HEIGHT_COMPACT,
    TABLE_ROW_HEIGHT_DEFAULT,
};

pub use typography::{
    FONT_SIZE_BODY,
    // Font sizes
    FONT_SIZE_CAPTION,
    FONT_SIZE_DISPLAY,
    FONT_SIZE_HEADING,
    FONT_SIZE_SMALL,
    FONT_SIZE_SUBTITLE,
    FONT_SIZE_TITLE,
    LINE_HEIGHT_NORMAL,
    LINE_HEIGHT_RELAXED,
    // Line heights
    LINE_HEIGHT_TIGHT,
    MAX_CHARS_LABEL,
    MAX_CHARS_SHORT_LABEL,
    // Character limits
    MAX_CHARS_SINGLE_LINE,
    MAX_CHARS_VARIABLE_NAME,
};

// Re-export widget style functions
pub use clinical::{
    button_danger,
    button_ghost,
    // Button styles
    button_primary,
    button_secondary,
    // Container styles
    container_card,
    container_inset,
    container_modal,
    container_sidebar,
    container_surface,
    // Progress bar styles
    progress_bar_primary,
    progress_bar_success,
    // Text input styles
    text_input_default,
};
