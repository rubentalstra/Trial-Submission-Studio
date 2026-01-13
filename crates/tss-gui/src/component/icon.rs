//! Icon helper components using iced_fonts.
//!
//! Provides convenient wrappers for Font Awesome icons.

use iced::widget::text;
use iced::{Element, Font};

/// Font Awesome Solid font
const FA_SOLID: Font = Font::with_name("Font Awesome 6 Free Solid");

// =============================================================================
// ICON CHARACTERS (Font Awesome 6 Solid)
// =============================================================================

/// Folder icon character
const FOLDER_CHAR: char = '\u{f07b}';
/// File icon character
const FILE_CHAR: char = '\u{f15b}';
/// Check/checkmark icon character
const CHECK_CHAR: char = '\u{f00c}';
/// Warning triangle icon character
const WARNING_CHAR: char = '\u{f071}';
/// Error/times circle icon character
const ERROR_CHAR: char = '\u{f057}';
/// Search/magnifying glass icon character
const SEARCH_CHAR: char = '\u{f002}';
/// Spinner icon character
const SPINNER_CHAR: char = '\u{f110}';
/// Cog/settings icon character
const COG_CHAR: char = '\u{f013}';
/// Export/file-export icon character
const EXPORT_CHAR: char = '\u{f56e}';
/// Close/times icon character
const CLOSE_CHAR: char = '\u{f00d}';
/// Info circle icon character
const INFO_CHAR: char = '\u{f05a}';
/// Plus icon character
const PLUS_CHAR: char = '\u{2b}';
/// Minus icon character
const MINUS_CHAR: char = '\u{f068}';
/// Chevron right icon character
const CHEVRON_RIGHT_CHAR: char = '\u{f054}';
/// Chevron down icon character
const CHEVRON_DOWN_CHAR: char = '\u{f078}';
/// External link icon character
const EXTERNAL_LINK_CHAR: char = '\u{f35d}';

// =============================================================================
// GENERIC ICON FUNCTION
// =============================================================================

/// Create an icon element with the specified character and optional size.
///
/// # Arguments
///
/// * `icon_char` - The Font Awesome unicode character
/// * `size` - Optional font size (defaults to 16.0)
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::icon;
///
/// let folder_icon = icon('\u{f07b}', Some(20.0));
/// ```
pub fn icon<'a, M: 'a>(icon_char: char, size: Option<f32>) -> Element<'a, M> {
    text(icon_char.to_string())
        .font(FA_SOLID)
        .size(size.unwrap_or(16.0))
        .into()
}

// =============================================================================
// CONVENIENCE ICON FUNCTIONS
// =============================================================================

/// Folder icon (default size 16px)
pub fn icon_folder<'a, M: 'a>() -> Element<'a, M> {
    icon(FOLDER_CHAR, None)
}

/// File icon (default size 16px)
pub fn icon_file<'a, M: 'a>() -> Element<'a, M> {
    icon(FILE_CHAR, None)
}

/// Check/checkmark icon (default size 16px)
pub fn icon_check<'a, M: 'a>() -> Element<'a, M> {
    icon(CHECK_CHAR, None)
}

/// Warning triangle icon (default size 16px)
pub fn icon_warning<'a, M: 'a>() -> Element<'a, M> {
    icon(WARNING_CHAR, None)
}

/// Error/times circle icon (default size 16px)
pub fn icon_error<'a, M: 'a>() -> Element<'a, M> {
    icon(ERROR_CHAR, None)
}

/// Search/magnifying glass icon (default size 16px)
pub fn icon_search<'a, M: 'a>() -> Element<'a, M> {
    icon(SEARCH_CHAR, None)
}

/// Spinner icon (default size 16px)
pub fn icon_spinner<'a, M: 'a>() -> Element<'a, M> {
    icon(SPINNER_CHAR, None)
}

/// Cog/settings icon (default size 16px)
pub fn icon_cog<'a, M: 'a>() -> Element<'a, M> {
    icon(COG_CHAR, None)
}

/// Export icon (default size 16px)
pub fn icon_export<'a, M: 'a>() -> Element<'a, M> {
    icon(EXPORT_CHAR, None)
}

/// Close/times icon (default size 16px)
pub fn icon_close<'a, M: 'a>() -> Element<'a, M> {
    icon(CLOSE_CHAR, None)
}

/// Info circle icon (default size 16px)
pub fn icon_info<'a, M: 'a>() -> Element<'a, M> {
    icon(INFO_CHAR, None)
}

/// Plus icon (default size 16px)
pub fn icon_plus<'a, M: 'a>() -> Element<'a, M> {
    icon(PLUS_CHAR, None)
}

/// Minus icon (default size 16px)
pub fn icon_minus<'a, M: 'a>() -> Element<'a, M> {
    icon(MINUS_CHAR, None)
}

/// Chevron right icon (default size 16px)
pub fn icon_chevron_right<'a, M: 'a>() -> Element<'a, M> {
    icon(CHEVRON_RIGHT_CHAR, None)
}

/// Chevron down icon (default size 16px)
pub fn icon_chevron_down<'a, M: 'a>() -> Element<'a, M> {
    icon(CHEVRON_DOWN_CHAR, None)
}

/// External link icon (default size 16px)
pub fn icon_external_link<'a, M: 'a>() -> Element<'a, M> {
    icon(EXTERNAL_LINK_CHAR, None)
}

// =============================================================================
// SIZED ICON FUNCTIONS
// =============================================================================

/// Create a small icon (12px)
pub fn icon_sm<'a, M: 'a>(icon_char: char) -> Element<'a, M> {
    icon(icon_char, Some(12.0))
}

/// Create a medium icon (20px)
pub fn icon_md<'a, M: 'a>(icon_char: char) -> Element<'a, M> {
    icon(icon_char, Some(20.0))
}

/// Create a large icon (24px)
pub fn icon_lg<'a, M: 'a>(icon_char: char) -> Element<'a, M> {
    icon(icon_char, Some(24.0))
}

/// Create an extra large icon (32px)
pub fn icon_xl<'a, M: 'a>(icon_char: char) -> Element<'a, M> {
    icon(icon_char, Some(32.0))
}
