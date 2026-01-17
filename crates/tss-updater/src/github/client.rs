//! GitHub API client for fetching release information.

use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};

use super::types::GitHubRelease;
use crate::error::{Result, UpdateError};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GitHubClient::new("rubentalstra", "Trial-Submission-Studio");
        assert!(client.is_ok());
    }
}
