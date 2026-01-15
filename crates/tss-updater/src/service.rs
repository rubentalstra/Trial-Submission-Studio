//! High-level update service orchestrating the update process.
//!
//! This module provides the main `UpdateService` that coordinates
//! checking for updates, downloading, verifying, and installing.

use std::str::FromStr;

use crate::config::UpdateSettings;
use crate::download::{DownloadProgress, download_with_progress};
use crate::error::{Result, UpdateError};
use crate::github::{GitHubAsset, GitHubClient, GitHubRelease};
use crate::install::{
    extract_binary, get_target_triple, replace_current_executable, restart_application,
};
use crate::release::{ReleaseAsset, UpdateInfo};
use crate::verify::{VerificationStatus, verify_sha256};
use crate::version::Version;

/// Repository owner for GitHub releases.
pub const REPO_OWNER: &str = "rubentalstra";

/// Repository name for GitHub releases.
pub const REPO_NAME: &str = "Trial-Submission-Studio";

/// Current application version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// High-level update service.
///
/// This struct provides a stateless interface for update operations.
/// All methods take configuration as parameters, making it easy to use
/// from different contexts.
pub struct UpdateService;

impl UpdateService {
    /// Checks for available updates.
    ///
    /// # Arguments
    /// * `settings` - Update settings including channel preference
    ///
    /// # Returns
    /// * `Ok(Some(UpdateInfo))` - An update is available
    /// * `Ok(None)` - No update available (already on latest version)
    /// * `Err(UpdateError)` - Failed to check for updates
    pub async fn check_for_update(settings: &UpdateSettings) -> Result<Option<UpdateInfo>> {
        tracing::info!("Checking for updates (current version: {})", VERSION);

        let client = GitHubClient::new(REPO_OWNER, REPO_NAME)?;
        let release = client.get_latest_release().await?;

        // Skip draft releases
        if release.draft {
            tracing::debug!("Skipping draft release");
            return Ok(None);
        }

        // Parse versions for comparison
        let current_version = Version::from_str(VERSION)
            .map_err(|_| UpdateError::InvalidVersion(VERSION.to_string()))?;

        let release_version = Version::from_tag(release.version())
            .map_err(|_| UpdateError::InvalidVersion(release.tag_name.clone()))?;

        // Check if version matches user's channel preference
        if !settings.channel.includes(&release_version) {
            tracing::debug!(
                "Skipping {} (not in {} channel)",
                release.version(),
                settings.channel.label()
            );
            return Ok(None);
        }

        // Check if update is newer
        if release_version <= current_version {
            tracing::info!(
                "No update available (current: {}, latest: {})",
                VERSION,
                release.version()
            );
            return Ok(None);
        }

        // Check if this version should be skipped
        if settings
            .skipped_version
            .as_ref()
            .is_some_and(|skipped| skipped == release.version() || skipped == &release.tag_name)
        {
            tracing::info!("Skipping version {} (user preference)", release.version());
            return Ok(None);
        }

        // Find asset for current platform
        let target = get_target_triple();
        let asset = release
            .find_asset_for_target(&target)
            .ok_or_else(|| UpdateError::NoAssetFound(target.clone()))?;

        tracing::info!(
            "Update available: {} -> {} (asset: {})",
            VERSION,
            release.version(),
            asset.name
        );

        Ok(Some(UpdateInfo::from_github_release(&release, asset)))
    }

    /// Downloads an update with progress reporting.
    ///
    /// # Arguments
    /// * `info` - Information about the update to download
    /// * `on_progress` - Callback for progress updates
    ///
    /// # Returns
    /// The downloaded and verified data as bytes.
    pub async fn download_update<F>(info: &UpdateInfo, on_progress: F) -> Result<Vec<u8>>
    where
        F: Fn(DownloadProgress) + Send + 'static,
    {
        tracing::info!("Downloading update: {}", info.version);

        let data =
            download_with_progress(&info.asset.download_url, info.asset.size, on_progress).await?;

        // Verify download if digest is available
        if let Some(ref digest) = info.asset.digest {
            tracing::info!("Verifying download with SHA256");
            verify_sha256(&data, digest)?;
        } else {
            tracing::warn!("No digest available for verification");
        }

        Ok(data)
    }

    /// Verifies downloaded data against the expected digest.
    ///
    /// # Arguments
    /// * `data` - The downloaded data
    /// * `info` - Update info containing the expected digest
    ///
    /// # Returns
    /// The verification status.
    pub fn verify_download(data: &[u8], info: &UpdateInfo) -> VerificationStatus {
        match &info.asset.digest {
            Some(digest) => match verify_sha256(data, digest) {
                Ok(()) => VerificationStatus::Verified,
                Err(UpdateError::ChecksumMismatch { expected, actual }) => {
                    VerificationStatus::Failed { expected, actual }
                }
                Err(_) => VerificationStatus::Unavailable,
            },
            None => VerificationStatus::Unavailable,
        }
    }

    /// Installs the downloaded update.
    ///
    /// This extracts the binary from the archive and replaces the current executable.
    ///
    /// # Arguments
    /// * `data` - The downloaded archive data
    /// * `info` - Update info containing asset metadata
    pub fn install_update(data: &[u8], info: &UpdateInfo) -> Result<()> {
        tracing::info!("Installing update: {}", info.version);

        // Extract binary from archive
        let binary = extract_binary(data, &info.asset.name)?;

        // Replace current executable
        replace_current_executable(&binary)?;

        tracing::info!("Update installed successfully");
        Ok(())
    }

    /// Restarts the application to apply the update.
    ///
    /// This function does not return - it exits the current process
    /// and spawns a new one.
    pub fn restart() -> Result<()> {
        restart_application()
    }

    /// Gets the current target triple for the running platform.
    #[must_use]
    pub fn get_target() -> String {
        get_target_triple()
    }
}

impl UpdateInfo {
    /// Creates an `UpdateInfo` from GitHub release data.
    fn from_github_release(release: &GitHubRelease, asset: &GitHubAsset) -> Self {
        let parsed_version = Version::from_tag(release.version()).unwrap_or_default();

        Self {
            version: release.version().to_string(),
            parsed_version,
            changelog: release.changelog().to_string(),
            asset: ReleaseAsset {
                name: asset.name.clone(),
                download_url: asset.browser_download_url.clone(),
                digest: asset.digest.clone(),
                size: asset.size,
            },
            has_verification: asset.has_verification(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        // Should be able to parse the current version
        let version = Version::from_str(VERSION);
        assert!(version.is_ok());
    }

    #[test]
    fn test_repo_constants() {
        assert_eq!(REPO_OWNER, "rubentalstra");
        assert_eq!(REPO_NAME, "Trial-Submission-Studio");
    }

    #[test]
    fn test_get_target() {
        let target = UpdateService::get_target();
        assert!(!target.is_empty());
        assert!(target.contains('-'));
    }
}
