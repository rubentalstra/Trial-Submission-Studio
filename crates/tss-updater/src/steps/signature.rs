//! Code signature verification (macOS only).

use std::path::Path;
use std::process::Command;

use crate::error::{Result, UpdateError};

/// Verifies the code signature of an app bundle.
///
/// Returns the team ID from the signature if available.
#[cfg(target_os = "macos")]
pub fn verify_signature(app_path: &Path) -> Result<Option<String>> {
    tracing::info!("Verifying code signature: {:?}", app_path);

    // Verify the signature
    let verify_output = Command::new("codesign")
        .args(["--verify", "--deep", "--strict"])
        .arg(app_path)
        .output()
        .map_err(|e| UpdateError::Installation(format!("Failed to run codesign: {}", e)))?;

    if !verify_output.status.success() {
        let stderr = String::from_utf8_lossy(&verify_output.stderr);
        return Err(UpdateError::SignatureInvalid(stderr.to_string()));
    }

    tracing::info!("Code signature verified successfully");

    // Try to get team ID for logging
    let team_id = get_team_id(app_path).ok();

    Ok(team_id)
}

/// Gets the team ID from a signed app bundle.
#[cfg(target_os = "macos")]
fn get_team_id(app_path: &Path) -> Result<String> {
    let output = Command::new("codesign")
        .args(["-dvv"])
        .arg(app_path)
        .output()
        .map_err(|e| UpdateError::Installation(format!("Failed to run codesign: {}", e)))?;

    // Team ID is in stderr (codesign outputs info to stderr)
    let stderr = String::from_utf8_lossy(&output.stderr);

    for line in stderr.lines() {
        if line.starts_with("TeamIdentifier=") {
            let team_id = line
                .strip_prefix("TeamIdentifier=")
                .unwrap_or("")
                .to_string();
            if team_id != "not set" {
                return Ok(team_id);
            }
        }
    }

    Err(UpdateError::Installation("No team ID found".to_string()))
}

/// Stub implementation for non-macOS platforms.
#[cfg(not(target_os = "macos"))]
pub fn verify_signature(_app_path: &Path) -> Result<Option<String>> {
    // No-op on non-macOS platforms
    Ok(None)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_signature_module_exists() {
        // Verify the module compiles - test is intentionally minimal
    }
}
