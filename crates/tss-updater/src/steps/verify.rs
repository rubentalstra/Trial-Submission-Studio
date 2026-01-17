//! SHA256 verification for downloaded updates.

use sha2::{Digest, Sha256};

use crate::error::{Result, UpdateError};
use crate::release::UpdateInfo;

/// Verifies the downloaded data against the expected SHA256 digest.
///
/// Returns the verified SHA256 hash on success.
pub fn verify_download(data: &[u8], info: &UpdateInfo) -> Result<String> {
    match &info.asset.digest {
        Some(digest) => verify_sha256(data, digest),
        None => Err(UpdateError::NoDigestAvailable),
    }
}

/// Verifies that the downloaded data matches the expected SHA256 digest.
///
/// Returns the verified SHA256 hash on success.
pub fn verify_sha256(data: &[u8], expected_digest: &str) -> Result<String> {
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
    Ok(actual_hash)
}

/// Computes the SHA256 hash of the given data.
#[must_use]
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_sha256_valid() {
        let data = b"Hello, World!";
        // SHA256 of "Hello, World!"
        let expected = "sha256:dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";

        let result = verify_sha256(data, expected);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_sha256_without_prefix() {
        let data = b"Hello, World!";
        let expected = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";

        let result = verify_sha256(data, expected);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_sha256_mismatch() {
        let data = b"Hello, World!";
        let wrong_digest =
            "sha256:0000000000000000000000000000000000000000000000000000000000000000";

        let result = verify_sha256(data, wrong_digest);
        assert!(matches!(result, Err(UpdateError::ChecksumMismatch { .. })));
    }

    #[test]
    fn test_verify_sha256_empty_digest() {
        let data = b"Hello, World!";
        let result = verify_sha256(data, "");

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
}
