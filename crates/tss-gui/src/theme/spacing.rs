//! Spacing constants for consistent layout throughout the application.
//!
//! All spacing values are in pixels (f32) and follow a consistent scale.

// =============================================================================
// SPACING SCALE
// =============================================================================

/// Extra small spacing - tight gaps between related elements
pub const SPACING_XS: f32 = 4.0;

/// Small spacing - small gaps, icon margins
pub const SPACING_SM: f32 = 8.0;

/// Medium spacing - default padding, standard gaps
pub const SPACING_MD: f32 = 16.0;

/// Large spacing - section padding, major gaps
pub const SPACING_LG: f32 = 24.0;

/// Extra large spacing - page margins, large separations
pub const SPACING_XL: f32 = 32.0;

/// Double extra large spacing - hero sections, major divisions
pub const SPACING_XXL: f32 = 48.0;

// =============================================================================
// BORDER RADIUS
// =============================================================================

/// Small radius - buttons, inputs, chips
pub const BORDER_RADIUS_SM: f32 = 4.0;

/// Medium radius - cards, panels
pub const BORDER_RADIUS_MD: f32 = 6.0;

/// Large radius - modals, dialogs
pub const BORDER_RADIUS_LG: f32 = 8.0;

/// Extra large radius - special containers
pub const BORDER_RADIUS_XL: f32 = 12.0;

/// Full/pill radius - tags, badges
pub const BORDER_RADIUS_FULL: f32 = 9999.0;

// =============================================================================
// BORDER WIDTHS
// =============================================================================

/// Thin border - subtle separators
pub const BORDER_WIDTH_THIN: f32 = 1.0;

/// Medium border - standard borders
pub const BORDER_WIDTH_MEDIUM: f32 = 2.0;

/// Thick border - emphasis borders
pub const BORDER_WIDTH_THICK: f32 = 3.0;

// =============================================================================
// COMPONENT SIZES
// =============================================================================

/// Icon size - small (inline with text)
pub const ICON_SIZE_SM: f32 = 16.0;

/// Icon size - medium (buttons, list items)
pub const ICON_SIZE_MD: f32 = 20.0;

/// Icon size - large (feature icons)
pub const ICON_SIZE_LG: f32 = 24.0;

/// Icon size - extra large (hero icons)
pub const ICON_SIZE_XL: f32 = 32.0;

/// Button height - small
pub const BUTTON_HEIGHT_SM: f32 = 28.0;

/// Button height - medium (default)
pub const BUTTON_HEIGHT_MD: f32 = 36.0;

/// Button height - large
pub const BUTTON_HEIGHT_LG: f32 = 44.0;

/// Input height - standard
pub const INPUT_HEIGHT: f32 = 36.0;

/// Input height - large
pub const INPUT_HEIGHT_LG: f32 = 44.0;

// =============================================================================
// LAYOUT WIDTHS
// =============================================================================

/// Master panel width in master-detail layouts
pub const MASTER_WIDTH: f32 = 320.0;

/// Sidebar width - master panel in master-detail layouts
pub const SIDEBAR_WIDTH: f32 = 280.0;

/// Sidebar width - narrow variant
pub const SIDEBAR_WIDTH_NARROW: f32 = 220.0;

/// Sidebar width - wide variant
pub const SIDEBAR_WIDTH_WIDE: f32 = 340.0;

/// Modal width - small
pub const MODAL_WIDTH_SM: f32 = 320.0;

/// Modal width - medium (default)
pub const MODAL_WIDTH_MD: f32 = 480.0;

/// Modal width - large
pub const MODAL_WIDTH_LG: f32 = 640.0;

/// Modal width - extra large
pub const MODAL_WIDTH_XL: f32 = 800.0;

/// Settings panel width
pub const SETTINGS_WIDTH: f32 = 600.0;

// =============================================================================
// SCROLLBAR
// =============================================================================

/// Scrollbar width
pub const SCROLLBAR_WIDTH: f32 = 8.0;

/// Scrollbar border radius
pub const SCROLLBAR_RADIUS: f32 = 4.0;

// =============================================================================
// DATA TABLE
// =============================================================================

/// Table row height - compact
pub const TABLE_ROW_HEIGHT_COMPACT: f32 = 32.0;

/// Table row height - default
pub const TABLE_ROW_HEIGHT_DEFAULT: f32 = 40.0;

/// Table row height - comfortable
pub const TABLE_ROW_HEIGHT_COMFORTABLE: f32 = 48.0;

/// Table header height
pub const TABLE_HEADER_HEIGHT: f32 = 44.0;

/// Table cell padding horizontal
pub const TABLE_CELL_PADDING_X: f32 = 12.0;

/// Table cell padding vertical
pub const TABLE_CELL_PADDING_Y: f32 = 8.0;

// =============================================================================
// TAB BAR
// =============================================================================

/// Tab bar height
pub const TAB_BAR_HEIGHT: f32 = 44.0;

/// Tab item padding horizontal
pub const TAB_PADDING_X: f32 = 16.0;

/// Tab item padding vertical
pub const TAB_PADDING_Y: f32 = 8.0;

/// Tab indicator height (active tab underline)
pub const TAB_INDICATOR_HEIGHT: f32 = 2.0;
