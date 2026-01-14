//! Error types for the auto-update system.

use thiserror::Error;

/// Errors that can occur during the update process.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UpdateError {
    /// Failed to parse version string.
    #[error("invalid version format: {0}")]
    InvalidVersion(String),

    /// Network request failed.
    #[error("network error: {0}")]
    Network(String),

    /// No suitable release asset found for the current platform.
    #[error("no release asset found for target: {0}")]
    NoAssetFound(String),

    /// SHA256 checksum verification failed.
    #[error("checksum verification failed: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// Expected SHA256 hash from GitHub.
        expected: String,
        /// Actual SHA256 hash of downloaded data.
        actual: String,
    },

    /// Release asset has no digest available for verification.
    #[error("no digest available for verification")]
    NoDigestAvailable,

    /// Failed to install the update.
    #[error("installation error: {0}")]
    Installation(String),

    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    Io(String),

    /// Failed to parse JSON response.
    #[error("JSON parse error: {0}")]
    JsonParse(String),

    /// Archive extraction failed.
    #[error("archive extraction error: {0}")]
    ArchiveExtraction(String),

    /// GitHub API rate limit exceeded.
    #[error("GitHub API rate limit exceeded, retry after {retry_after} seconds")]
    RateLimited {
        /// Seconds until rate limit resets.
        retry_after: u64,
    },
}

impl UpdateError {
    /// Returns a user-friendly error message suitable for display in the UI.
    #[must_use]
    pub fn user_message(&self) -> &str {
        match self {
            Self::Network(_) => {
                "Could not connect to GitHub. Please check your internet connection."
            }
            Self::ChecksumMismatch { .. } => {
                "Security verification failed. The download may have been tampered with."
            }
            Self::NoDigestAvailable => "Security verification not available for this release.",
            Self::NoAssetFound(_) => "No update available for your platform.",
            Self::Installation(_) => "Could not install the update. Please try again.",
            Self::ArchiveExtraction(_) => "Could not extract the update package.",
            Self::RateLimited { .. } => "GitHub API rate limit reached. Please try again later.",
            Self::InvalidVersion(_) | Self::Io(_) | Self::JsonParse(_) => {
                "An unexpected error occurred."
            }
        }
    }

    /// Returns whether this error is potentially recoverable with a retry.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::RateLimited { .. } | Self::Io(_)
        )
    }
}

impl From<reqwest::Error> for UpdateError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

impl From<std::io::Error> for UpdateError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for UpdateError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonParse(err.to_string())
    }
}

impl From<zip::result::ZipError> for UpdateError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::ArchiveExtraction(err.to_string())
    }
}

/// Result type alias for update operations.
pub type Result<T> = std::result::Result<T, UpdateError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_messages() {
        let err = UpdateError::Network("connection refused".to_string());
        assert!(err.user_message().contains("internet connection"));

        let err = UpdateError::ChecksumMismatch {
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        assert!(err.user_message().contains("Security verification failed"));

        let err = UpdateError::NoAssetFound("x86_64-apple-darwin".to_string());
        assert!(err.user_message().contains("platform"));
    }

    #[test]
    fn test_retryable() {
        assert!(UpdateError::Network("timeout".to_string()).is_retryable());
        assert!(UpdateError::RateLimited { retry_after: 60 }.is_retryable());
        assert!(
            !UpdateError::ChecksumMismatch {
                expected: "a".to_string(),
                actual: "b".to_string()
            }
            .is_retryable()
        );
    }
}
