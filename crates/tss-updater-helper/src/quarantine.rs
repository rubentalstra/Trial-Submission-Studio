//! Quarantine attribute removal.

use std::path::Path;
use std::process::Command;

/// Removes the quarantine extended attribute from a file or directory.
///
/// macOS marks downloaded files with a quarantine attribute that can cause
/// issues when executing apps. This function removes that attribute.
pub fn remove_quarantine(app_path: &Path) -> Result<(), String> {
    eprintln!(
        "[helper] Removing quarantine attribute from: {:?}",
        app_path
    );

    let output = Command::new("xattr")
        .args(["-rd", "com.apple.quarantine"])
        .arg(app_path)
        .output()
        .map_err(|e| format!("Failed to run xattr: {}", e))?;

    // xattr returns success even if the attribute doesn't exist
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Ignore "No such xattr" errors
        if !stderr.contains("No such xattr") {
            eprintln!("[helper] Warning: xattr removal failed: {}", stderr);
        }
    } else {
        eprintln!("[helper] Quarantine attribute removed");
    }

    Ok(())
}
