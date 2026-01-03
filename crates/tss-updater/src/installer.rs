//! Platform-specific update installation using self_update.
//!
//! This module handles the seamless installation of updates, including:
//! - Extracting the downloaded archive
//! - Replacing the current executable/app bundle
//! - Restarting the application

use std::path::Path;

use self_update::backends::github::Update;
use self_update::cargo_crate_version;
use tracing::{debug, info};

use crate::error::{Result, UpdateError};
use crate::release::UpdateInfo;

/// GitHub repository owner.
const REPO_OWNER: &str = "rubentalstra";

/// GitHub repository name.
const REPO_NAME: &str = "Trial-Submission-Studio";

/// Binary name within the archive.
#[cfg(target_os = "windows")]
const BIN_NAME: &str = "trial-submission-studio.exe";

#[cfg(not(target_os = "windows"))]
const BIN_NAME: &str = "trial-submission-studio";

/// Install an update using self_update.
///
/// This function:
/// 1. Downloads the release from GitHub
/// 2. Extracts the archive
/// 3. Replaces the current binary
/// 4. The caller should then restart the application
///
/// # Arguments
///
/// * `update_info` - Information about the update to install
///
/// # Errors
///
/// Returns an error if the update fails at any stage.
pub fn install_update(update_info: &UpdateInfo) -> Result<()> {
    let version_tag = format!("v{}", update_info.new_version);

    info!(
        "Installing update {} -> {}",
        update_info.current_version, update_info.new_version
    );
    debug!("Target version tag: {}", version_tag);

    let status = Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .target_version_tag(&version_tag)
        .show_download_progress(false) // We handle progress ourselves
        .current_version(cargo_crate_version!())
        .build()
        .map_err(|e| UpdateError::SelfUpdate(e.to_string()))?
        .update()
        .map_err(|e| UpdateError::SelfUpdate(e.to_string()))?;

    info!("Update installed successfully: {}", status.version());

    Ok(())
}

/// Install an update from a pre-downloaded archive.
///
/// Use this when you've already downloaded and verified the update
/// using our custom download and checksum modules.
///
/// # Arguments
///
/// * `archive_path` - Path to the downloaded archive (zip or tar.gz)
/// * `update_info` - Information about the update
///
/// # Errors
///
/// Returns an error if extraction or installation fails.
pub fn install_from_archive(archive_path: &Path, update_info: &UpdateInfo) -> Result<()> {
    info!("Installing update from archive: {}", archive_path.display());

    // Determine archive type and extract
    let extension = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let temp_dir = std::env::temp_dir().join("tss-update-extract");
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).map_err(UpdateError::Io)?;
    }
    std::fs::create_dir_all(&temp_dir).map_err(UpdateError::Io)?;

    match extension {
        "zip" => extract_zip(archive_path, &temp_dir)?,
        "gz" => extract_tar_gz(archive_path, &temp_dir)?,
        _ => {
            return Err(UpdateError::InstallFailed(format!(
                "Unsupported archive format: {}",
                extension
            )));
        }
    }

    // Find the binary in the extracted contents
    let new_binary = find_binary_in_dir(&temp_dir)?;

    // Get the current executable path
    let current_exe = std::env::current_exe().map_err(UpdateError::Io)?;

    info!(
        "Replacing {} with {}",
        current_exe.display(),
        new_binary.display()
    );

    // Use self_update's Move to handle the platform-specific replacement
    // Create a temporary path for the backup
    let temp_backup = current_exe.with_extension("backup");

    self_update::Move::from_source(&new_binary)
        .replace_using_temp(&temp_backup)
        .to_dest(&current_exe)
        .map_err(|e| UpdateError::SelfUpdate(e.to_string()))?;

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
    let _ = std::fs::remove_file(archive_path);

    info!("Update {} installed successfully", update_info.new_version);

    Ok(())
}

/// Extract a ZIP archive.
fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    debug!(
        "Extracting ZIP: {} -> {}",
        archive_path.display(),
        dest_dir.display()
    );

    let file = std::fs::File::open(archive_path).map_err(UpdateError::Io)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| UpdateError::InstallFailed(format!("Failed to open ZIP: {e}")))?;

    archive
        .extract(dest_dir)
        .map_err(|e| UpdateError::InstallFailed(format!("Failed to extract ZIP: {e}")))?;

    Ok(())
}

/// Extract a TAR.GZ archive.
fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    debug!(
        "Extracting TAR.GZ: {} -> {}",
        archive_path.display(),
        dest_dir.display()
    );

    let file = std::fs::File::open(archive_path).map_err(UpdateError::Io)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    archive
        .unpack(dest_dir)
        .map_err(|e| UpdateError::InstallFailed(format!("Failed to extract TAR.GZ: {e}")))?;

    Ok(())
}

/// Find the binary in an extracted directory.
fn find_binary_in_dir(dir: &Path) -> Result<std::path::PathBuf> {
    // First, look for the binary directly
    let direct_path = dir.join(BIN_NAME);
    if direct_path.exists() {
        return Ok(direct_path);
    }

    // Look in subdirectories (some archives have a top-level folder)
    for entry in std::fs::read_dir(dir).map_err(UpdateError::Io)? {
        let entry = entry.map_err(UpdateError::Io)?;
        let path = entry.path();

        if path.is_dir() {
            let nested_binary = path.join(BIN_NAME);
            if nested_binary.exists() {
                return Ok(nested_binary);
            }

            // On macOS, look for .app bundle
            #[cfg(target_os = "macos")]
            {
                if path.extension().is_some_and(|e| e == "app") {
                    // The binary is inside Contents/MacOS/
                    let app_binary = path
                        .join("Contents")
                        .join("MacOS")
                        .join("trial-submission-studio");
                    if app_binary.exists() {
                        return Ok(app_binary);
                    }
                }
            }
        }
    }

    Err(UpdateError::InstallFailed(format!(
        "Binary '{}' not found in extracted archive",
        BIN_NAME
    )))
}

/// Restart the application.
///
/// This will start a new instance of the application and exit the current one.
/// Call this after `install_update()` or `install_from_archive()` completes.
pub fn restart_application() -> Result<()> {
    let current_exe = std::env::current_exe().map_err(UpdateError::Io)?;

    info!("Restarting application: {}", current_exe.display());

    // On macOS, if we're in an .app bundle, open the bundle instead
    #[cfg(target_os = "macos")]
    {
        if let Some(app_path) = find_app_bundle(&current_exe) {
            debug!("Restarting via app bundle: {}", app_path.display());
            std::process::Command::new("open")
                .arg("-n") // Open a new instance
                .arg(&app_path)
                .spawn()
                .map_err(|e| UpdateError::InstallFailed(format!("Failed to restart: {e}")))?;

            std::process::exit(0);
        }
    }

    // Generic restart for other platforms
    std::process::Command::new(&current_exe)
        .spawn()
        .map_err(|e| UpdateError::InstallFailed(format!("Failed to restart: {e}")))?;

    std::process::exit(0);
}

/// Find the .app bundle containing the current executable (macOS only).
#[cfg(target_os = "macos")]
fn find_app_bundle(exe_path: &Path) -> Option<std::path::PathBuf> {
    let mut current = exe_path.to_path_buf();

    while let Some(parent) = current.parent() {
        if parent.extension().is_some_and(|e| e == "app") {
            return Some(parent.to_path_buf());
        }
        current = parent.to_path_buf();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin_name() {
        #[cfg(target_os = "windows")]
        assert!(BIN_NAME.ends_with(".exe"));

        #[cfg(not(target_os = "windows"))]
        assert!(!BIN_NAME.contains('.'));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_find_app_bundle() {
        let path = Path::new(
            "/Applications/Trial Submission Studio.app/Contents/MacOS/trial-submission-studio",
        );
        let bundle = find_app_bundle(path);
        assert!(bundle.is_some());
        assert_eq!(
            bundle.unwrap(),
            Path::new("/Applications/Trial Submission Studio.app")
        );
    }
}
