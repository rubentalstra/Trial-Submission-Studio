//! Release information types.
//!
//! This module defines the types used to represent release information
//! from GitHub, including assets with SHA256 digests.

use crate::version::Version;

/// Information about a release asset (downloadable file).
#[derive(Debug, Clone)]
pub struct ReleaseAsset {
    /// Asset filename (e.g., "trial-submission-studio-v0.1.0-x86_64-apple-darwin.tar.gz").
    pub name: String,

    /// Direct download URL.
    pub download_url: String,

    /// SHA256 digest from GitHub (format: "sha256:...").
    /// This is `None` if the digest is not available.
    pub digest: Option<String>,

    /// File size in bytes.
    pub size: u64,
}

impl ReleaseAsset {
    /// Creates a new release asset.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        download_url: impl Into<String>,
        digest: Option<String>,
        size: u64,
    ) -> Self {
        Self {
            name: name.into(),
            download_url: download_url.into(),
            digest,
            size,
        }
    }

    /// Returns the SHA256 hash without the "sha256:" prefix.
    #[must_use]
    pub fn sha256(&self) -> Option<&str> {
        self.digest
            .as_deref()
            .and_then(|d| d.strip_prefix("sha256:"))
    }

    /// Returns whether this asset has SHA256 verification available.
    #[must_use]
    pub fn has_verification(&self) -> bool {
        self.sha256().is_some()
    }

    /// Returns the file size in human-readable form.
    #[must_use]
    pub fn size_display(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size >= GB {
            format!("{:.1} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.1} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.1} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} B", self.size)
        }
    }
}

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Version string (e.g., "0.1.0" or "0.1.0-beta.1").
    pub version: String,

    /// Parsed semantic version for comparison.
    pub parsed_version: Version,

    /// Release notes/changelog in markdown format.
    pub changelog: String,

    /// The matching asset for the current platform.
    pub asset: ReleaseAsset,

    /// Whether SHA256 verification is available for this release.
    pub has_verification: bool,
}

impl UpdateInfo {
    /// Creates a new update info.
    #[must_use]
    pub fn new(
        version: impl Into<String>,
        parsed_version: Version,
        changelog: impl Into<String>,
        asset: ReleaseAsset,
    ) -> Self {
        let has_verification = asset.has_verification();
        Self {
            version: version.into(),
            parsed_version,
            changelog: changelog.into(),
            asset,
            has_verification,
        }
    }

    /// Returns the version string for display (without "v" prefix).
    #[must_use]
    pub fn version_display(&self) -> &str {
        self.version.strip_prefix('v').unwrap_or(&self.version)
    }
}

impl Default for UpdateInfo {
    fn default() -> Self {
        Self {
            version: String::new(),
            parsed_version: Version::default(),
            changelog: String::new(),
            asset: ReleaseAsset {
                name: String::new(),
                download_url: String::new(),
                digest: None,
                size: 0,
            },
            has_verification: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_asset_sha256() {
        let asset = ReleaseAsset::new(
            "test.tar.gz",
            "https://example.com/test.tar.gz",
            Some("sha256:abc123def456".to_string()),
            1024,
        );

        assert_eq!(asset.sha256(), Some("abc123def456"));
        assert!(asset.has_verification());
    }

    #[test]
    fn test_release_asset_no_digest() {
        let asset = ReleaseAsset::new("test.tar.gz", "https://example.com/test.tar.gz", None, 1024);

        assert_eq!(asset.sha256(), None);
        assert!(!asset.has_verification());
    }

    #[test]
    fn test_release_asset_size_display() {
        let asset = ReleaseAsset::new("test.tar.gz", "", None, 52_428_800);
        assert_eq!(asset.size_display(), "50.0 MB");

        let asset = ReleaseAsset::new("test.tar.gz", "", None, 1024);
        assert_eq!(asset.size_display(), "1.0 KB");

        let asset = ReleaseAsset::new("test.tar.gz", "", None, 512);
        assert_eq!(asset.size_display(), "512 B");
    }

    #[test]
    fn test_update_info_version_display() {
        let info = UpdateInfo::new(
            "v0.1.0",
            Version::default(),
            "Changes",
            ReleaseAsset::new("test.tar.gz", "", None, 0),
        );

        assert_eq!(info.version_display(), "0.1.0");

        let info = UpdateInfo::new(
            "0.1.0",
            Version::default(),
            "Changes",
            ReleaseAsset::new("test.tar.gz", "", None, 0),
        );

        assert_eq!(info.version_display(), "0.1.0");
    }
}
