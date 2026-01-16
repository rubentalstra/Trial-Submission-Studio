//! Auto-update system for Trial Submission Studio.
//!
//! This crate provides functionality for checking for updates from GitHub releases,
//! downloading them with progress reporting, verifying with SHA256, and installing them.
//!
//! # Overview
//!
//! The update system uses GitHub Releases as its source. It supports:
//!
//! - Semantic versioning with pre-release tags (alpha, beta, rc)
//! - Configurable update channels (stable, beta)
//! - Automatic platform detection (macOS, Windows, Linux on x64 and ARM64)
//! - SHA256 verification using GitHub's built-in digest feature
//! - Progress reporting during downloads
//! - In-place binary replacement using `self_replace`
//!
//! # Supported Platforms
//!
//! - macOS: x86_64 (Intel) and aarch64 (Apple Silicon)
//! - Windows: x86_64 and aarch64
//! - Linux: x86_64 and aarch64
//!
//! # Example
//!
//! ```no_run
//! use tss_updater::{UpdateService, UpdateSettings};
//!
//! async fn check_updates() -> tss_updater::Result<()> {
//!     let settings = UpdateSettings::default();
//!
//!     // Check for updates
//!     if let Some(update) = UpdateService::check_for_update(&settings).await? {
//!         println!("Update available: {}", update.version);
//!         println!("Changelog: {}", update.changelog);
//!         println!("Download size: {} bytes", update.asset.size);
//!
//!         // Download with progress
//!         let data = UpdateService::download_update(&update, |progress| {
//!             println!("Downloaded: {:.1}%", progress.fraction * 100.0);
//!         }).await?;
//!
//!         // Verify (automatically done during download if digest available)
//!         let status = UpdateService::verify_download(&data, &update);
//!         println!("Verification: {:?}", status);
//!
//!         // Install and restart
//!         UpdateService::install_update(&data, &update)?;
//!         UpdateService::restart()?;
//!     }
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Public modules
pub mod config;
pub mod download;
pub mod error;
pub mod github;
pub mod install;
pub mod platform;
pub mod release;
pub mod service;
pub mod verify;
pub mod version;

// Re-export main types for convenience
pub use config::{UpdateChannel, UpdateSettings};
pub use download::DownloadProgress;
pub use error::{Result, UpdateError};
pub use release::{ReleaseAsset, UpdateInfo};
pub use service::{REPO_NAME, REPO_OWNER, UpdateService, VERSION};
pub use verify::VerificationStatus;
pub use version::{PreRelease, Version};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_repo_constants() {
        assert_eq!(REPO_OWNER, "rubentalstra");
        assert_eq!(REPO_NAME, "Trial-Submission-Studio");
    }
}
