//! Utility macros and helpers.
//!
//! Contains common patterns used across the GUI crate.

use std::path::{Path, PathBuf};

use crate::error::GuiError;

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
// PATH SECURITY
// =============================================================================

// These functions are public API for use by handlers
#[allow(dead_code)]
/// Validate that a path is safe and contained within an allowed directory.
///
/// This prevents path traversal attacks by ensuring the resolved path
/// stays within the expected directory.
///
/// # Arguments
///
/// * `path` - The path to validate
/// * `allowed_dir` - The directory that must contain the path
///
/// # Returns
///
/// The canonicalized path if valid, or a `GuiError::Security` if path traversal
/// was detected.
///
/// # Example
///
/// ```ignore
/// let safe_path = validate_path_security(&user_path, &study_dir)?;
/// ```
pub fn validate_path_security(path: &Path, allowed_dir: &Path) -> Result<PathBuf, GuiError> {
    // Canonicalize both paths to resolve symlinks and ..
    let canonical_path = path.canonicalize().map_err(|e| GuiError::FileOperation {
        reason: format!("Cannot resolve path '{}': {}", path.display(), e),
    })?;

    let canonical_allowed = allowed_dir
        .canonicalize()
        .map_err(|e| GuiError::FileOperation {
            reason: format!(
                "Cannot resolve allowed directory '{}': {}",
                allowed_dir.display(),
                e
            ),
        })?;

    // Check containment
    if !canonical_path.starts_with(&canonical_allowed) {
        return Err(GuiError::path_traversal(path.display()));
    }

    Ok(canonical_path)
}

#[allow(dead_code)]
/// Validate that a path is a regular file within an allowed directory.
///
/// Combines path traversal protection with file type checking.
pub fn validate_file_path(path: &Path, allowed_dir: &Path) -> Result<PathBuf, GuiError> {
    let canonical = validate_path_security(path, allowed_dir)?;

    // Check it's a regular file (not a directory or special file)
    let metadata = std::fs::metadata(&canonical).map_err(|e| GuiError::FileOperation {
        reason: format!("Cannot read file metadata for '{}': {}", path.display(), e),
    })?;

    if !metadata.is_file() {
        return Err(GuiError::FileOperation {
            reason: format!("Path '{}' is not a regular file", path.display()),
        });
    }

    Ok(canonical)
}

#[allow(dead_code)]
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
