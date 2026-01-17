//! macOS-specific update installation.
//!
//! On macOS, we can't use `self_replace` because:
//! 1. macOS apps are bundles (.app directories), not single binaries
//! 2. Replacing the binary inside a signed bundle invalidates the code signature
//! 3. Apple's Gatekeeper requires valid signatures for apps to run
//!
//! Instead, we:
//! 1. Extract the full .app bundle from the downloaded DMG
//! 2. Verify its code signature
//! 3. Spawn a helper binary that swaps the bundles after we exit
//! 4. The helper relaunches the new version
//!
//! DMG is the only supported format for macOS updates because it perfectly preserves:
//! - Code signatures
//! - Extended attributes (xattrs)
//! - Resource forks
//! - ACLs and file flags

use crate::error::{Result, UpdateError};
use crate::release::UpdateInfo;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

    // Spawn helper with config file path
    spawn_helper_and_exit(&helper_path, &config_json, &temp_path)
}

/// Extracts the .app bundle from a DMG archive.
///
/// DMG is the only supported format for macOS updates because it perfectly preserves:
/// - Code signatures
/// - Extended attributes (xattrs)
/// - Resource forks
/// - ACLs and file flags
fn extract_app_bundle(
    data: &[u8],
    asset_name: &str,
    dest_dir: &std::path::Path,
) -> Result<PathBuf> {
    let asset_lower = asset_name.to_lowercase();

    if !asset_lower.ends_with(".dmg") {
        return Err(UpdateError::ArchiveExtraction(format!(
            "macOS updates require DMG format, got: {}",
            asset_name
        )));
    }

    tracing::info!(
        "Extracting app bundle from DMG: {} ({} bytes)",
        asset_name,
        data.len()
    );

    // Write DMG to temp file
    let dmg_path = dest_dir.join("update.dmg");
    tracing::debug!("Writing DMG to: {:?}", dmg_path);
    fs::write(&dmg_path, data)
        .map_err(|e| UpdateError::ArchiveExtraction(format!("Failed to write DMG: {}", e)))?;
    tracing::debug!("DMG written successfully");

    // Create mount point
    let mount_point = dest_dir.join("dmg_mount");
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
        let stdout = String::from_utf8_lossy(&attach_output.stdout);
        tracing::error!(
            "hdiutil attach failed - stdout: {}, stderr: {}",
            stdout,
            stderr
        );
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
    let dest_app = dest_dir.join(APP_BUNDLE_NAME);
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
        tracing::error!("ditto copy failed: {}", stderr);
        return Err(UpdateError::ArchiveExtraction(format!(
            "ditto copy failed: {}",
            stderr
        )));
    }

    tracing::info!("Extracted app bundle to: {:?}", dest_app);
    Ok(dest_app)
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

    tracing::debug!("Current executable path: {:?}", exe);

    // Traverse up to find .app bundle
    // Typical path: /Applications/Trial Submission Studio.app/Contents/MacOS/trial-submission-studio
    let mut path = exe.as_path();

    while let Some(parent) = path.parent() {
        if path.extension().is_some_and(|ext| ext == "app") {
            return Ok(path.to_path_buf());
        }
        path = parent;
    }

    tracing::warn!("Not running from an app bundle. Executable path: {:?}", exe);
    Err(UpdateError::NotInAppBundle)
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
/// Config is written to a file (rather than stdin) to avoid race conditions
/// when the parent exits before the helper has fully read the input.
fn spawn_helper_and_exit(
    helper_path: &std::path::Path,
    config_json: &str,
    temp_dir: &std::path::Path,
) -> Result<()> {
    tracing::info!("Spawning update helper and exiting");

    // Write config to file (persists after parent exits, avoiding race condition)
    let config_path = temp_dir.join("update_config.json");
    fs::write(&config_path, config_json)
        .map_err(|e| UpdateError::HelperFailed(format!("Failed to write config file: {}", e)))?;
    tracing::debug!("Config written to: {:?}", config_path);

    // Spawn helper with config file path as argument
    Command::new(helper_path)
        .arg(&config_path)
        .spawn()
        .map_err(|e| UpdateError::HelperFailed(format!("Failed to spawn helper: {}", e)))?;

    // Safe to exit - config file persists for helper to read
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
