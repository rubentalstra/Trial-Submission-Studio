//! Utility macros and helpers.
//!
//! Contains common patterns used across the GUI crate.

use std::path::Path;

// =============================================================================
// SEARCH FILTERING
// =============================================================================

/// Case-insensitive search filter.
///
/// Returns `true` if:
/// - The filter is empty (matches everything), or
/// - The text contains the filter (case-insensitive)
///
/// # Example
///
/// ```ignore
/// use crate::util::matches_search;
///
/// assert!(matches_search("USUBJID", ""));
/// assert!(matches_search("USUBJID", "subj"));
/// assert!(matches_search("USUBJID", "SUBJ"));
/// assert!(!matches_search("USUBJID", "xyz"));
/// ```
pub fn matches_search(text: &str, filter: &str) -> bool {
    filter.is_empty() || text.to_lowercase().contains(&filter.to_lowercase())
}

/// Check if any of the provided texts match the search filter.
///
/// Returns `true` if:
/// - The filter is empty (matches everything), or
/// - Any of the texts contain the filter (case-insensitive)
///
/// # Example
///
/// ```ignore
/// use crate::util::matches_search_any;
///
/// assert!(matches_search_any(&["USUBJID", "Subject ID"], "subj"));
/// assert!(!matches_search_any(&["USUBJID", "Subject ID"], "xyz"));
/// ```
pub fn matches_search_any(texts: &[&str], filter: &str) -> bool {
    filter.is_empty() || texts.iter().any(|t| matches_search(t, filter))
}

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
