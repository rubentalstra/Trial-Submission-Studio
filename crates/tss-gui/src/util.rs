//! Utility macros and helpers.
//!
//! Contains common patterns used across the GUI crate.

use std::path::Path;

/// Log failures for best-effort operations that should succeed but aren't critical (#273).
///
/// Use this macro instead of `let _ = ...` for operations that:
/// - Should succeed under normal conditions
/// - Won't cause data loss if they fail
/// - Should still be logged for debugging purposes
///
/// # Example
///
/// ```ignore
/// // Before:
/// let _ = state.settings.save();
///
/// // After:
/// best_effort!(state.settings.save(), "saving settings");
/// ```
macro_rules! best_effort {
    ($expr:expr, $context:literal) => {
        if let Err(e) = $expr {
            tracing::warn!(error = %e, concat!("Best-effort failed: ", $context));
        }
    };
}
pub(crate) use best_effort;

// =============================================================================
// MACOS QUARANTINE CHECK
// =============================================================================

/// Check for macOS quarantine attribute on a path.
///
/// Returns a user-friendly message if the path is quarantined.
#[cfg(target_os = "macos")]
pub fn check_macos_quarantine(path: &Path) -> Option<String> {
    use std::process::Command;

    let path_str = path.to_str()?;

    let output = Command::new("xattr").args(["-l", path_str]).output().ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("com.apple.quarantine") {
        Some(format!(
            "The folder '{}' is quarantined by macOS Gatekeeper. \
            To fix: Right-click the folder in Finder, select 'Open', then confirm in the dialog.",
            path.display()
        ))
    } else {
        None
    }
}

/// Stub for non-macOS platforms.
#[cfg(not(target_os = "macos"))]
pub fn check_macos_quarantine(_path: &Path) -> Option<String> {
    None
}
