//! Installation step for updates.

use std::fs;
#[cfg(not(target_os = "macos"))]
use std::io::Write;
use std::path::Path;

use crate::error::{Result, UpdateError};
use crate::release::UpdateInfo;

/// Helper binary name (macOS).
#[cfg(target_os = "macos")]
const HELPER_NAME: &str = "tss-updater-helper";

/// Installs an update on desktop platforms (Windows/Linux).
///
/// This extracts the binary and replaces the current executable.
#[cfg(not(target_os = "macos"))]
pub fn install_desktop(data: &[u8], info: &UpdateInfo) -> Result<()> {
    tracing::info!("Installing desktop update: {}", info.version);

    // Extract binary from archive
    let binary = crate::steps::extract::extract_archive(data, &info.asset.name)?;

    // Read the extracted binary
    let binary_data = fs::read(&binary).map_err(|e| {
        UpdateError::Installation(format!("Failed to read extracted binary: {}", e))
    })?;

    // Replace current executable
    replace_current_executable(&binary_data)?;

    tracing::info!("Desktop update installed successfully");
    Ok(())
}

/// Extracts binary bytes from an archive.
///
/// This is a convenience wrapper that extracts the archive and reads the binary.
#[cfg(not(target_os = "macos"))]
pub fn extract_binary(data: &[u8], asset_name: &str) -> Result<Vec<u8>> {
    let binary_path = crate::steps::extract::extract_archive(data, asset_name)?;

    fs::read(&binary_path)
        .map_err(|e| UpdateError::Installation(format!("Failed to read extracted binary: {}", e)))
}

/// Installs an update on macOS by spawning the helper.
///
/// This function spawns the helper binary and exits the current process.
/// It does not return on success.
#[cfg(target_os = "macos")]
pub fn install_macos(extracted_app_path: &Path, info: &UpdateInfo) -> Result<()> {
    use serde::Serialize;
    use std::process::Command;

    tracing::info!("Installing macOS update: {}", info.version);

    /// Configuration passed to the helper binary.
    #[derive(Debug, Serialize)]
    struct HelperConfig {
        new_app_path: std::path::PathBuf,
        current_app_path: std::path::PathBuf,
        parent_pid: u32,
        version: String,
        previous_version: String,
    }

    // Get current bundle path
    let current_app_path = get_current_bundle()?;
    tracing::info!("Current bundle: {:?}", current_app_path);

    // Get helper path
    let helper_path = get_helper_path(&current_app_path)?;
    tracing::info!("Helper path: {:?}", helper_path);

    // Prepare helper config
    let config = HelperConfig {
        new_app_path: extracted_app_path.to_path_buf(),
        current_app_path: current_app_path.clone(),
        parent_pid: std::process::id(),
        version: info.version.clone(),
        previous_version: crate::VERSION.to_string(),
    };

    let config_json = serde_json::to_string(&config)
        .map_err(|e| UpdateError::Installation(format!("Failed to serialize config: {}", e)))?;

    // Write config to a temp file (persists after parent exits)
    let config_dir = extracted_app_path
        .parent()
        .ok_or_else(|| UpdateError::Installation("Invalid extracted path".to_string()))?;
    let config_path = config_dir.join("update_config.json");
    fs::write(&config_path, &config_json)
        .map_err(|e| UpdateError::HelperFailed(format!("Failed to write config file: {}", e)))?;
    tracing::debug!("Config written to: {:?}", config_path);

    // Spawn helper with config file path as argument
    tracing::info!("Spawning update helper and exiting");
    Command::new(&helper_path)
        .arg(&config_path)
        .spawn()
        .map_err(|e| UpdateError::HelperFailed(format!("Failed to spawn helper: {}", e)))?;

    // Exit - helper will complete the update
    tracing::info!("Exiting for helper to complete update");
    std::process::exit(0);
}

/// Stub for non-macOS platforms.
#[cfg(not(target_os = "macos"))]
pub fn install_macos(_extracted_app_path: &Path, _info: &UpdateInfo) -> Result<()> {
    Err(UpdateError::Installation(
        "macOS installation is only supported on macOS".to_string(),
    ))
}

/// Gets the path to the current running .app bundle (macOS).
#[cfg(target_os = "macos")]
fn get_current_bundle() -> Result<std::path::PathBuf> {
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

/// Gets the path to the helper binary inside the current bundle (macOS).
#[cfg(target_os = "macos")]
fn get_helper_path(bundle_path: &Path) -> Result<std::path::PathBuf> {
    let helper_path = bundle_path
        .join("Contents/Helpers/tss-updater-helper.app/Contents/MacOS")
        .join(HELPER_NAME);

    if helper_path.exists() {
        Ok(helper_path)
    } else {
        Err(UpdateError::HelperNotFound)
    }
}

/// Replaces the current executable with the new binary.
#[cfg(not(target_os = "macos"))]
pub fn replace_current_executable(binary: &[u8]) -> Result<()> {
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
