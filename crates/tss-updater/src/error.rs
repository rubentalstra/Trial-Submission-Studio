//! Error types for the auto-update system.

use thiserror::Error;

/// Suggested action for the user when an error occurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestedAction {
    /// Retry the operation immediately.
    Retry,
    /// Wait for a specified time and retry.
    WaitAndRetry(u64),
    /// Retry the download from scratch.
    RetryDownload,
    /// Download the update manually from the releases page.
    ManualDownload,
    /// Run the application as administrator.
    RunAsAdmin,
    /// Free up disk space.
    FreeSpace,
    /// Reinstall the application.
    Reinstall,
    /// Report an issue to support.
    ReportIssue,
    /// No action can be taken.
    None,
}

impl SuggestedAction {
    /// Returns a user-friendly description of the suggested action.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Retry => "Please try again.",
            Self::WaitAndRetry(_) => "Please wait and try again.",
            Self::RetryDownload => "Please try downloading again.",
            Self::ManualDownload => "Please download the update manually from the releases page.",
            Self::RunAsAdmin => "Please run the application as administrator.",
            Self::FreeSpace => "Please free up some disk space and try again.",
            Self::Reinstall => "Please reinstall the application.",
            Self::ReportIssue => "Please report this issue to support.",
            Self::None => "",
        }
    }
}

/// Errors that can occur during the update process.
#[derive(Debug, Error, Clone)]
#[non_exhaustive]
pub enum UpdateError {
    /// Failed to parse version string.
    #[error("invalid version format: {0}")]
    InvalidVersion(String),

    /// Network request failed.
    #[error("network error: {0}")]
    Network(String),

    /// Connection timed out.
    #[error("connection timed out")]
    Timeout,

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

    /// Code signature verification failed (macOS).
    #[error("code signature verification failed: {0}")]
    SignatureInvalid(String),

    /// Updater helper binary not found (macOS).
    #[error("updater helper not found")]
    HelperNotFound,

    /// Helper process failed (macOS).
    #[error("helper process failed: {0}")]
    HelperFailed(String),

    /// Not running from an app bundle (macOS development build).
    #[error("not running from an app bundle")]
    NotInAppBundle,

    /// No compatible asset format available for this platform.
    ///
    /// On macOS, DMG format is required to preserve code signatures.
    /// This error occurs when a release only has older formats (tar.gz).
    #[error("no compatible update package found for this platform")]
    NoCompatibleAsset,

    /// Update was cancelled by user.
    #[error("update cancelled")]
    Cancelled,

    /// Insufficient disk space.
    #[error("insufficient disk space: need {required} bytes, have {available} bytes")]
    InsufficientSpace {
        /// Required bytes.
        required: u64,
        /// Available bytes.
        available: u64,
    },

    /// Permission denied during file operations.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid update state transition.
    #[error("invalid state transition: cannot go from {from} to {to}")]
    InvalidStateTransition {
        /// Current state.
        from: String,
        /// Attempted target state.
        to: String,
    },

    /// No update available.
    #[error("no update available")]
    NoUpdateAvailable,

    /// Already up to date.
    #[error("already running the latest version: {0}")]
    AlreadyUpToDate(String),
}

impl UpdateError {
    /// Returns a user-friendly error message suitable for display in the UI.
    #[must_use]
    pub fn user_message(&self) -> &str {
        match self {
            Self::Network(_) | Self::Timeout => {
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
            Self::SignatureInvalid(_) => {
                "The update's code signature is invalid. Please download again or contact support."
            }
            Self::HelperNotFound => "Update helper not found. Please reinstall the application.",
            Self::HelperFailed(_) => {
                "The update process failed. Please try again or reinstall manually."
            }
            Self::NotInAppBundle => {
                "Cannot update: not running from an installed app bundle. \
                 Please install the app to /Applications and run it from there."
            }
            Self::NoCompatibleAsset => {
                "This release doesn't have a compatible update package for macOS. \
                 Please download the latest DMG from the releases page."
            }
            Self::Cancelled => "Update was cancelled.",
            Self::InsufficientSpace { .. } => {
                "Not enough disk space to download the update. Please free up some space."
            }
            Self::PermissionDenied(_) => {
                "Permission denied. Try running the application as administrator."
            }
            Self::NoUpdateAvailable | Self::AlreadyUpToDate(_) => {
                "You are already running the latest version."
            }
            Self::InvalidVersion(_)
            | Self::Io(_)
            | Self::JsonParse(_)
            | Self::InvalidStateTransition { .. } => "An unexpected error occurred.",
        }
    }

    /// Returns whether this error is potentially recoverable with a retry.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::Timeout | Self::RateLimited { .. } | Self::Io(_)
        )
    }

    /// Returns the suggested action for the user.
    #[must_use]
    pub fn suggested_action(&self) -> SuggestedAction {
        match self {
            Self::Network(_) | Self::Timeout => SuggestedAction::Retry,
            Self::RateLimited { retry_after } => SuggestedAction::WaitAndRetry(*retry_after),
            Self::ChecksumMismatch { .. } => SuggestedAction::RetryDownload,
            Self::NoCompatibleAsset | Self::NoAssetFound(_) => SuggestedAction::ManualDownload,
            Self::PermissionDenied(_) => SuggestedAction::RunAsAdmin,
            Self::InsufficientSpace { .. } => SuggestedAction::FreeSpace,
            Self::HelperNotFound | Self::NotInAppBundle => SuggestedAction::Reinstall,
            Self::SignatureInvalid(_) | Self::HelperFailed(_) => SuggestedAction::ReportIssue,
            Self::Cancelled | Self::NoUpdateAvailable | Self::AlreadyUpToDate(_) => {
                SuggestedAction::None
            }
            _ => SuggestedAction::Retry,
        }
    }
}

impl From<reqwest::Error> for UpdateError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout
        } else {
            Self::Network(err.to_string())
        }
    }
}

impl From<std::io::Error> for UpdateError {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind;
        match err.kind() {
            ErrorKind::PermissionDenied => Self::PermissionDenied(err.to_string()),
            _ => Self::Io(err.to_string()),
        }
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

        let err = UpdateError::Cancelled;
        assert_eq!(err.user_message(), "Update was cancelled.");
    }

    #[test]
    fn test_retryable() {
        assert!(UpdateError::Network("timeout".to_string()).is_retryable());
        assert!(UpdateError::Timeout.is_retryable());
        assert!(UpdateError::RateLimited { retry_after: 60 }.is_retryable());
        assert!(
            !UpdateError::ChecksumMismatch {
                expected: "a".to_string(),
                actual: "b".to_string()
            }
            .is_retryable()
        );
        assert!(!UpdateError::Cancelled.is_retryable());
    }

    #[test]
    fn test_suggested_actions() {
        assert_eq!(
            UpdateError::Network("test".to_string()).suggested_action(),
            SuggestedAction::Retry
        );
        assert_eq!(
            UpdateError::RateLimited { retry_after: 60 }.suggested_action(),
            SuggestedAction::WaitAndRetry(60)
        );
        assert_eq!(
            UpdateError::InsufficientSpace {
                required: 100,
                available: 50
            }
            .suggested_action(),
            SuggestedAction::FreeSpace
        );
        assert_eq!(
            UpdateError::Cancelled.suggested_action(),
            SuggestedAction::None
        );
    }

    #[test]
    fn test_suggested_action_description() {
        assert!(!SuggestedAction::Retry.description().is_empty());
        assert!(!SuggestedAction::ManualDownload.description().is_empty());
        assert!(SuggestedAction::None.description().is_empty());
    }
}
