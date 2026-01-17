//! Check for available updates.

use std::str::FromStr;

use crate::config::UpdateSettings;
use crate::error::{Result, UpdateError};
use crate::github::client::GitHubClient;
use crate::github::types::GitHubRelease;
use crate::release::{ReleaseAsset, UpdateInfo};
use crate::version::Version;
use crate::{REPO_NAME, REPO_OWNER, VERSION};

/// Checks for available updates from GitHub releases.
///
/// Returns `Some(UpdateInfo)` if an update is available, `None` if already up to date.
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
    let current_version =
        Version::from_str(VERSION).map_err(|_| UpdateError::InvalidVersion(VERSION.to_string()))?;

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
    if settings.should_skip_version(release.version()) {
        tracing::info!("Skipping version {} (user preference)", release.version());
        return Ok(None);
    }

    // Find asset for current platform
    let target = get_target_triple();
    let asset = release.find_asset_for_target(&target).ok_or_else(|| {
        // On macOS, DMG is required (preserves code signatures).
        if target.contains("apple-darwin") {
            UpdateError::NoCompatibleAsset
        } else {
            UpdateError::NoAssetFound(target.clone())
        }
    })?;

    tracing::info!(
        "Update available: {} -> {} (asset: {})",
        VERSION,
        release.version(),
        asset.name
    );

    Ok(Some(create_update_info(&release, asset)))
}

/// Gets the current target triple for the running system.
fn get_target_triple() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    match (os, arch) {
        ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
        ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
        ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),
        ("windows", "aarch64") => "aarch64-pc-windows-msvc".to_string(),
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu".to_string(),
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu".to_string(),
        _ => format!("{}-{}", arch, os),
    }
}

/// Creates UpdateInfo from GitHub release data.
fn create_update_info(
    release: &GitHubRelease,
    asset: &crate::github::types::GitHubAsset,
) -> UpdateInfo {
    let parsed_version = Version::from_tag(release.version()).unwrap_or_default();

    UpdateInfo {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_target_triple() {
        let target = get_target_triple();
        assert!(!target.is_empty());
        assert!(target.contains('-'));
    }
}
