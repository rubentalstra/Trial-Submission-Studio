//! Export gating logic - determines which domains can be exported.

use super::types::ExportBypasses;
use crate::state::DomainState;
use cdisc_validate::Severity;

/// Domain export status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainStatus {
    /// Ready to export (all required mapped, no errors/rejects).
    Ready,
    /// Has warnings but can export.
    Warnings,
    /// Blocked from export (missing required mappings or validation errors).
    Blocked,
    /// Not yet processed (no preview run).
    Incomplete,
}

impl DomainStatus {
    /// Check if domain can be exported.
    pub fn can_export(&self) -> bool {
        matches!(self, Self::Ready | Self::Warnings)
    }

    /// Get icon name for egui_phosphor.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Ready => egui_phosphor::regular::CHECK_CIRCLE,
            Self::Warnings => egui_phosphor::regular::WARNING,
            Self::Blocked => egui_phosphor::regular::X_CIRCLE,
            Self::Incomplete => egui_phosphor::regular::CIRCLE,
        }
    }

    /// Get color for UI.
    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Ready => egui::Color32::from_rgb(34, 197, 94), // Green
            Self::Warnings => egui::Color32::from_rgb(234, 179, 8), // Yellow
            Self::Blocked => egui::Color32::from_rgb(239, 68, 68), // Red
            Self::Incomplete => egui::Color32::from_rgb(156, 163, 175), // Gray
        }
    }

    /// Get tooltip text.
    pub fn tooltip(&self) -> &'static str {
        match self {
            Self::Ready => "Ready to export",
            Self::Warnings => "Has warnings but can export",
            Self::Blocked => "Cannot export - has errors or missing mappings",
            Self::Incomplete => "Not yet processed - run preview first",
        }
    }
}

/// Get the export status for a domain.
pub fn get_domain_status(domain: &DomainState, bypasses: &ExportBypasses) -> DomainStatus {
    // No preview yet = incomplete
    if domain.derived.preview.is_none() {
        return DomainStatus::Incomplete;
    }

    let summary = domain.summary();
    let has_required_missing = summary.required_mapped < summary.required_total;

    // Check if blocked by mappings (unless bypassed)
    if has_required_missing {
        if bypasses.developer_mode && bypasses.allow_incomplete_mappings {
            // Bypassed - continue to check validation
        } else {
            return DomainStatus::Blocked;
        }
    }

    // Check validation issues
    if let Some(ref report) = domain.derived.validation {
        let mut has_blocking_issues = false;
        let mut has_warnings = false;

        for issue in &report.issues {
            let severity = issue.default_severity();

            match severity {
                Severity::Reject | Severity::Error => {
                    // Check if this issue is bypassed
                    if !is_issue_bypassed(issue, bypasses) {
                        has_blocking_issues = true;
                    }
                }
                Severity::Warning => {
                    has_warnings = true;
                }
            }
        }

        if has_blocking_issues {
            return DomainStatus::Blocked;
        }

        if has_warnings {
            return DomainStatus::Warnings;
        }
    }

    DomainStatus::Ready
}

/// Check if a specific validation issue is bypassed.
fn is_issue_bypassed(issue: &cdisc_validate::Issue, bypasses: &ExportBypasses) -> bool {
    // Developer mode must be enabled for any bypass
    if !bypasses.developer_mode {
        return false;
    }

    // Allow all errors bypass
    if bypasses.allow_errors {
        return true;
    }

    // Check category bypass
    let category = issue.category();
    if bypasses.bypassed_categories.contains(&category) {
        return true;
    }

    // Check individual rule ID bypass
    let rule_id = issue.rule_id();
    if bypasses.bypassed_rule_ids.contains(rule_id) {
        return true;
    }

    false
}

/// Check if a domain can be exported.
pub fn can_export_domain(domain: &DomainState, bypasses: &ExportBypasses) -> bool {
    get_domain_status(domain, bypasses).can_export()
}

/// Count bypassed issues for a domain (for UI display).
pub fn count_bypassed_issues(domain: &DomainState, bypasses: &ExportBypasses) -> usize {
    if !bypasses.developer_mode {
        return 0;
    }

    let Some(ref report) = domain.derived.validation else {
        return 0;
    };

    report
        .issues
        .iter()
        .filter(|issue| {
            let severity = issue.default_severity();
            matches!(severity, Severity::Reject | Severity::Error)
                && is_issue_bypassed(issue, bypasses)
        })
        .count()
}
