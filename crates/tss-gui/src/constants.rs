//! Application identity constants.
//!
//! Centralized constants for application metadata used across the codebase.
//! This avoids magic strings scattered throughout the application.

/// Application display name.
pub const APP_NAME: &str = "Trial Submission Studio";

/// Application identifier (reverse domain notation).
pub const APP_ID: &str = "com.trialsubmissionstudio.app";

/// Application author.
pub const APP_AUTHOR: &str = "Ruben Talstra";

/// Application version from Cargo.toml.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application website URL.
pub const APP_WEBSITE: &str = "https://rubentalstra.github.io/Trial-Submission-Studio/";

/// GitHub repository URL.
pub const APP_GITHUB: &str = "https://github.com/rubentalstra/trial-submission-studio";

/// Application description.
pub const APP_DESCRIPTION: &str = "Transform clinical trial data into FDA-compliant CDISC formats";

/// Copyright notice.
pub fn copyright() -> String {
    let year = chrono::Utc::now().format("%Y");
    format!("Copyright {} {}", year, APP_AUTHOR)
}
