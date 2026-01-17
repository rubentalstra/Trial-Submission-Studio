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
//! - Streaming progress reporting during downloads
//!
//! # Architecture
//!
//! This crate provides simple async functions that integrate with Iced's native patterns:
//!
//! - `check_for_update()` - One-shot async operation, use with `Task::perform()`
//! - `download_with_data()` - Streaming download, use with `Task::run()`
//! - `verify_sha256()` - Sync verification function
//! - `extract_archive()` - Sync extraction function
//! - `install_and_restart()` - Sync installation function
//!
//! The GUI handles state management through its own `UpdateState` enum,
//! using Iced's message/update loop as the state machine.
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
//! use tss_updater::{UpdateSettings, check_for_update, download_with_data, DownloadStreamItem};
//! use futures_util::StreamExt;
//!
//! async fn check_updates() -> tss_updater::Result<()> {
//!     let settings = UpdateSettings::default();
//!
//!     // Check for updates
//!     if let Some(info) = check_for_update(&settings).await? {
//!         println!("Update available: {}", info.version);
//!
//!         // Download with progress (url is passed as owned String)
//!         let url = info.asset.download_url.clone();
//!         let mut stream = std::pin::pin!(download_with_data(url, info.asset.size));
//!         while let Some(result) = stream.next().await {
//!             match result? {
//!                 DownloadStreamItem::Progress(p) => {
//!                     println!("Downloaded: {}%", p.percentage());
//!                 }
//!                 DownloadStreamItem::Complete(result) => {
//!                     println!("Download complete: {} bytes", result.data.len());
//!                 }
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Core modules
pub mod config;
pub mod error;
pub mod release;
pub mod version;

// Individual steps
pub mod steps;

// GitHub API
pub mod github;

// Platform-specific installation
pub mod platform;

// Re-export main types for convenience
pub use config::{UpdateChannel, UpdateSettings};
pub use error::{Result, SuggestedAction, UpdateError};
pub use release::{ReleaseAsset, UpdateInfo};
pub use version::{PreRelease, Version};

// Re-export step functions and types
pub use steps::check::check_for_update;
pub use steps::download::{
    DownloadProgress, DownloadResult, DownloadStreamItem, download_simple, download_stream,
    download_with_data, format_bytes, format_speed,
};
pub use steps::extract::{ArchiveType, detect_archive_type, extract_archive};
pub use steps::signature::verify_signature;
pub use steps::verify::{verify_download, verify_sha256};

/// Current version of the application.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub repository owner.
pub const REPO_OWNER: &str = "rubentalstra";

/// GitHub repository name.
pub const REPO_NAME: &str = "Trial-Submission-Studio";

/// Verification status for downloaded updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationStatus {
    /// SHA256 hash matched.
    Verified,
    /// SHA256 hash did not match.
    Failed {
        /// Expected hash.
        expected: String,
        /// Actual hash.
        actual: String,
    },
    /// No digest available from GitHub (verification skipped).
    Unavailable,
}

/// Install the update and restart the application.
///
/// On macOS, this spawns a helper process to swap the app bundle.
/// On other platforms, this replaces the current binary and restarts.
pub fn install_and_restart(data: &[u8], info: &UpdateInfo) -> Result<()> {
    platform::install_and_restart(data, info)
}

/// Restart the application.
pub fn restart() -> Result<()> {
    steps::install::restart_application()
}

/// Service for update operations.
///
/// This is a facade that provides a simpler API for common update operations.
/// Kept for backwards compatibility with existing code.
pub struct UpdateService;

impl UpdateService {
    /// Check for available updates.
    ///
    /// Returns `Some(UpdateInfo)` if an update is available, `None` if up to date.
    pub async fn check_for_update(settings: &UpdateSettings) -> Result<Option<UpdateInfo>> {
        check_for_update(settings).await
    }

    /// Download an update without progress reporting.
    ///
    /// For progress reporting, use `download_with_data()` directly.
    pub async fn download_update(
        info: &UpdateInfo,
        _progress: impl Fn(f64) + Send + Sync,
    ) -> Result<Vec<u8>> {
        download_simple(&info.asset.download_url).await
    }

    /// Verify the downloaded update data.
    pub fn verify_download(data: &[u8], info: &UpdateInfo) -> VerificationStatus {
        match &info.asset.digest {
            Some(expected_digest) => match verify_sha256(data, expected_digest) {
                Ok(_) => VerificationStatus::Verified,
                Err(UpdateError::ChecksumMismatch { expected, actual }) => {
                    VerificationStatus::Failed { expected, actual }
                }
                Err(_) => VerificationStatus::Unavailable,
            },
            None => VerificationStatus::Unavailable,
        }
    }

    /// Install the downloaded update (writes to temp location).
    ///
    /// On macOS, this extracts the app bundle. On other platforms, this extracts
    /// the binary. This is the same as `install_and_restart` - kept for API compatibility.
    pub fn install_update(data: &[u8], info: &UpdateInfo) -> Result<()> {
        install_and_restart(data, info)
    }

    /// Install the update and restart the application.
    ///
    /// On macOS, this spawns a helper process to swap the app bundle.
    /// On other platforms, this replaces the current binary and restarts.
    pub fn install_and_restart(data: &[u8], info: &UpdateInfo) -> Result<()> {
        install_and_restart(data, info)
    }

    /// Restart the application.
    pub fn restart() -> Result<()> {
        restart()
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
}
