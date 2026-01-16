//! Desktop (Windows/Linux) update installation using self_replace.
//!
//! On Windows and Linux, we can use the simpler `self_replace` approach:
//! 1. Extract the binary from the downloaded archive
//! 2. Replace the current executable using self_replace
//! 3. Restart the application

use crate::error::{Result, UpdateError};
use crate::install::{extract_binary, replace_current_executable, restart_application};
use crate::release::UpdateInfo;

/// Installs the update and restarts the application.
///
/// On Windows/Linux, this:
/// 1. Extracts the binary from the archive
/// 2. Replaces the current executable using `self_replace`
/// 3. Restarts the application
///
/// # Arguments
/// * `data` - The downloaded archive data
/// * `info` - Update metadata
///
/// # Returns
/// This function does not return on success - it restarts the application.
pub fn install_and_restart(data: &[u8], info: &UpdateInfo) -> Result<()> {
    tracing::info!("Starting desktop update installation");

    // Extract binary from archive
    let binary = extract_binary(data, &info.asset.name)?;
    tracing::info!("Extracted binary ({} bytes)", binary.len());

    // Replace current executable
    replace_current_executable(&binary)?;
    tracing::info!("Executable replaced");

    // Restart application
    restart_application()
}
