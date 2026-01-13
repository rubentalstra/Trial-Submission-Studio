//! Dialog views for Trial Submission Studio.
//!
//! Modal dialogs for application-level interactions:
//! - About: Application info, version, links
//! - Settings: User preferences with master-detail layout
//! - Third-party: Open source license acknowledgments
//! - Update: Check for and install updates
//!
//! Each dialog has two view functions:
//! - `view_*_dialog()` - For overlay/modal mode (with backdrop)
//! - `view_*_dialog_content()` - For standalone window mode (multi-window)

pub mod about;
pub mod settings;
pub mod third_party;
pub mod update;

// Re-export view functions for overlay/modal mode
pub use about::view_about_dialog;
pub use settings::view_settings_dialog;
pub use third_party::view_third_party_dialog;
pub use update::view_update_dialog;

// Re-export view functions for standalone window mode (multi-window)
pub use about::view_about_dialog_content;
pub use settings::view_settings_dialog_content;
pub use third_party::view_third_party_dialog_content;
pub use update::view_update_dialog_content;
