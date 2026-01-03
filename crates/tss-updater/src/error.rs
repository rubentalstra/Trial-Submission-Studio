//! Error types for the auto-update system.

use thiserror::Error;

/// Errors that can occur during the update process.
#[derive(Debug, Error)]
pub enum UpdateError {
    /// Failed to parse version string.
    #[error("invalid version format: {0}")]
    InvalidVersion(String),

    /// Network request failed.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Failed to parse JSON response.
    #[error("failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    /// No compatible release asset found for current platform.
    #[error("no compatible release found for {platform} {arch}")]
    NoCompatibleRelease {
        /// The current platform (e.g., "macos", "windows", "linux").
        platform: String,
        /// The current architecture (e.g., "x86_64", "aarch64").
        arch: String,
    },

    /// Checksum verification failed.
    #[error("checksum verification failed: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// The expected SHA256 hash.
        expected: String,
        /// The actual SHA256 hash of the downloaded file.
        actual: String,
    },

    /// Failed to download checksum file.
    #[error("checksum file not found for asset: {0}")]
    ChecksumNotFound(String),

    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to apply update.
    #[error("failed to apply update: {0}")]
    InstallFailed(String),

    /// Update was cancelled by user.
    #[error("update cancelled")]
    Cancelled,

    /// Rate limited by GitHub API.
    #[error("rate limited by GitHub API, retry after {retry_after_secs} seconds")]
    RateLimited {
        /// Seconds to wait before retrying.
        retry_after_secs: u64,
    },

    /// Already up to date.
    #[error("already running the latest version ({0})")]
    AlreadyUpToDate(String),

    /// GitHub API error.
    #[error("GitHub API error: {status} - {message}")]
    GitHubApi {
        /// HTTP status code.
        status: u16,
        /// Error message from GitHub.
        message: String,
    },

    /// Self-update crate error.
    #[error("self-update error: {0}")]
    SelfUpdate(String),

    /// Signature verification failed.
    #[error("signature verification failed: {0}")]
    SignatureInvalid(String),
}

/// Result type alias for update operations.
pub type Result<T> = std::result::Result<T, UpdateError>;
