//! GitHub release and asset types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Result, UpdateError};
use crate::version::Version;

/// A GitHub release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// The release tag name (e.g., "v1.0.0").
    pub tag_name: String,

    /// The release title/name.
    pub name: Option<String>,

    /// The release body (changelog/release notes in markdown).
    pub body: Option<String>,

    /// Whether this is a draft release.
    pub draft: bool,

    /// Whether this is a pre-release.
    pub prerelease: bool,

    /// When the release was created.
    pub created_at: DateTime<Utc>,

    /// When the release was published.
    pub published_at: Option<DateTime<Utc>>,

    /// The release assets (downloadable files).
    pub assets: Vec<Asset>,

    /// URL to the release page on GitHub.
    pub html_url: String,
}

impl Release {
    /// Get the parsed version from the tag name.
    pub fn version(&self) -> Result<Version> {
        Version::from_tag(&self.tag_name)
    }

    /// Get the release notes/changelog.
    #[must_use]
    pub fn changelog(&self) -> &str {
        self.body.as_deref().unwrap_or("")
    }

    /// Get the display name for this release.
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.tag_name)
    }

    /// Find an asset by name pattern.
    #[must_use]
    pub fn find_asset(&self, pattern: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.name.contains(pattern))
    }

    /// Find the checksum file for a given asset.
    #[must_use]
    pub fn find_checksum_asset(&self, asset: &Asset) -> Option<&Asset> {
        let checksum_name = format!("{}.sha256", asset.name);
        self.assets.iter().find(|a| a.name == checksum_name)
    }
}

/// A downloadable asset attached to a release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// The asset file name.
    pub name: String,

    /// The content type (e.g., "application/zip").
    pub content_type: String,

    /// The file size in bytes.
    pub size: u64,

    /// Number of times this asset has been downloaded.
    pub download_count: u64,

    /// Direct download URL for this asset.
    pub browser_download_url: String,

    /// When the asset was created.
    pub created_at: DateTime<Utc>,

    /// When the asset was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Asset {
    /// Check if this is a ZIP archive.
    #[must_use]
    pub fn is_zip(&self) -> bool {
        self.name.ends_with(".zip")
    }

    /// Check if this is a TAR.GZ archive.
    #[must_use]
    pub fn is_tar_gz(&self) -> bool {
        self.name.ends_with(".tar.gz")
    }

    /// Check if this is a DMG disk image.
    #[must_use]
    pub fn is_dmg(&self) -> bool {
        self.name.ends_with(".dmg")
    }

    /// Check if this is a checksum file.
    #[must_use]
    pub fn is_checksum(&self) -> bool {
        self.name.ends_with(".sha256")
    }

    /// Get the file extension.
    #[must_use]
    pub fn extension(&self) -> &str {
        if self.name.ends_with(".tar.gz") {
            "tar.gz"
        } else {
            self.name.rsplit('.').next().unwrap_or("")
        }
    }

    /// Get human-readable file size.
    #[must_use]
    pub fn human_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size >= GB {
            format!("{:.2} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.2} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.2} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} bytes", self.size)
        }
    }
}

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// The current version.
    pub current_version: Version,

    /// The new version available.
    pub new_version: Version,

    /// The release containing the update.
    pub release: Release,

    /// The asset to download for the current platform.
    pub asset: Asset,

    /// The checksum asset for verification (if available).
    pub checksum_asset: Option<Asset>,
}

impl UpdateInfo {
    /// Create update info from release and asset.
    pub fn new(
        current_version: Version,
        release: Release,
        asset: Asset,
        checksum_asset: Option<Asset>,
    ) -> Result<Self> {
        let new_version = release.version()?;

        if new_version <= current_version {
            return Err(UpdateError::AlreadyUpToDate(current_version.to_string()));
        }

        Ok(Self {
            current_version,
            new_version,
            release,
            asset,
            checksum_asset,
        })
    }

    /// Get the changelog for this update.
    #[must_use]
    pub fn changelog(&self) -> &str {
        self.release.changelog()
    }

    /// Get the download URL.
    #[must_use]
    pub fn download_url(&self) -> &str {
        &self.asset.browser_download_url
    }

    /// Get the download size in bytes.
    #[must_use]
    pub fn download_size(&self) -> u64 {
        self.asset.size
    }

    /// Get human-readable download size.
    #[must_use]
    pub fn human_download_size(&self) -> String {
        self.asset.human_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_extension() {
        let zip = Asset {
            name: "app-v1.0.0-macos.zip".to_string(),
            content_type: "application/zip".to_string(),
            size: 1024,
            download_count: 0,
            browser_download_url: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(zip.extension(), "zip");
        assert!(zip.is_zip());

        let tar = Asset {
            name: "app-v1.0.0-linux.tar.gz".to_string(),
            content_type: "application/gzip".to_string(),
            size: 1024,
            download_count: 0,
            browser_download_url: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(tar.extension(), "tar.gz");
        assert!(tar.is_tar_gz());
    }

    #[test]
    fn test_human_size() {
        let asset = |size| Asset {
            name: String::new(),
            content_type: String::new(),
            size,
            download_count: 0,
            browser_download_url: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(asset(500).human_size(), "500 bytes");
        assert_eq!(asset(1536).human_size(), "1.50 KB");
        assert_eq!(asset(2_621_440).human_size(), "2.50 MB");
        assert_eq!(asset(1_610_612_736).human_size(), "1.50 GB");
    }
}
