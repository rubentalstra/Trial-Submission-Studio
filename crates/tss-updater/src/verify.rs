//! SHA256 verification for downloaded updates.
//!
//! This module provides functions to verify the integrity of downloaded
//! update files using SHA256 hashes provided by the GitHub Releases API.

use crate::error::{Result, UpdateError};
use sha2::{Digest, Sha256};

/// Verifies that the downloaded data matches the expected SHA256 digest.
///
/// # Arguments
/// * `data` - The downloaded file bytes
/// * `expected_digest` - The expected digest from GitHub API (format: "sha256:..." or just the hex hash)
///
/// # Returns
/// * `Ok(())` if verification passes
/// * `Err(UpdateError::ChecksumMismatch)` if the hashes don't match
/// * `Err(UpdateError::NoDigestAvailable)` if the digest format is invalid
///
/// # Example
/// ```
/// use tss_updater::verify::verify_sha256;
///
/// let data = b"Hello, World!";
/// let digest = "sha256:dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";
///
/// assert!(verify_sha256(data, digest).is_ok());
/// ```
pub fn verify_sha256(data: &[u8], expected_digest: &str) -> Result<()> {
    // Extract the hex hash from the digest string
    // Support both "sha256:abc123" format and plain "abc123" format
    let expected_hash = expected_digest
        .strip_prefix("sha256:")
        .unwrap_or(expected_digest)
        .trim()
        .to_lowercase();

    // Validate that we have a valid hex string
    if expected_hash.is_empty() {
        return Err(UpdateError::NoDigestAvailable);
    }

    if expected_hash.len() != 64 {
        return Err(UpdateError::NoDigestAvailable);
    }

    // Compute SHA256 hash of the downloaded data
    let actual_hash = compute_sha256(data);

    // Compare hashes
    if actual_hash != expected_hash {
        return Err(UpdateError::ChecksumMismatch {
            expected: expected_hash,
            actual: actual_hash,
        });
    }

    tracing::info!("SHA256 verification passed: {}", actual_hash);
    Ok(())
}

/// Computes the SHA256 hash of the given data.
///
/// # Arguments
/// * `data` - The data to hash
///
/// # Returns
/// The lowercase hexadecimal representation of the SHA256 hash.
#[must_use]
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Verification result for display in the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationStatus {
    /// SHA256 verification passed.
    Verified,
    /// SHA256 verification failed with the given hash mismatch.
    Failed {
        /// Expected hash.
        expected: String,
        /// Actual computed hash.
        actual: String,
    },
    /// No digest available for verification.
    Unavailable,
}

impl VerificationStatus {
    /// Returns whether verification was successful.
    #[must_use]
    pub fn is_verified(&self) -> bool {
        matches!(self, Self::Verified)
    }

    /// Returns whether verification failed (not just unavailable).
    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_sha256_valid() {
        let data = b"Hello, World!";
        // SHA256 of "Hello, World!"
        let expected = "sha256:dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";

        assert!(verify_sha256(data, expected).is_ok());
    }

    #[test]
    fn test_verify_sha256_without_prefix() {
        let data = b"Hello, World!";
        let expected = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";

        assert!(verify_sha256(data, expected).is_ok());
    }

    #[test]
    fn test_verify_sha256_mismatch() {
        let data = b"Hello, World!";
        let wrong_digest =
            "sha256:0000000000000000000000000000000000000000000000000000000000000000";

        let result = verify_sha256(data, wrong_digest);
        assert!(result.is_err());

        match result {
            Err(UpdateError::ChecksumMismatch { expected, actual }) => {
                assert_eq!(
                    expected,
                    "0000000000000000000000000000000000000000000000000000000000000000"
                );
                assert_eq!(
                    actual,
                    "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
                );
            }
            _ => panic!("Expected ChecksumMismatch error"),
        }
    }

    #[test]
    fn test_verify_sha256_empty_digest() {
        let data = b"Hello, World!";
        let result = verify_sha256(data, "");

        assert!(matches!(result, Err(UpdateError::NoDigestAvailable)));
    }

    #[test]
    fn test_verify_sha256_invalid_length() {
        let data = b"Hello, World!";
        let result = verify_sha256(data, "sha256:abc123");

        assert!(matches!(result, Err(UpdateError::NoDigestAvailable)));
    }

    #[test]
    fn test_compute_sha256() {
        let data = b"Hello, World!";
        let hash = compute_sha256(data);

        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn test_compute_sha256_empty() {
        let data = b"";
        let hash = compute_sha256(data);

        // SHA256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_verification_status() {
        let verified = VerificationStatus::Verified;
        assert!(verified.is_verified());
        assert!(!verified.is_failed());

        let failed = VerificationStatus::Failed {
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        assert!(!failed.is_verified());
        assert!(failed.is_failed());

        let unavailable = VerificationStatus::Unavailable;
        assert!(!unavailable.is_verified());
        assert!(!unavailable.is_failed());
    }
}
