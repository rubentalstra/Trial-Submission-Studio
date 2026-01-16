//! macOS-specific update installation.
//!
//! On macOS, we can't use `self_replace` because:
//! 1. macOS apps are bundles (.app directories), not single binaries
//! 2. Replacing the binary inside a signed bundle invalidates the code signature
//! 3. Apple's Gatekeeper requires valid signatures for apps to run
//!
//! Instead, we:
//! 1. Extract the full .app bundle from the downloaded archive
//! 2. Verify its code signature
//! 3. Spawn a helper binary that swaps the bundles after we exit
//! 4. The helper relaunches the new version

use crate::error::{Result, UpdateError};
use crate::release::UpdateInfo;
use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tar::Archive;

/// Expected app bundle name in archives
const APP_BUNDLE_NAME: &str = "Trial Submission Studio.app";

/// Helper binary name
const HELPER_NAME: &str = "tss-updater-helper";

/// Configuration passed to the helper binary
#[derive(Debug, Serialize)]
struct HelperConfig {
    new_app_path: PathBuf,
    current_app_path: PathBuf,
    parent_pid: u32,
}

/// Installs the update and restarts the application.
///
/// On macOS, this:
/// 1. Extracts the .app bundle from the archive to a temp directory
/// 2. Verifies the code signature
/// 3. Spawns the helper binary with configuration
/// 4. Exits the current process (helper will complete the swap and relaunch)
///
/// # Arguments
/// * `data` - The downloaded archive data
/// * `info` - Update metadata
///
/// # Returns
/// This function does not return on success - it exits the process.
pub fn install_and_restart(data: &[u8], info: &UpdateInfo) -> Result<()> {
    tracing::info!("Starting macOS update installation");

    // Create temp directory for extraction
    let temp_dir = tempfile::tempdir().map_err(|e| {
        UpdateError::Installation(format!("Failed to create temp directory: {}", e))
    })?;

    // Extract app bundle
    let new_app_path = extract_app_bundle(data, &info.asset.name, temp_dir.path())?;
    tracing::info!("Extracted app bundle to: {:?}", new_app_path);

    // Verify code signature
    verify_signature(&new_app_path)?;
    tracing::info!("Code signature verified");

    // Get current bundle path
    let current_app_path = get_current_bundle()?;
    tracing::info!("Current bundle: {:?}", current_app_path);

    // Get helper path
    let helper_path = get_helper_path(&current_app_path)?;
    tracing::info!("Helper path: {:?}", helper_path);

    // Prepare helper config
    let config = HelperConfig {
        new_app_path: new_app_path.clone(),
        current_app_path: current_app_path.clone(),
        parent_pid: std::process::id(),
    };

    let config_json = serde_json::to_string(&config)
        .map_err(|e| UpdateError::Installation(format!("Failed to serialize config: {}", e)))?;

    // Keep temp directory alive (don't drop it)
    // The helper will use the extracted bundle
    let temp_path = temp_dir.keep();
    tracing::info!("Temp directory: {:?}", temp_path);

    // Spawn helper
    spawn_helper_and_exit(&helper_path, &config_json)
}

/// Extracts the .app bundle from a tar.gz archive.
fn extract_app_bundle(
    data: &[u8],
    asset_name: &str,
    dest_dir: &std::path::Path,
) -> Result<PathBuf> {
    let asset_lower = asset_name.to_lowercase();

    if asset_lower.ends_with(".tar.gz") || asset_lower.ends_with(".tgz") {
        extract_app_from_tar_gz(data, dest_dir)
    } else if asset_lower.ends_with(".zip") {
        extract_app_from_zip(data, dest_dir)
    } else {
        Err(UpdateError::ArchiveExtraction(format!(
            "Unsupported archive format for macOS: {}",
            asset_name
        )))
    }
}

/// Extracts .app bundle from tar.gz archive
fn extract_app_from_tar_gz(data: &[u8], dest_dir: &std::path::Path) -> Result<PathBuf> {
    tracing::debug!("Extracting app bundle from tar.gz");

    let cursor = Cursor::new(data);
    let decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(decoder);

    // Extract entire archive
    archive
        .unpack(dest_dir)
        .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to extract tar.gz: {}", e)))?;

    // Find the .app bundle
    let app_path = dest_dir.join(APP_BUNDLE_NAME);
    if app_path.exists() {
        return Ok(app_path);
    }

    // Look in subdirectories
    for entry in fs::read_dir(dest_dir)
        .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to read temp dir: {}", e)))?
    {
        let entry = entry
            .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();

        if path.is_dir() {
            let nested_app = path.join(APP_BUNDLE_NAME);
            if nested_app.exists() {
                // Move to top level
                let final_path = dest_dir.join(APP_BUNDLE_NAME);
                fs::rename(&nested_app, &final_path).map_err(|e| {
                    UpdateError::ArchiveExtraction(format!("Failed to move app bundle: {}", e))
                })?;
                return Ok(final_path);
            }
        }
    }

    Err(UpdateError::ArchiveExtraction(format!(
        "App bundle '{}' not found in archive",
        APP_BUNDLE_NAME
    )))
}

/// Extracts .app bundle from ZIP archive
fn extract_app_from_zip(data: &[u8], dest_dir: &std::path::Path) -> Result<PathBuf> {
    tracing::debug!("Extracting app bundle from ZIP");

    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath).map_err(|e| {
                UpdateError::ArchiveExtraction(format!("Failed to create directory: {}", e))
            })?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    UpdateError::ArchiveExtraction(format!("Failed to create parent dir: {}", e))
                })?;
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| {
                UpdateError::ArchiveExtraction(format!("Failed to create file: {}", e))
            })?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| {
                UpdateError::ArchiveExtraction(format!("Failed to write file: {}", e))
            })?;

            // Set executable permissions on macOS
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).ok();
                }
            }
        }
    }

    // Find the .app bundle
    let app_path = dest_dir.join(APP_BUNDLE_NAME);
    if app_path.exists() {
        return Ok(app_path);
    }

    Err(UpdateError::ArchiveExtraction(format!(
        "App bundle '{}' not found in ZIP archive",
        APP_BUNDLE_NAME
    )))
}

/// Verifies the code signature of an app bundle.
fn verify_signature(app_path: &std::path::Path) -> Result<()> {
    let output = Command::new("codesign")
        .args(["--verify", "--deep", "--strict"])
        .arg(app_path)
        .output()
        .map_err(|e| UpdateError::Installation(format!("Failed to run codesign: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(UpdateError::SignatureInvalid(stderr.to_string()))
    }
}

/// Gets the path to the current running .app bundle.
fn get_current_bundle() -> Result<PathBuf> {
    let exe = std::env::current_exe()
        .map_err(|e| UpdateError::Installation(format!("Failed to get current exe: {}", e)))?;

    // Traverse up to find .app bundle
    // Typical path: /Applications/Trial Submission Studio.app/Contents/MacOS/trial-submission-studio
    let mut path = exe.as_path();

    while let Some(parent) = path.parent() {
        if path.extension().is_some_and(|ext| ext == "app") {
            return Ok(path.to_path_buf());
        }
        path = parent;
    }

    Err(UpdateError::Installation(
        "Could not find .app bundle in current executable path".to_string(),
    ))
}

/// Gets the path to the helper binary inside the current bundle.
///
/// The helper is packaged as a nested app bundle following Apple's recommended structure:
/// `Contents/Helpers/tss-updater-helper.app/Contents/MacOS/tss-updater-helper`
fn get_helper_path(bundle_path: &std::path::Path) -> Result<PathBuf> {
    let helper_path = bundle_path
        .join("Contents/Helpers/tss-updater-helper.app/Contents/MacOS")
        .join(HELPER_NAME);

    if helper_path.exists() {
        Ok(helper_path)
    } else {
        Err(UpdateError::HelperNotFound)
    }
}

/// Spawns the helper binary and exits the current process.
///
/// This function does not return - it exits after spawning the helper.
fn spawn_helper_and_exit(helper_path: &std::path::Path, config_json: &str) -> Result<()> {
    tracing::info!("Spawning update helper and exiting");

    let mut child = Command::new(helper_path)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| UpdateError::HelperFailed(format!("Failed to spawn helper: {}", e)))?;

    // Write config to helper's stdin
    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(config_json.as_bytes())
            .map_err(|e| UpdateError::HelperFailed(format!("Failed to write config: {}", e)))?;
    }

    // Close stdin to signal end of input
    drop(child.stdin.take());

    // Exit - helper will complete the update
    tracing::info!("Exiting for helper to complete update");
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_config_serialization() {
        let config = HelperConfig {
            new_app_path: PathBuf::from("/tmp/new.app"),
            current_app_path: PathBuf::from("/Applications/Test.app"),
            parent_pid: 12345,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("new_app_path"));
        assert!(json.contains("12345"));
    }
}
