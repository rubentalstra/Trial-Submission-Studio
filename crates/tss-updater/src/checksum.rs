//! SHA256 checksum verification.
//!
//! Provides functionality for computing and verifying SHA256 checksums
//! of downloaded files to ensure integrity.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};
use tracing::{debug, info};

use crate::error::{Result, UpdateError};

/// Buffer size for reading files during checksum computation.
const BUFFER_SIZE: usize = 65536; // 64 KB

/// Compute the SHA256 hash of a file.
pub fn compute_file_sha256(path: &Path) -> Result<String> {
    debug!("Computing SHA256 for: {}", path.display());

    let file = File::open(path).map_err(UpdateError::Io)?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(UpdateError::Io)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    let hex_hash = hex::encode(hash);

    debug!("SHA256: {}", hex_hash);
    Ok(hex_hash)
}

/// Verify that a file matches the expected SHA256 hash.
pub fn verify_sha256(path: &Path, expected: &str) -> Result<()> {
    info!("Verifying SHA256 checksum for: {}", path.display());

    let actual = compute_file_sha256(path)?;
    let expected = expected.to_lowercase();

    if actual != expected {
        return Err(UpdateError::ChecksumMismatch { expected, actual });
    }

    info!("Checksum verification successful");
    Ok(())
}

/// Parse a checksum from a checksum file content.
///
/// Supports two formats:
/// - Just the hash: `abc123...`
/// - Hash with filename: `abc123...  filename.zip`
pub fn parse_checksum_file(content: &str) -> Option<String> {
    content
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().next())
        .map(|s| s.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_compute_sha256() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_sha256.txt");

        // Write known content
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        drop(file);

        let hash = compute_file_sha256(&test_file).unwrap();

        // Known SHA256 of "Hello, World!"
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_verify_sha256_success() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_verify_sha256.txt");

        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        drop(file);

        let result = verify_sha256(
            &test_file,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f",
        );
        assert!(result.is_ok());

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_verify_sha256_failure() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_verify_sha256_fail.txt");

        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        drop(file);

        let result = verify_sha256(&test_file, "wrong_hash");
        assert!(result.is_err());

        if let Err(UpdateError::ChecksumMismatch { expected, actual }) = result {
            assert_eq!(expected, "wrong_hash");
            assert_eq!(
                actual,
                "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
            );
        } else {
            panic!("Expected ChecksumMismatch error");
        }

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_parse_checksum_file() {
        // Just hash
        assert_eq!(
            parse_checksum_file("abc123def456"),
            Some("abc123def456".to_string())
        );

        // Hash with filename (BSD style)
        assert_eq!(
            parse_checksum_file("abc123def456  myfile.zip"),
            Some("abc123def456".to_string())
        );

        // Hash with filename (GNU style with *)
        assert_eq!(
            parse_checksum_file("abc123def456 *myfile.zip"),
            Some("abc123def456".to_string())
        );

        // Multiple lines (take first)
        assert_eq!(
            parse_checksum_file("abc123\nother content"),
            Some("abc123".to_string())
        );

        // Empty content
        assert_eq!(parse_checksum_file(""), None);

        // Uppercase should be lowercased
        assert_eq!(parse_checksum_file("ABC123DEF"), Some("abc123def".to_string()));
    }
}
