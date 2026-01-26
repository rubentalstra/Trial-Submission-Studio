//! Helper functions for the Validation tab.
//!
//! Contains severity colors, issue categorization, and text utilities.

use iced::Color;
use tss_submit::{Issue, Severity};

// =============================================================================
// SEVERITY COLORS
// =============================================================================

/// Get the color for a severity level.
/// Returns a static Color that works across themes.
pub(super) fn get_severity_color(severity: Severity) -> Color {
    match severity {
        Severity::Reject | Severity::Error => Color::from_rgb(0.90, 0.30, 0.25),
        Severity::Warning => Color::from_rgb(0.95, 0.65, 0.15),
        Severity::Info => Color::from_rgb(0.30, 0.60, 0.85), // Blue for informational
    }
}

// =============================================================================
// ISSUE HELPERS
// =============================================================================

/// Get issue category for display.
pub(super) fn issue_category(issue: &Issue) -> &'static str {
    match issue {
        Issue::RequiredMissing { .. }
        | Issue::RequiredEmpty { .. }
        | Issue::ExpectedMissing { .. }
        | Issue::IdentifierNull { .. } => "Presence",
        Issue::InvalidDate { .. } | Issue::TextTooLong { .. } => "Format",
        Issue::DataTypeMismatch { .. } => "Type",
        Issue::DuplicateSequence { .. } => "Consistency",
        Issue::UsubjidNotInDm { .. }
        | Issue::ParentNotFound { .. }
        | Issue::InvalidRdomain { .. }
        | Issue::RelsubNotInDm { .. }
        | Issue::RelsubNotBidirectional { .. }
        | Issue::RelspecInvalidParent { .. }
        | Issue::RelrecInvalidReference { .. } => "Cross Reference",
        Issue::CtViolation { .. } => "Terminology",
    }
}

/// Truncate message for display in list.
pub(super) fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() <= max_len {
        msg.to_string()
    } else {
        format!("{}...", &msg[..max_len])
    }
}
