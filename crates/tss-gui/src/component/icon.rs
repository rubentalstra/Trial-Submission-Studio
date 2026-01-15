//! Icon helper components using iced_fonts with Lucide icons.
//!
//! This module provides convenient wrappers for Lucide icons via iced_fonts.
//! Lucide is a clean, modern icon set perfect for professional applications.
//!
//! # Usage
//!
//! ```rust,ignore
//! use tss_gui::component::icon::{icon_folder, icon_check, icon_search};
//!
//! // Use directly in your view - returns Text widget
//! row![
//!     icon_folder(),
//!     text("Documents"),
//! ]
//!
//! // Custom size - chain .size()
//! icon_folder().size(24)
//! ```
//!
//! # Icon Reference
//!
//! See <https://lucide.dev/icons/> for the full icon catalog.

use iced::widget::Text;

// Re-export the Lucide font bytes for loading in main.rs
pub use iced_fonts::LUCIDE_FONT_BYTES;

// =============================================================================
// NAVIGATION & FILES
// =============================================================================

/// Folder icon - for directories and file navigation
pub fn icon_folder() -> Text<'static> {
    iced_fonts::lucide::folder()
}

/// Folder open icon - for expanded directories
pub fn icon_folder_open() -> Text<'static> {
    iced_fonts::lucide::folder_open()
}

/// File icon - for generic files
pub fn icon_file() -> Text<'static> {
    iced_fonts::lucide::file()
}

/// File text icon - for text/data files
pub fn icon_file_text() -> Text<'static> {
    iced_fonts::lucide::file_text()
}

/// File spreadsheet icon - for data/CSV files
pub fn icon_file_spreadsheet() -> Text<'static> {
    iced_fonts::lucide::file_spreadsheet()
}

// =============================================================================
// STATUS & FEEDBACK
// =============================================================================

/// Check/checkmark icon - for success states
pub fn icon_check() -> Text<'static> {
    iced_fonts::lucide::check()
}

/// Circle check icon - for confirmed/valid states
pub fn icon_circle_check() -> Text<'static> {
    iced_fonts::lucide::circle_check()
}

/// Warning triangle icon - for warnings
pub fn icon_warning() -> Text<'static> {
    iced_fonts::lucide::triangle_alert()
}

/// Error/X circle icon - for errors
pub fn icon_error() -> Text<'static> {
    iced_fonts::lucide::circle_x()
}

/// Info icon - for informational messages
pub fn icon_info() -> Text<'static> {
    iced_fonts::lucide::info()
}

// =============================================================================
// ACTIONS
// =============================================================================

/// Search/magnifying glass icon
pub fn icon_search() -> Text<'static> {
    iced_fonts::lucide::search()
}

/// Settings/cog icon
pub fn icon_settings() -> Text<'static> {
    iced_fonts::lucide::settings()
}

/// Cog icon (alias for settings)
pub fn icon_cog() -> Text<'static> {
    iced_fonts::lucide::settings()
}

/// Export/download icon
pub fn icon_export() -> Text<'static> {
    iced_fonts::lucide::download()
}

/// Upload icon
pub fn icon_upload() -> Text<'static> {
    iced_fonts::lucide::upload()
}

/// Close/X icon
pub fn icon_close() -> Text<'static> {
    iced_fonts::lucide::x()
}

/// Plus icon - for adding items
pub fn icon_plus() -> Text<'static> {
    iced_fonts::lucide::plus()
}

/// Minus icon - for removing items
pub fn icon_minus() -> Text<'static> {
    iced_fonts::lucide::minus()
}

/// Refresh/sync icon
pub fn icon_refresh() -> Text<'static> {
    iced_fonts::lucide::refresh_cw()
}

/// Save icon
pub fn icon_save() -> Text<'static> {
    iced_fonts::lucide::save()
}

/// Spinner/loader icon
pub fn icon_spinner() -> Text<'static> {
    iced_fonts::lucide::loader()
}

// =============================================================================
// NAVIGATION ARROWS
// =============================================================================

/// Chevron right icon - for navigation/expansion
pub fn icon_chevron_right() -> Text<'static> {
    iced_fonts::lucide::chevron_right()
}

/// Chevron down icon - for expanded state
pub fn icon_chevron_down() -> Text<'static> {
    iced_fonts::lucide::chevron_down()
}

/// Chevron left icon - for back navigation
pub fn icon_chevron_left() -> Text<'static> {
    iced_fonts::lucide::chevron_left()
}

/// Chevron up icon - for collapse
pub fn icon_chevron_up() -> Text<'static> {
    iced_fonts::lucide::chevron_up()
}

/// Arrow right icon
pub fn icon_arrow_right() -> Text<'static> {
    iced_fonts::lucide::arrow_right()
}

/// Arrow left icon
pub fn icon_arrow_left() -> Text<'static> {
    iced_fonts::lucide::arrow_left()
}

// =============================================================================
// CLINICAL/DOMAIN SPECIFIC
// =============================================================================

/// Table/grid icon - for data tables
pub fn icon_table() -> Text<'static> {
    iced_fonts::lucide::table()
}

/// List icon - for list views
pub fn icon_list() -> Text<'static> {
    iced_fonts::lucide::list()
}

/// Columns icon - for column management
pub fn icon_columns() -> Text<'static> {
    iced_fonts::lucide::columns_two()
}

/// Database icon - for data sources
pub fn icon_database() -> Text<'static> {
    iced_fonts::lucide::database()
}

/// Clipboard/checklist icon - for validation
pub fn icon_clipboard_check() -> Text<'static> {
    iced_fonts::lucide::clipboard_check()
}

/// Eye icon - for preview
pub fn icon_eye() -> Text<'static> {
    iced_fonts::lucide::eye()
}

/// Eye off icon - for hidden
pub fn icon_eye_off() -> Text<'static> {
    iced_fonts::lucide::eye_off()
}

/// Link icon - for mappings
pub fn icon_link() -> Text<'static> {
    iced_fonts::lucide::link()
}

/// Unlink icon - for unmapped
pub fn icon_unlink() -> Text<'static> {
    iced_fonts::lucide::unlink()
}

/// Layers icon - for domains
pub fn icon_layers() -> Text<'static> {
    iced_fonts::lucide::layers()
}

/// Package icon - for export packages
pub fn icon_package() -> Text<'static> {
    iced_fonts::lucide::package()
}

// =============================================================================
// UI ELEMENTS
// =============================================================================

/// Menu/hamburger icon
pub fn icon_menu() -> Text<'static> {
    iced_fonts::lucide::menu()
}

/// More horizontal (ellipsis) icon
pub fn icon_more_horizontal() -> Text<'static> {
    iced_fonts::lucide::ellipsis()
}

/// More vertical icon
pub fn icon_more_vertical() -> Text<'static> {
    iced_fonts::lucide::ellipsis_vertical()
}

/// External link icon
pub fn icon_external_link() -> Text<'static> {
    iced_fonts::lucide::external_link()
}

/// Home icon
pub fn icon_home() -> Text<'static> {
    iced_fonts::lucide::house()
}

/// Help/question mark icon
pub fn icon_help() -> Text<'static> {
    iced_fonts::lucide::circle_help()
}

/// Edit/pencil icon
pub fn icon_edit() -> Text<'static> {
    iced_fonts::lucide::pencil()
}

/// Trash/delete icon
pub fn icon_trash() -> Text<'static> {
    iced_fonts::lucide::trash_two()
}

/// Copy icon
pub fn icon_copy() -> Text<'static> {
    iced_fonts::lucide::copy()
}

/// Filter icon (funnel)
pub fn icon_filter() -> Text<'static> {
    iced_fonts::lucide::funnel()
}

/// Sort icon
pub fn icon_sort() -> Text<'static> {
    iced_fonts::lucide::arrow_up_down()
}

// =============================================================================
// LOADING & PROGRESS
// =============================================================================

/// Loader/spinner icon (alias)
pub fn icon_loader() -> Text<'static> {
    iced_fonts::lucide::loader()
}

/// Clock/time icon
pub fn icon_clock() -> Text<'static> {
    iced_fonts::lucide::timer()
}
