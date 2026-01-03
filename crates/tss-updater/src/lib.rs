//! Auto-update system for Trial Submission Studio.
//!
//! This crate provides functionality for checking for updates, downloading them,
//! verifying their integrity, and seamlessly installing them.
//!
//! # Overview
//!
//! The update system uses GitHub Releases as its source. It supports:
//!
//! - Semantic versioning with pre-release tags (alpha, beta, rc)
//! - Configurable update channels (stable, beta)
//! - SHA256 checksum verification
//! - Cryptographic signature verification (via zipsign)
//! - Download progress reporting
//! - Seamless in-place updates
//!
//! # Example
//!
//! ```no_run
//! use tss_updater::{
//!     GitHubClient, UpdateSettings, UpdateChannel, Platform, Version,
//!     Downloader, checksum, installer,
//! };
//!
//! // Check for updates
//! let client = GitHubClient::new().unwrap();
//! let current = Version::current();
//! let platform = Platform::current();
//!
//! if let Some(update) = client.check_for_update(&current, UpdateChannel::Stable, &platform).unwrap() {
//!     println!("Update available: {} -> {}", update.current_version, update.new_version);
//!     println!("Changelog: {}", update.changelog());
//!
//!     // Download the update
//!     let downloader = Downloader::new().unwrap();
//!     let path = downloader.download(&update, |progress| {
//!         println!("{}% ({}/{})",
//!             progress.percentage(),
//!             progress.human_downloaded(),
//!             progress.human_total()
//!         );
//!     }).unwrap();
//!
//!     // Verify checksum
//!     if let Some(ref checksum_asset) = update.checksum_asset {
//!         let expected = client.fetch_checksum(&checksum_asset.browser_download_url).unwrap();
//!         checksum::verify_sha256(&path, &expected).unwrap();
//!     }
//!
//!     // Install and restart
//!     installer::install_from_archive(&path, &update).unwrap();
//!     installer::restart_application().unwrap();
//! }
//! ```
//!
//! # Modules
//!
//! - [`error`] - Error types for update operations
//! - [`version`] - Semantic version parsing and comparison
//! - [`config`] - Update settings and configuration
//! - [`release`] - GitHub release and asset types
//! - [`platform`] - Platform detection and asset matching
//! - [`client`] - GitHub API client
//! - [`download`] - Download with progress reporting
//! - [`checksum`] - SHA256 verification
//! - [`installer`] - Update installation

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod checksum;
pub mod client;
pub mod config;
pub mod download;
pub mod error;
pub mod installer;
pub mod platform;
pub mod release;
pub mod version;

// Re-export main types for convenience
pub use checksum::{compute_file_sha256, verify_sha256};
pub use client::GitHubClient;
pub use config::{UpdateChannel, UpdateCheckFrequency, UpdateSettings};
pub use download::{DownloadProgress, Downloader};
pub use error::{Result, UpdateError};
pub use installer::{install_from_archive, install_update, restart_application};
pub use platform::{Arch, Os, Platform};
pub use release::{Asset, Release, UpdateInfo};
pub use version::{PreRelease, Version};

/// The current version of this crate (from Cargo.toml).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub repository owner.
pub const REPO_OWNER: &str = "rubentalstra";

/// GitHub repository name.
pub const REPO_NAME: &str = "Trial-Submission-Studio";

/// High-level update service that coordinates checking, downloading, and installing updates.
pub struct UpdateService {
    /// GitHub API client.
    client: GitHubClient,
    /// Downloader for release assets.
    downloader: Downloader,
    /// Current platform.
    platform: Platform,
}

impl UpdateService {
    /// Create a new update service.
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: GitHubClient::new()?,
            downloader: Downloader::new()?,
            platform: Platform::current(),
        })
    }

    /// Get a reference to the GitHub client.
    #[must_use]
    pub fn client(&self) -> &GitHubClient {
        &self.client
    }

    /// Get a reference to the downloader.
    #[must_use]
    pub fn downloader(&self) -> &Downloader {
        &self.downloader
    }

    /// Check for available updates.
    ///
    /// Returns `Some(UpdateInfo)` if an update is available, `None` otherwise.
    pub fn check_for_update(
        &self,
        settings: &UpdateSettings,
    ) -> Result<Option<UpdateInfo>> {
        let current = Version::current();
        self.client
            .check_for_update(&current, settings.channel, &self.platform)
    }

    /// Check for updates, forcing a fresh API call (bypassing cache).
    pub fn check_for_update_fresh(
        &self,
        settings: &UpdateSettings,
    ) -> Result<Option<UpdateInfo>> {
        let current = Version::current();
        self.client
            .check_for_update_fresh(&current, settings.channel, &self.platform)
    }

    /// Download an update and return the path to the downloaded file.
    ///
    /// The progress callback is called periodically with download progress.
    pub fn download_update<F>(
        &self,
        update: &UpdateInfo,
        progress_callback: F,
    ) -> Result<std::path::PathBuf>
    where
        F: Fn(DownloadProgress),
    {
        self.downloader.download(update, progress_callback)
    }

    /// Verify the downloaded update's checksum.
    pub fn verify_update(&self, path: &std::path::Path, update: &UpdateInfo) -> Result<()> {
        if let Some(ref checksum_asset) = update.checksum_asset {
            let expected = self.client.fetch_checksum(&checksum_asset.browser_download_url)?;
            verify_sha256(path, &expected)?;
        }
        Ok(())
    }

    /// Install the update and restart the application.
    ///
    /// This function does not return on success - the application is restarted.
    pub fn install_and_restart(
        &self,
        path: &std::path::Path,
        update: &UpdateInfo,
    ) -> Result<()> {
        install_from_archive(path, update)?;
        restart_application()?;
        Ok(())
    }

    /// Get a cancellation handle for the downloader.
    ///
    /// This can be used to cancel an ongoing download from another thread.
    #[must_use]
    pub fn cancellation_handle(&self) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        self.downloader.cancellation_handle()
    }

    /// Cancel any ongoing download.
    pub fn cancel_download(&self) {
        self.downloader.cancel();
    }
}

impl Default for UpdateService {
    fn default() -> Self {
        Self::new().expect("Failed to create UpdateService")
    }
}

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

    #[test]
    fn test_update_service_creation() {
        // This may fail if there's no network, but it shouldn't panic
        let service = UpdateService::new();
        assert!(service.is_ok());
    }
}
