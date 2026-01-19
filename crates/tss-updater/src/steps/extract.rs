//! Archive extraction for updates.

use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Command;

use flate2::read::GzDecoder;
use tar::Archive;

use crate::error::{Result, UpdateError};

/// Archive type for extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveType {
    /// tar.gz archive (Linux)
    TarGz,
    /// ZIP archive (Windows)
    Zip,
    /// DMG disk image (macOS)
    Dmg,
}

impl ArchiveType {
    /// Get a human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::TarGz => "tar.gz",
            Self::Zip => "ZIP",
            Self::Dmg => "DMG",
        }
    }
}

impl std::fmt::Display for ArchiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// The expected binary name in archives.
#[cfg(target_os = "windows")]
const BINARY_NAME: &str = "trial-submission-studio.exe";

#[cfg(not(target_os = "windows"))]
const BINARY_NAME: &str = "trial-submission-studio";

/// Expected app bundle name for macOS.
#[cfg(target_os = "macos")]
const APP_BUNDLE_NAME: &str = "Trial Submission Studio.app";

/// Detects the archive type from the asset name.
#[must_use]
pub fn detect_archive_type(asset_name: &str) -> ArchiveType {
    let name_lower = asset_name.to_lowercase();

    if name_lower.ends_with(".dmg") {
        ArchiveType::Dmg
    } else if name_lower.ends_with(".tar.gz") || name_lower.ends_with(".tgz") {
        ArchiveType::TarGz
    } else if name_lower.ends_with(".zip") {
        ArchiveType::Zip
    } else {
        // Default based on platform
        #[cfg(target_os = "macos")]
        {
            ArchiveType::Dmg
        }
        #[cfg(target_os = "windows")]
        {
            ArchiveType::Zip
        }
        #[cfg(target_os = "linux")]
        {
            ArchiveType::TarGz
        }
    }
}

/// Extracts the update archive to a temporary directory.
///
/// Returns the path to the extracted content (binary or app bundle).
pub fn extract_archive(data: &[u8], asset_name: &str) -> Result<PathBuf> {
    let archive_type = detect_archive_type(asset_name);

    match archive_type {
        ArchiveType::TarGz => extract_tar_gz(data),
        ArchiveType::Zip => extract_zip(data),
        ArchiveType::Dmg => {
            #[cfg(target_os = "macos")]
            {
                extract_dmg(data)
            }
            #[cfg(not(target_os = "macos"))]
            {
                Err(UpdateError::ArchiveExtraction(
                    "DMG extraction is only supported on macOS".to_string(),
                ))
            }
        }
    }
}

/// Extracts binary from a tar.gz archive.
fn extract_tar_gz(data: &[u8]) -> Result<PathBuf> {
    tracing::debug!("Extracting from tar.gz archive");

    // Create temp directory
    let temp_dir = tempfile::tempdir()
        .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to create temp dir: {}", e)))?;

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

        // Look for the binary
        if path_str.ends_with(BINARY_NAME) || path.file_name().is_some_and(|n| n == BINARY_NAME) {
            let dest_path = temp_dir.path().join(BINARY_NAME);
            tracing::debug!("Extracting binary to: {:?}", dest_path);

            let mut binary = Vec::new();
            entry.read_to_end(&mut binary).map_err(|e| {
                UpdateError::ArchiveExtraction(format!("Failed to read binary: {}", e))
            })?;

            fs::write(&dest_path, &binary).map_err(|e| {
                UpdateError::ArchiveExtraction(format!("Failed to write binary: {}", e))
            })?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = fs::Permissions::from_mode(0o755);
                fs::set_permissions(&dest_path, perms).map_err(|e| {
                    UpdateError::ArchiveExtraction(format!("Failed to set permissions: {}", e))
                })?;
            }

            // Keep the temp directory around (don't delete on drop)
            let temp_path = temp_dir.keep();
            let path = temp_path.join(BINARY_NAME);
            return Ok(path);
        }
    }

    Err(UpdateError::ArchiveExtraction(format!(
        "Binary '{}' not found in tar.gz archive",
        BINARY_NAME
    )))
}

/// Extracts binary from a ZIP archive.
fn extract_zip(data: &[u8]) -> Result<PathBuf> {
    tracing::debug!("Extracting from ZIP archive");

    // Create temp directory
    let temp_dir = tempfile::tempdir()
        .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to create temp dir: {}", e)))?;

    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Look for the binary
        if name.ends_with(BINARY_NAME)
            || std::path::Path::new(&name)
                .file_name()
                .is_some_and(|n| n == BINARY_NAME)
        {
            let dest_path = temp_dir.path().join(BINARY_NAME);
            tracing::debug!("Extracting binary to: {:?}", dest_path);

            let mut binary = Vec::new();
            file.read_to_end(&mut binary).map_err(|e| {
                UpdateError::ArchiveExtraction(format!("Failed to read binary: {}", e))
            })?;

            fs::write(&dest_path, &binary).map_err(|e| {
                UpdateError::ArchiveExtraction(format!("Failed to write binary: {}", e))
            })?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = fs::Permissions::from_mode(0o755);
                fs::set_permissions(&dest_path, perms).map_err(|e| {
                    UpdateError::ArchiveExtraction(format!("Failed to set permissions: {}", e))
                })?;
            }

            // Keep the temp directory around (don't delete on drop)
            let temp_path = temp_dir.keep();
            let path = temp_path.join(BINARY_NAME);
            return Ok(path);
        }
    }

    Err(UpdateError::ArchiveExtraction(format!(
        "Binary '{}' not found in ZIP archive",
        BINARY_NAME
    )))
}

/// Extracts app bundle from a DMG (macOS only).
#[cfg(target_os = "macos")]
fn extract_dmg(data: &[u8]) -> Result<PathBuf> {
    tracing::info!("Extracting app bundle from DMG ({} bytes)", data.len());

    // Create temp directory
    let temp_dir = tempfile::tempdir()
        .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to create temp dir: {}", e)))?;

    // Write DMG to temp file
    let dmg_path = temp_dir.path().join("update.dmg");
    tracing::debug!("Writing DMG to: {:?}", dmg_path);
    fs::write(&dmg_path, data)
        .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to write DMG: {}", e)))?;

    // Create mount point
    let mount_point = temp_dir.path().join("dmg_mount");
    tracing::debug!("Creating mount point: {:?}", mount_point);
    fs::create_dir_all(&mount_point).map_err(|e| {
        UpdateError::ArchiveExtraction(format!("Failed to create mount point: {}", e))
    })?;

    // Mount DMG (readonly, no Finder window)
    tracing::info!("Mounting DMG...");
    let attach_output = Command::new("hdiutil")
        .args(["attach", "-nobrowse", "-readonly", "-mountpoint"])
        .arg(&mount_point)
        .arg(&dmg_path)
        .output()
        .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to run hdiutil: {}", e)))?;

    if !attach_output.status.success() {
        let stderr = String::from_utf8_lossy(&attach_output.stderr);
        let _ = fs::remove_file(&dmg_path);
        return Err(UpdateError::ArchiveExtraction(format!(
            "Failed to mount DMG: {}",
            stderr
        )));
    }
    tracing::info!("DMG mounted successfully at: {:?}", mount_point);

    // Find the .app bundle in the mounted DMG
    let mounted_app = mount_point.join(APP_BUNDLE_NAME);
    tracing::debug!("Looking for app bundle at: {:?}", mounted_app);

    if !mounted_app.exists() {
        // List what's actually in the mounted DMG for debugging
        let contents: Vec<String> = fs::read_dir(&mount_point)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();
        tracing::error!(
            "App bundle '{}' not found. DMG contains: {:?}",
            APP_BUNDLE_NAME,
            contents
        );

        // Detach and clean up before returning error
        let _ = Command::new("hdiutil")
            .args(["detach", "-quiet"])
            .arg(&mount_point)
            .output();
        let _ = fs::remove_file(&dmg_path);
        return Err(UpdateError::ArchiveExtraction(format!(
            "App bundle '{}' not found in DMG. Contents: {:?}",
            APP_BUNDLE_NAME, contents
        )));
    }
    tracing::info!("Found app bundle: {:?}", mounted_app);

    // Copy .app bundle using ditto (preserves ALL macOS metadata)
    let dest_app = temp_dir.path().join(APP_BUNDLE_NAME);
    tracing::info!("Copying app bundle with ditto to: {:?}", dest_app);
    let copy_output = Command::new("ditto")
        .arg(&mounted_app)
        .arg(&dest_app)
        .output()
        .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to run ditto: {}", e)))?;

    // Always detach DMG
    tracing::debug!("Detaching DMG...");
    let _ = Command::new("hdiutil")
        .args(["detach", "-quiet"])
        .arg(&mount_point)
        .output();

    // Clean up DMG file
    let _ = fs::remove_file(&dmg_path);

    if !copy_output.status.success() {
        let stderr = String::from_utf8_lossy(&copy_output.stderr);
        return Err(UpdateError::ArchiveExtraction(format!(
            "ditto copy failed: {}",
            stderr
        )));
    }

    // Keep the temp directory around and return path to app bundle
    let temp_path = temp_dir.keep();
    let final_path = temp_path.join(APP_BUNDLE_NAME);
    tracing::info!("Extracted app bundle to: {:?}", final_path);
    Ok(final_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_archive_type() {
        assert_eq!(
            detect_archive_type("app-v1.0.0-x86_64-apple-darwin.dmg"),
            ArchiveType::Dmg
        );
        assert_eq!(
            detect_archive_type("app-v1.0.0-x86_64-unknown-linux-gnu.tar.gz"),
            ArchiveType::TarGz
        );
        assert_eq!(
            detect_archive_type("app-v1.0.0-x86_64-pc-windows-msvc.zip"),
            ArchiveType::Zip
        );
    }

    #[test]
    fn test_extract_tar_gz_missing_binary() {
        // Create an empty gzip-compressed tar archive
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&[0u8; 1024]).unwrap();
        let compressed = encoder.finish().unwrap();

        let result = extract_tar_gz(&compressed);
        assert!(matches!(result, Err(UpdateError::ArchiveExtraction(_))));
    }

    #[test]
    fn test_extract_zip_missing_binary() {
        // Create an empty ZIP archive
        let mut buffer = Vec::new();
        {
            let zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
            zip.finish().unwrap();
        }

        let result = extract_zip(&buffer);
        assert!(matches!(result, Err(UpdateError::ArchiveExtraction(_))));
    }
}
