//! GitHub API client for fetching release information.
//!
//! This module provides a client for interacting with the GitHub Releases API
//! to fetch release metadata. SHA256 digests are automatically provided by GitHub
//! for all release assets (since June 2025).

use crate::error::{Result, UpdateError};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;

/// GitHub API base URL.
const GITHUB_API_URL: &str = "https://api.github.com";

/// User agent string for API requests.
const USER_AGENT_VALUE: &str = concat!(
    "trial-submission-studio/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/rubentalstra/Trial-Submission-Studio)"
);

/// GitHub API client for fetching release information.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: reqwest::Client,
    owner: String,
    repo: String,
}

impl GitHubClient {
    /// Creates a new GitHub client for the specified repository.
    ///
    /// # Arguments
    /// * `owner` - The repository owner (e.g., "rubentalstra")
    /// * `repo` - The repository name (e.g., "Trial-Submission-Studio")
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| UpdateError::Network(format!("failed to create HTTP client: {e}")))?;

        Ok(Self {
            client,
            owner: owner.into(),
            repo: repo.into(),
        })
    }

    /// Fetches the latest release from GitHub.
    ///
    /// Returns the release information including all assets with their digests
    /// automatically populated by GitHub (since June 2025).
    pub async fn get_latest_release(&self) -> Result<GitHubRelease> {
        let url = format!(
            "{}/repos/{}/{}/releases/latest",
            GITHUB_API_URL, self.owner, self.repo
        );

        tracing::debug!("Fetching latest release from {}", url);

        let response = self.client.get(&url).send().await?;
        let release = self.handle_response(response).await?;

        Ok(release)
    }

    /// Fetches a specific release by tag name.
    ///
    /// # Arguments
    /// * `tag` - The release tag (e.g., "v0.1.0")
    pub async fn get_release_by_tag(&self, tag: &str) -> Result<GitHubRelease> {
        let url = format!(
            "{}/repos/{}/{}/releases/tags/{}",
            GITHUB_API_URL, self.owner, self.repo, tag
        );

        tracing::debug!("Fetching release by tag from {}", url);

        let response = self.client.get(&url).send().await?;
        let release = self.handle_response(response).await?;

        Ok(release)
    }

    /// Handles the HTTP response, checking for errors and parsing JSON.
    async fn handle_response(&self, response: reqwest::Response) -> Result<GitHubRelease> {
        let status = response.status();

        // Check for rate limiting
        if status == reqwest::StatusCode::FORBIDDEN
            && response
                .headers()
                .get("x-ratelimit-remaining")
                .is_some_and(|remaining| remaining.to_str().unwrap_or("1") == "0")
        {
            let retry_after = response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(|reset| {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    reset.saturating_sub(now)
                })
                .unwrap_or(60);

            return Err(UpdateError::RateLimited { retry_after });
        }

        // Check for not found (no releases)
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(UpdateError::Network(
                "No releases found for this repository".to_string(),
            ));
        }

        // Check for other errors
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(UpdateError::Network(format!(
                "GitHub API error ({}): {}",
                status, body
            )));
        }

        // Parse the response
        let release: GitHubRelease = response.json().await?;

        Ok(release)
    }
}

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
        let preferred_ext = if is_windows { ".zip" } else { ".tar.gz" };

        matching
            .iter()
            .find(|a| a.name.to_lowercase().ends_with(preferred_ext))
            .or(matching.first())
            .copied()
    }
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
    fn test_version_without_prefix() {
        let release = GitHubRelease {
            tag_name: "0.1.0".to_string(),
            name: None,
            body: None,
            prerelease: false,
            draft: false,
            assets: vec![],
            html_url: String::new(),
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
    fn test_asset_no_digest() {
        let asset = GitHubAsset {
            name: "test.tar.gz".to_string(),
            browser_download_url: "https://example.com/test.tar.gz".to_string(),
            state: "uploaded".to_string(),
            digest: None,
            size: 1024,
            content_type: "application/gzip".to_string(),
            download_count: 0,
            created_at: String::new(),
            updated_at: String::new(),
        };

        assert_eq!(asset.sha256(), None);
        assert!(!asset.has_verification());
    }

    #[test]
    fn test_asset_upload_state() {
        let uploaded = GitHubAsset {
            name: "test.tar.gz".to_string(),
            browser_download_url: String::new(),
            state: "uploaded".to_string(),
            digest: None,
            size: 1024,
            content_type: "application/gzip".to_string(),
            download_count: 0,
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert!(uploaded.is_uploaded());

        let open = GitHubAsset {
            name: "test.tar.gz".to_string(),
            browser_download_url: String::new(),
            state: "open".to_string(),
            digest: None,
            size: 1024,
            content_type: "application/gzip".to_string(),
            download_count: 0,
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert!(!open.is_uploaded());
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
                    name: "trial-submission-studio-v0.1.0-x86_64-apple-darwin.tar.gz".to_string(),
                    browser_download_url: String::new(),
                    state: "uploaded".to_string(),
                    digest: None,
                    size: 1024,
                    content_type: "application/gzip".to_string(),
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
        assert!(asset.unwrap().name.ends_with(".tar.gz"));

        let asset = release.find_asset_for_target("x86_64-pc-windows-msvc");
        assert!(asset.is_some());
        assert!(asset.unwrap().name.contains("windows"));
        assert!(asset.unwrap().name.ends_with(".zip"));

        let asset = release.find_asset_for_target("aarch64-unknown-linux-gnu");
        assert!(asset.is_none());
    }

    #[test]
    fn test_find_asset_skips_incomplete_uploads() {
        let release = GitHubRelease {
            tag_name: "v0.1.0".to_string(),
            name: None,
            body: None,
            prerelease: false,
            draft: false,
            assets: vec![GitHubAsset {
                name: "app-v0.1.0-x86_64-apple-darwin.tar.gz".to_string(),
                browser_download_url: String::new(),
                state: "open".to_string(), // Still uploading
                digest: None,
                size: 1024,
                content_type: "application/gzip".to_string(),
                download_count: 0,
                created_at: String::new(),
                updated_at: String::new(),
            }],
            html_url: String::new(),
            published_at: None,
        };

        // Should not find the asset because it's still uploading
        let asset = release.find_asset_for_target("x86_64-apple-darwin");
        assert!(asset.is_none());
    }

    #[test]
    fn test_find_asset_prefers_correct_format() {
        let release = GitHubRelease {
            tag_name: "v0.1.0".to_string(),
            name: None,
            body: None,
            prerelease: false,
            draft: false,
            assets: vec![
                GitHubAsset {
                    name: "app-v0.1.0-x86_64-apple-darwin.zip".to_string(),
                    browser_download_url: String::new(),
                    state: "uploaded".to_string(),
                    digest: None,
                    size: 1024,
                    content_type: "application/zip".to_string(),
                    download_count: 0,
                    created_at: String::new(),
                    updated_at: String::new(),
                },
                GitHubAsset {
                    name: "app-v0.1.0-x86_64-apple-darwin.tar.gz".to_string(),
                    browser_download_url: String::new(),
                    state: "uploaded".to_string(),
                    digest: None,
                    size: 1024,
                    content_type: "application/gzip".to_string(),
                    download_count: 0,
                    created_at: String::new(),
                    updated_at: String::new(),
                },
            ],
            html_url: String::new(),
            published_at: None,
        };

        // For macOS, should prefer tar.gz
        let asset = release.find_asset_for_target("x86_64-apple-darwin");
        assert!(asset.unwrap().name.ends_with(".tar.gz"));
    }
}
