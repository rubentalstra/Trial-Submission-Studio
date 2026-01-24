//! Utility macros and helpers.
//!
//! Contains common patterns used across the GUI crate.

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
