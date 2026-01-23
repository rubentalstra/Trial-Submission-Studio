//! Dialog views for Trial Submission Studio.
//!
//! Modal dialogs for application-level interactions:
//! - About: Application info, version, links
//! - Settings: User preferences with master-detail layout
//! - Third-party: Open source license acknowledgments
//! - Update: Check for and install updates
//! - Close Project: Confirmation before closing a project
//! - Export Progress: Export operation in progress
//! - Export Complete: Export results (success, error, cancelled)
//!
//! Each dialog has view functions for standalone window mode (multi-window).

pub mod about;
pub mod close_study;
pub mod export;
pub mod settings;
pub mod third_party;
pub mod update;

// Re-export view functions for standalone window mode (multi-window)
pub use about::view_about_dialog_content;
pub use close_study::view_close_project_dialog_content;
pub use export::{view_export_complete_dialog_content, view_export_progress_dialog_content};
pub use settings::view_settings_dialog_content;
pub use third_party::view_third_party_dialog_content;
pub use update::view_update_dialog_content;
