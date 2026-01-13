//! View module for Trial Submission Studio.
//!
//! This module contains all view implementations using Iced 0.14.0.
//! Views are pure functions that render UI based on application state.
//!
//! ## Module Structure
//!
//! - `home.rs` - Home screen (study selection, domain list)
//! - `export.rs` - Export configuration and progress
//! - `domain_editor/` - Domain editing with tabbed interface
//! - `dialog/` - Modal dialogs (about, settings, etc.)

pub mod home;

// Re-export commonly used view functions
pub use home::view_home;

// TODO: Phase 4 - Domain editor views
// pub mod domain_editor;
// pub use domain_editor::view_domain_editor;

// TODO: Phase 5 - Export and dialog views
// pub mod export;
// pub mod dialog;
// pub use export::view_export;
