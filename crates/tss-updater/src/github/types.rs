//! GitHub API types.

use serde::Deserialize;

/// Raw release data from the GitHub API.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    /// The release tag name (e.g., "v0.1.0").
    pub tag_name: String,

    /// The release title/name.
    pub name: Option<String>,

    /// Release notes/body in markdown format.
    pub body: Option<String>,

    /// Whether this is a pre-release.
    pub prerelease: bool,

    /// Whether this is a draft release.
    pub draft: bool,

    /// Release assets (binaries, archives, etc.).
    pub assets: Vec<GitHubAsset>,

    /// HTML URL to the release page.
    pub html_url: String,

    /// Publication timestamp.
    pub published_at: Option<String>,
}

impl GitHubRelease {
    /// Returns the version string without the "v" prefix if present.
    #[must_use]
    pub fn version(&self) -> &str {
        self.tag_name.strip_prefix('v').unwrap_or(&self.tag_name)
    }

    /// Returns the changelog/release notes.
    #[must_use]
    pub fn changelog(&self) -> &str {
        self.body.as_deref().unwrap_or("")
    }

    /// Finds an asset matching the given target triple.
    ///
    /// Prefers tar.gz for Unix-like systems and zip for Windows.
    /// Only returns fully uploaded assets (state == "uploaded").
    ///
    /// # Arguments
    /// * `target` - The target triple (e.g., "x86_64-apple-darwin")
    #[must_use]
    pub fn find_asset_for_target(&self, target: &str) -> Option<&GitHubAsset> {
        let target_lower = target.to_lowercase();
        let is_windows = target_lower.contains("windows");

        // Filter assets that match the target and are fully uploaded
        let matching: Vec<_> = self
            .assets
            .iter()
            .filter(|asset| {
                let name = asset.name.to_lowercase();
                // Must contain the target triple AND be fully uploaded
                name.contains(&target_lower) && asset.is_uploaded()
            })
            .collect();

        if matching.is_empty() {
            return None;
        }

        // Prefer the right archive format for the platform
        // macOS: DMG preserves code signatures perfectly
        // Linux: tar.gz is standard
        // Windows: zip is standard
        let is_macos = target_lower.contains("apple-darwin");
        let preferred_ext = if is_macos {
            ".dmg"
        } else if is_windows {
            ".zip"
        } else {
            ".tar.gz"
        };

        let preferred_asset = matching
            .iter()
            .find(|a| a.name.to_lowercase().ends_with(preferred_ext));

        // For macOS, DMG is required (preserves code signatures).
        // For other platforms, fall back to any matching asset if preferred format isn't found.
        if is_macos {
            preferred_asset.copied()
        } else {
            preferred_asset.or(matching.first()).copied()
        }
    }
}

/// Release asset data from the GitHub API.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAsset {
    /// Asset filename (e.g., "trial-submission-studio-v0.1.0-x86_64-apple-darwin.tar.gz").
    pub name: String,

    /// Direct download URL.
    pub browser_download_url: String,

    /// Upload state: "uploaded" (complete) or "open" (still uploading).
    pub state: String,

    /// SHA256 digest (format: "sha256:...").
    /// Always present in API response but may be null if not yet computed.
    pub digest: Option<String>,

    /// File size in bytes.
    pub size: u64,

    /// Content type (e.g., "application/gzip").
    pub content_type: String,

    /// Download count.
    pub download_count: u64,

    /// Creation timestamp.
    pub created_at: String,

    /// Last update timestamp.
    pub updated_at: String,
}

impl GitHubAsset {
    /// Returns whether this asset is fully uploaded and ready for download.
    #[must_use]
    pub fn is_uploaded(&self) -> bool {
        self.state == "uploaded"
    }

    /// Returns the SHA256 hash from the digest field, if available.
    ///
    /// The digest field has the format "sha256:...".
    #[must_use]
    pub fn sha256(&self) -> Option<&str> {
        self.digest
            .as_deref()
            .and_then(|d| d.strip_prefix("sha256:"))
    }

    /// Returns whether this asset has verification available.
    #[must_use]
    pub fn has_verification(&self) -> bool {
        self.sha256().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let release = GitHubRelease {
            tag_name: "v0.1.0".to_string(),
            name: Some("Release 0.1.0".to_string()),
            body: Some("## Changes\n- Initial release".to_string()),
            prerelease: false,
            draft: false,
            assets: vec![],
            html_url: "https://github.com/test/test/releases/tag/v0.1.0".to_string(),
            published_at: None,
        };

        assert_eq!(release.version(), "0.1.0");
    }

    #[test]
    fn test_asset_sha256() {
        let asset = GitHubAsset {
            name: "test.tar.gz".to_string(),
            browser_download_url: "https://example.com/test.tar.gz".to_string(),
            state: "uploaded".to_string(),
            digest: Some("sha256:abc123def456".to_string()),
            size: 1024,
            content_type: "application/gzip".to_string(),
            download_count: 0,
            created_at: String::new(),
            updated_at: String::new(),
        };

        assert_eq!(asset.sha256(), Some("abc123def456"));
        assert!(asset.has_verification());
        assert!(asset.is_uploaded());
    }

    #[test]
    fn test_find_asset_for_target() {
        let release = GitHubRelease {
            tag_name: "v0.1.0".to_string(),
            name: None,
            body: None,
            prerelease: false,
            draft: false,
            assets: vec![
                GitHubAsset {
                    name: "trial-submission-studio-v0.1.0-x86_64-apple-darwin.dmg".to_string(),
                    browser_download_url: String::new(),
                    state: "uploaded".to_string(),
                    digest: None,
                    size: 1024,
                    content_type: "application/x-apple-diskimage".to_string(),
                    download_count: 0,
                    created_at: String::new(),
                    updated_at: String::new(),
                },
                GitHubAsset {
                    name: "trial-submission-studio-v0.1.0-x86_64-pc-windows-msvc.zip".to_string(),
                    browser_download_url: String::new(),
                    state: "uploaded".to_string(),
                    digest: None,
                    size: 2048,
                    content_type: "application/zip".to_string(),
                    download_count: 0,
                    created_at: String::new(),
                    updated_at: String::new(),
                },
            ],
            html_url: String::new(),
            published_at: None,
        };

        let asset = release.find_asset_for_target("x86_64-apple-darwin");
        assert!(asset.is_some());
        assert!(asset.unwrap().name.contains("apple-darwin"));
        assert!(asset.unwrap().name.ends_with(".dmg"));

        let asset = release.find_asset_for_target("x86_64-pc-windows-msvc");
        assert!(asset.is_some());
        assert!(asset.unwrap().name.contains("windows"));
    }
}
