//! Code signature verification.

use std::path::Path;
use std::process::Command;

/// Verifies the code signature of an app bundle.
///
/// Returns Ok(()) if the signature is valid, Err with details otherwise.
pub fn verify_signature(app_path: &Path) -> Result<(), String> {
    eprintln!("[helper] Verifying code signature: {:?}", app_path);

    let output = Command::new("codesign")
        .args(["--verify", "--deep", "--strict"])
        .arg(app_path)
        .output()
        .map_err(|e| format!("Failed to run codesign: {}", e))?;

    if output.status.success() {
        eprintln!("[helper] Code signature valid");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Code signature verification failed: {}", stderr))
    }
}

/// Gets the team ID from a signed app bundle (for logging).
#[allow(dead_code)]
pub fn get_team_id(app_path: &Path) -> Option<String> {
    let output = Command::new("codesign")
        .args(["-dvv"])
        .arg(app_path)
        .output()
        .ok()?;

    // Team ID is in stderr (codesign outputs info to stderr)
    let stderr = String::from_utf8_lossy(&output.stderr);

    for line in stderr.lines() {
        if line.starts_with("TeamIdentifier=") {
            let team_id = line
                .strip_prefix("TeamIdentifier=")
                .unwrap_or("")
                .to_string();
            if team_id != "not set" {
                return Some(team_id);
            }
        }
    }

    None
}
