//! Installation functionality for updates.
//!
//! This module handles archive extraction and binary replacement
//! for installing downloaded updates.

use crate::error::{Result, UpdateError};
use flate2::read::GzDecoder;
use std::io::{Cursor, Read};
use tar::Archive;

/// The expected binary name in archives.
#[cfg(target_os = "windows")]
const BINARY_NAME: &str = "trial-submission-studio.exe";

#[cfg(not(target_os = "windows"))]
const BINARY_NAME: &str = "trial-submission-studio";

/// Extracts the application binary from an archive.
///
/// Automatically detects the archive type based on the asset name:
/// - `.tar.gz` or `.tgz` - tar+gzip archive
/// - `.zip` - ZIP archive
///
/// # Arguments
/// * `data` - The archive data as bytes
/// * `asset_name` - The asset filename (used to determine archive type)
///
/// # Returns
/// The extracted binary as bytes.
pub fn extract_binary(data: &[u8], asset_name: &str) -> Result<Vec<u8>> {
    let asset_lower = asset_name.to_lowercase();

    if asset_lower.ends_with(".tar.gz") || asset_lower.ends_with(".tgz") {
        extract_from_tar_gz(data)
    } else if asset_lower.ends_with(".zip") {
        extract_from_zip(data)
    } else {
        Err(UpdateError::ArchiveExtraction(format!(
            "Unknown archive format: {}",
            asset_name
        )))
    }
}

/// Extracts the binary from a tar.gz archive.
fn extract_from_tar_gz(data: &[u8]) -> Result<Vec<u8>> {
    tracing::debug!("Extracting from tar.gz archive");

    let cursor = Cursor::new(data);
    let decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(decoder);

    for entry_result in archive
        .entries()
        .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to read tar entries: {}", e)))?
    {
        let mut entry = entry_result.map_err(|e| {
            UpdateError::ArchiveExtraction(format!("Failed to read tar entry: {}", e))
        })?;

        let path = entry.path().map_err(|e| {
            UpdateError::ArchiveExtraction(format!("Failed to read entry path: {}", e))
        })?;

        let path_str = path.to_string_lossy();

        // Look for the binary (might be at root or in a subdirectory)
        if path_str.ends_with(BINARY_NAME) || path.file_name().is_some_and(|n| n == BINARY_NAME) {
            tracing::debug!("Found binary at: {}", path_str);

            let mut binary = Vec::new();
            entry.read_to_end(&mut binary).map_err(|e| {
                UpdateError::ArchiveExtraction(format!("Failed to read binary: {}", e))
            })?;

            return Ok(binary);
        }
    }

    Err(UpdateError::ArchiveExtraction(format!(
        "Binary '{}' not found in tar.gz archive",
        BINARY_NAME
    )))
}

/// Extracts the binary from a ZIP archive.
fn extract_from_zip(data: &[u8]) -> Result<Vec<u8>> {
    tracing::debug!("Extracting from ZIP archive");

    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Look for the binary (might be at root or in a subdirectory)
        if name.ends_with(BINARY_NAME)
            || std::path::Path::new(&name)
                .file_name()
                .is_some_and(|n| n == BINARY_NAME)
        {
            tracing::debug!("Found binary at: {}", name);

            let mut binary = Vec::new();
            file.read_to_end(&mut binary).map_err(|e| {
                UpdateError::ArchiveExtraction(format!("Failed to read binary: {}", e))
            })?;

            return Ok(binary);
        }
    }

    Err(UpdateError::ArchiveExtraction(format!(
        "Binary '{}' not found in ZIP archive",
        BINARY_NAME
    )))
}

/// Replaces the current executable with the new binary.
///
/// Uses the `self_replace` crate to safely replace the running executable.
///
/// # Arguments
/// * `binary` - The new binary data
///
/// # Safety
/// This function replaces the currently running executable. The application
/// must be restarted for the new version to take effect.
pub fn replace_current_executable(binary: &[u8]) -> Result<()> {
    use std::io::Write;

    tracing::info!("Replacing current executable ({} bytes)", binary.len());

    // Write binary to a temporary file
    let mut temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| UpdateError::Installation(format!("Failed to create temp file: {}", e)))?;

    temp_file
        .write_all(binary)
        .map_err(|e| UpdateError::Installation(format!("Failed to write to temp file: {}", e)))?;

    // Sync to ensure all data is written
    temp_file
        .flush()
        .map_err(|e| UpdateError::Installation(format!("Failed to flush temp file: {}", e)))?;

    // Replace the current executable with the temp file
    self_replace::self_replace(temp_file.path())
        .map_err(|e| UpdateError::Installation(format!("Failed to replace executable: {}", e)))?;

    tracing::info!("Executable replaced successfully");
    Ok(())
}

/// Restarts the application.
///
/// This function attempts to restart the application by spawning a new
/// process and exiting the current one.
pub fn restart_application() -> Result<()> {
    tracing::info!("Restarting application");

    let current_exe = std::env::current_exe().map_err(|e| {
        UpdateError::Installation(format!("Failed to get current executable path: {}", e))
    })?;

    // Spawn the new process
    std::process::Command::new(&current_exe)
        .spawn()
        .map_err(|e| UpdateError::Installation(format!("Failed to spawn new process: {}", e)))?;

    // Exit the current process
    std::process::exit(0);
}

/// Gets the current target triple for the running system.
///
/// This is used to find the correct release asset for the current platform.
#[must_use]
pub fn get_target_triple() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    match (os, arch) {
        ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
        ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
        ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),
        ("windows", "aarch64") => "aarch64-pc-windows-msvc".to_string(),
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu".to_string(),
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu".to_string(),
        _ => format!("{}-{}", arch, os),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_target_triple() {
        let target = get_target_triple();
        // Should return a valid-looking target triple
        assert!(target.contains('-'));
        assert!(!target.is_empty());
    }

    #[test]
    fn test_extract_binary_unknown_format() {
        let data = b"not an archive";
        let result = extract_binary(data, "unknown.xyz");

        assert!(matches!(result, Err(UpdateError::ArchiveExtraction(_))));
    }

    #[test]
    fn test_extract_binary_empty_tar_gz() {
        // Create an empty gzip-compressed tar archive
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        // Write an empty tar (just end-of-archive markers)
        encoder.write_all(&[0u8; 1024]).unwrap();
        let compressed = encoder.finish().unwrap();

        let result = extract_binary(&compressed, "test.tar.gz");

        // Should fail because binary not found, not because of parsing
        assert!(matches!(result, Err(UpdateError::ArchiveExtraction(_))));
    }

    #[test]
    fn test_extract_binary_empty_zip() {
        // Create an empty ZIP archive
        let mut buffer = Vec::new();
        {
            let zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
            zip.finish().unwrap();
        }

        let result = extract_binary(&buffer, "test.zip");

        // Should fail because binary not found
        assert!(matches!(result, Err(UpdateError::ArchiveExtraction(_))));
    }
}
