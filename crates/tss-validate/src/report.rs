//! Validation report containing all issues for a domain.

use serde::{Deserialize, Serialize};

use crate::issue::{Issue, Severity};
use crate::rules::RuleRegistry;

/// Validation report for a domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    pub domain: String,
    pub issues: Vec<Issue>,
}

impl ValidationReport {
    /// Create an empty report for a domain.
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            issues: Vec::new(),
        }
    }

    /// Add an issue to the report.
    pub fn add(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    /// Check if the report has any issues.
    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    /// Total number of issues.
    pub fn len(&self) -> usize {
        self.issues.len()
    }

    /// Count of errors (Error + Reject severity).
    pub fn error_count(&self, registry: Option<&RuleRegistry>) -> usize {
        self.issues
            .iter()
            .filter(|i| matches!(i.severity(registry), Severity::Error | Severity::Reject))
            .count()
    }

    /// Count of warnings.
    pub fn warning_count(&self, registry: Option<&RuleRegistry>) -> usize {
        self.issues
            .iter()
            .filter(|i| matches!(i.severity(registry), Severity::Warning))
            .count()
    }

    /// Check if report has any errors.
    pub fn has_errors(&self, registry: Option<&RuleRegistry>) -> bool {
        self.error_count(registry) > 0
    }

    /// Get issues sorted by severity (Reject first, then Error, then Warning).
    pub fn sorted_by_severity(&self, registry: Option<&RuleRegistry>) -> Vec<&Issue> {
        let mut issues: Vec<_> = self.issues.iter().collect();
        issues.sort_by_key(|i| match i.severity(registry) {
            Severity::Reject => 0,
            Severity::Error => 1,
            Severity::Warning => 2,
        });
        issues
    }
}
