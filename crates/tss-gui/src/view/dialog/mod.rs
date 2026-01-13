//! Dialog views for Trial Submission Studio.
//!
//! Modal dialogs for application-level interactions:
//! - About: Application info, version, links
//! - Settings: User preferences with master-detail layout
//! - Third-party: Open source license acknowledgments
//! - Update: Check for and install updates

pub mod about;
pub mod settings;
pub mod third_party;
pub mod update;

// Re-export view functions
pub use about::view_about_dialog;
pub use settings::view_settings_dialog;
pub use third_party::view_third_party_dialog;
pub use update::view_update_dialog;
