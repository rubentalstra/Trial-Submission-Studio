//! GitHub API client for fetching releases.
//!
//! Provides a client for interacting with the GitHub Releases API to
//! check for and download application updates.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use tracing::{debug, warn};

use crate::config::UpdateChannel;
use crate::error::{Result, UpdateError};
use crate::platform::Platform;
use crate::release::{Release, UpdateInfo};
use crate::version::Version;

/// GitHub repository owner.
const REPO_OWNER: &str = "rubentalstra";

/// GitHub repository name.
const REPO_NAME: &str = "Trial-Submission-Studio";

/// GitHub API base URL.
const GITHUB_API_URL: &str = "https://api.github.com";

/// Cache duration for release checks (5 minutes).
const CACHE_DURATION: Duration = Duration::from_secs(300);

/// HTTP request timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Cached release data.
struct CachedReleases {
    /// The cached releases.
    releases: Vec<Release>,
    /// When the cache was populated.
    fetched_at: Instant,
}

/// Client for interacting with GitHub Releases API.
pub struct GitHubClient {
    /// HTTP client.
    client: Client,
    /// Cached releases to avoid repeated API calls.
    cache: Mutex<Option<CachedReleases>>,
}

impl GitHubClient {
    /// Create a new GitHub client.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(UpdateError::Network)?;

        Ok(Self {
            client,
            cache: Mutex::new(None),
        })
    }

    /// Get the releases API URL.
    fn releases_url(&self) -> String {
        format!("{GITHUB_API_URL}/repos/{REPO_OWNER}/{REPO_NAME}/releases")
    }

    /// Fetch all releases from GitHub (with caching).
    fn fetch_releases(&self, force_refresh: bool) -> Result<Vec<Release>> {
        // Check cache first
        if !force_refresh {
            let cache = self.cache.lock().unwrap();
            if let Some(ref cached) = *cache {
                if cached.fetched_at.elapsed() < CACHE_DURATION {
                    debug!("Using cached releases (age: {:?})", cached.fetched_at.elapsed());
                    return Ok(cached.releases.clone());
                }
            }
        }

        debug!("Fetching releases from GitHub API");

        let response = self
            .client
            .get(self.releases_url())
            .header(USER_AGENT, format!("{}/{}", REPO_NAME, env!("CARGO_PKG_VERSION")))
            .header(ACCEPT, "application/vnd.github.v3+json")
            .send()
            .map_err(UpdateError::Network)?;

        // Handle rate limiting
        if response.status().as_u16() == 403 {
            if let Some(retry_after) = response.headers().get("retry-after") {
                if let Ok(secs) = retry_after.to_str().unwrap_or("60").parse() {
                    return Err(UpdateError::RateLimited {
                        retry_after_secs: secs,
                    });
                }
            }
            return Err(UpdateError::RateLimited {
                retry_after_secs: 60,
            });
        }

        // Handle other errors
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(UpdateError::GitHubApi { status, message });
        }

        let releases: Vec<Release> = response.json().map_err(UpdateError::Network)?;

        // Update cache
        {
            let mut cache = self.cache.lock().unwrap();
            *cache = Some(CachedReleases {
                releases: releases.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(releases)
    }

    /// Get the latest release for the specified channel.
    ///
    /// For stable channel, returns the latest non-prerelease, non-draft release.
    /// For beta channel, returns the latest release (including prereleases).
    pub fn get_latest_release(&self, channel: UpdateChannel) -> Result<Option<Release>> {
        let releases = self.fetch_releases(false)?;

        let latest = releases
            .into_iter()
            .filter(|r| !r.draft)
            .filter(|r| {
                match channel {
                    UpdateChannel::Stable => !r.prerelease,
                    UpdateChannel::Beta => true, // Include all releases
                }
            })
            .filter(|r| {
                // Only include releases we can parse the version for
                r.version().is_ok()
            })
            .max_by(|a, b| {
                // Compare by parsed version
                let va = a.version().unwrap();
                let vb = b.version().unwrap();
                va.cmp(&vb)
            });

        Ok(latest)
    }

    /// Check for available updates.
    ///
    /// Returns `Some(UpdateInfo)` if an update is available, `None` if already up to date.
    pub fn check_for_update(
        &self,
        current_version: &Version,
        channel: UpdateChannel,
        platform: &Platform,
    ) -> Result<Option<UpdateInfo>> {
        debug!(
            "Checking for updates (current: {}, channel: {:?})",
            current_version,
            channel
        );

        let release = match self.get_latest_release(channel)? {
            Some(r) => r,
            None => {
                debug!("No releases found");
                return Ok(None);
            }
        };

        let new_version = release.version()?;

        // Check if this version matches the channel preference
        if !channel.includes(&new_version) {
            debug!("Latest release {} not in channel {:?}", new_version, channel);
            return Ok(None);
        }

        // Check if update is newer
        if new_version <= *current_version {
            debug!("Already up to date (latest: {})", new_version);
            return Ok(None);
        }

        debug!("Update available: {} -> {}", current_version, new_version);

        // Find the appropriate asset for this platform
        let asset = platform.find_asset(&release)?.clone();

        // Find the checksum asset
        let checksum_asset = release.find_checksum_asset(&asset).cloned();

        if checksum_asset.is_none() {
            warn!("No checksum file found for asset: {}", asset.name);
        }

        let update_info = UpdateInfo::new(
            current_version.clone(),
            release,
            asset,
            checksum_asset,
        )?;

        Ok(Some(update_info))
    }

    /// Force a fresh check for updates (bypassing cache).
    pub fn check_for_update_fresh(
        &self,
        current_version: &Version,
        channel: UpdateChannel,
        platform: &Platform,
    ) -> Result<Option<UpdateInfo>> {
        // Clear the cache first
        {
            let mut cache = self.cache.lock().unwrap();
            *cache = None;
        }

        self.check_for_update(current_version, channel, platform)
    }

    /// Fetch the content of a checksum file.
    pub fn fetch_checksum(&self, url: &str) -> Result<String> {
        debug!("Fetching checksum from: {}", url);

        let response = self
            .client
            .get(url)
            .header(USER_AGENT, format!("{}/{}", REPO_NAME, env!("CARGO_PKG_VERSION")))
            .send()
            .map_err(UpdateError::Network)?;

        if !response.status().is_success() {
            return Err(UpdateError::ChecksumNotFound(url.to_string()));
        }

        let content = response.text().map_err(UpdateError::Network)?;

        // Parse the checksum file (format: "hash  filename" or just "hash")
        let checksum = content
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().next())
            .map(|s| s.to_lowercase())
            .ok_or_else(|| UpdateError::ChecksumNotFound(url.to_string()))?;

        Ok(checksum)
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_releases_url() {
        let client = GitHubClient::new().unwrap();
        assert_eq!(
            client.releases_url(),
            "https://api.github.com/repos/rubentalstra/Trial-Submission-Studio/releases"
        );
    }

    #[test]
    fn test_client_creation() {
        let client = GitHubClient::new();
        assert!(client.is_ok());
    }
}
