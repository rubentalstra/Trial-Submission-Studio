//! Dialog views for Trial Submission Studio.
//!
//! Modal dialogs for application-level interactions:
//! - About: Application info, version, links
//! - Settings: User preferences with master-detail layout
//! - Third-party: Open source license acknowledgments
//! - Update: Check for and install updates
//! - Close Study: Confirmation before closing a study
//!
//! Each dialog has view functions for standalone window mode (multi-window).

pub mod about;
pub mod close_study;
pub mod settings;
pub mod third_party;
pub mod update;

// Re-export view functions for standalone window mode (multi-window)
pub use about::view_about_dialog_content;
pub use close_study::view_close_study_dialog_content;
pub use settings::view_settings_dialog_content;
pub use third_party::view_third_party_dialog_content;
pub use update::view_update_dialog_content;
