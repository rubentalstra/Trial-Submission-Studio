//! File hashing utilities for source change detection.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::{PersistenceError, Result};

/// Compute SHA-256 hash of a file.
///
/// Uses buffered reading for efficient processing of large files.
pub fn compute_file_hash(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(|e| PersistenceError::Io {
        operation: "read",
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| PersistenceError::Io {
            operation: "read",
            path: path.to_path_buf(),
            source: e,
        })?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

/// Verify that a file's hash matches the expected value.
///
/// Returns `Ok(true)` if hashes match, `Ok(false)` if they don't,
/// or an error if the file can't be read.
pub fn verify_file_hash(path: &Path, expected_hash: &str) -> Result<bool> {
    let actual_hash = compute_file_hash(path)?;
    Ok(actual_hash == expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compute_file_hash() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let hash = compute_file_hash(temp_file.path()).unwrap();

        // Known SHA-256 hash for "Hello, World!"
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn test_verify_file_hash() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Test content").unwrap();
        temp_file.flush().unwrap();

        let hash = compute_file_hash(temp_file.path()).unwrap();

        // Same content should verify
        assert!(verify_file_hash(temp_file.path(), &hash).unwrap());

        // Wrong hash should not verify
        assert!(!verify_file_hash(temp_file.path(), "wrong_hash").unwrap());
    }
}
