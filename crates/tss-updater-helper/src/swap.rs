//! Bundle swap with rollback support.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of a successful bundle swap.
pub struct SwapResult {
    /// Path to the backup bundle.
    pub backup_path: PathBuf,
}

/// Performs the swap of app bundles using ditto (works across filesystems).
///
/// Steps:
/// 1. Remove old backup if exists
/// 2. Move current → backup (rename, same filesystem)
/// 3. Copy new → current (ditto, cross-filesystem safe)
/// 4. Clean up temp new bundle
pub fn swap_bundles(new_app: &Path, current_app: &Path) -> Result<SwapResult, String> {
    eprintln!(
        "[helper] Swapping bundles: {:?} -> {:?}",
        new_app, current_app
    );

    // Create backup path
    let backup_path = current_app.with_extension("app.backup");

    // Remove old backup if exists
    if backup_path.exists() {
        eprintln!("[helper] Removing old backup: {:?}", backup_path);
        fs::remove_dir_all(&backup_path)
            .map_err(|e| format!("Failed to remove old backup: {}", e))?;
    }

    // Move current to backup (same filesystem, rename works)
    eprintln!("[helper] Moving current app to backup");
    fs::rename(current_app, &backup_path)
        .map_err(|e| format!("Failed to move current app to backup: {}", e))?;
    eprintln!("[helper] Current app moved to backup: {:?}", backup_path);

    // Copy new app using ditto (works across filesystems, preserves metadata)
    eprintln!("[helper] Copying new app with ditto");
    let copy_result = Command::new("ditto").arg(new_app).arg(current_app).output();

    match copy_result {
        Ok(output) if output.status.success() => {
            eprintln!("[helper] New app installed: {:?}", current_app);

            // Clean up the source (temp) app bundle
            if let Err(e) = fs::remove_dir_all(new_app) {
                eprintln!("[helper] Warning: Failed to clean up temp app: {}", e);
            }

            Ok(SwapResult { backup_path })
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[helper] ditto copy failed: {}", stderr);

            // Rollback: restore backup
            rollback(&backup_path, current_app)?;

            Err(format!("Failed to copy new app with ditto: {}", stderr))
        }
        Err(e) => {
            // Rollback: restore backup
            eprintln!("[helper] ditto command failed: {}", e);

            rollback(&backup_path, current_app)?;

            Err(format!("Failed to run ditto: {}", e))
        }
    }
}

/// Performs rollback by restoring the backup.
fn rollback(backup_path: &Path, current_app: &Path) -> Result<(), String> {
    eprintln!("[helper] Rolling back...");

    // If current_app exists (partial copy), remove it first
    if current_app.exists() {
        let _ = fs::remove_dir_all(current_app);
    }

    fs::rename(backup_path, current_app)
        .map_err(|e| format!("Install failed and rollback failed: {}", e))?;

    eprintln!("[helper] Rollback complete");
    Ok(())
}

/// Cleans up the backup after successful update.
pub fn cleanup_backup(backup_path: &Path) {
    eprintln!("[helper] Cleaning up backup: {:?}", backup_path);

    // Give the new app a moment to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    if let Err(e) = fs::remove_dir_all(backup_path) {
        eprintln!("[helper] Warning: Failed to clean up backup: {}", e);
    } else {
        eprintln!("[helper] Backup cleaned up successfully");
    }
}
