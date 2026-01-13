//! View module for Trial Submission Studio.
//!
//! This module contains all view implementations using Iced 0.14.0.
//! Views are pure functions that render UI based on application state.
//!
//! ## Module Structure
//!
//! - `home.rs` - Home screen (study selection, domain list)
//! - `domain_editor/` - Domain editing with tabbed interface
//! - `export.rs` - Export configuration and progress
//! - `dialog/` - Modal dialogs (about, settings, third-party, update)

pub mod dialog;
pub mod domain_editor;
pub mod export;
pub mod home;

// Re-export commonly used view functions (overlay/modal mode)
pub use dialog::{
    view_about_dialog, view_settings_dialog, view_third_party_dialog, view_update_dialog,
};

// Re-export view functions for standalone window mode (multi-window)
pub use dialog::{
    view_about_dialog_content, view_settings_dialog_content, view_third_party_dialog_content,
    view_update_dialog_content,
};

pub use domain_editor::view_domain_editor;
pub use export::view_export;
pub use home::view_home;
