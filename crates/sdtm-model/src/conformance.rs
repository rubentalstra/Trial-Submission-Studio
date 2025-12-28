use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Reject,
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceIssue {
    pub code: String,
    pub message: String,
    pub severity: IssueSeverity,
    pub variable: Option<String>,
    pub count: Option<u64>,
    pub category: Option<String>,
    pub codelist_code: Option<String>,
    pub ct_source: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConformanceReport {
    #[serde(rename = "domain")]
    pub domain_code: String,
    pub issues: Vec<ConformanceIssue>,
}

impl ConformanceReport {
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| matches!(issue.severity, IssueSeverity::Error | IssueSeverity::Reject))
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == IssueSeverity::Warning)
            .count()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }
}

// ============================================================================
// Structured Issue Summary
// ============================================================================

/// Summary of issues by category for display and reporting.
///
/// This structure aggregates conformance issues into a format suitable for
/// rendering in CLI output and reports.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueSummary {
    /// Total error count across all domains.
    pub total_errors: usize,
    /// Total warning count across all domains.
    pub total_warnings: usize,
    /// Total reject count across all domains.
    pub total_rejects: usize,
    /// Issues grouped by category.
    pub by_category: BTreeMap<String, CategorySummary>,
    /// Issues grouped by domain.
    pub by_domain: BTreeMap<String, DomainIssueSummary>,
    /// Sample values for each issue type (limited to avoid excessive output).
    pub samples: BTreeMap<String, Vec<String>>,
}

/// Summary of issues within a category.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategorySummary {
    /// Category name (e.g., "Controlled Terminology", "Completeness").
    pub category: String,
    /// Number of errors in this category.
    pub error_count: usize,
    /// Number of warnings in this category.
    pub warning_count: usize,
}

/// Summary of issues within a domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainIssueSummary {
    /// Domain code.
    pub domain_code: String,
    /// Number of errors in this domain.
    pub error_count: usize,
    /// Number of warnings in this domain.
    pub warning_count: usize,
    /// Number of reject-level issues in this domain.
    pub reject_count: usize,
    /// Total record count for this domain (if available).
    pub record_count: Option<usize>,
}

impl IssueSummary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a summary from conformance reports.
    pub fn from_reports(reports: &[ConformanceReport]) -> Self {
        let mut summary = Self::new();

        for report in reports {
            let domain_code = report.domain_code.to_uppercase();

            // Initialize domain summary if not present
            let domain_summary =
                summary
                    .by_domain
                    .entry(domain_code.clone())
                    .or_insert_with(|| DomainIssueSummary {
                        domain_code: domain_code.clone(),
                        ..Default::default()
                    });

            for issue in &report.issues {
                // Update totals
                match issue.severity {
                    IssueSeverity::Reject => {
                        summary.total_rejects += 1;
                        domain_summary.reject_count += 1;
                    }
                    IssueSeverity::Error => {
                        summary.total_errors += 1;
                        domain_summary.error_count += 1;
                    }
                    IssueSeverity::Warning => {
                        summary.total_warnings += 1;
                        domain_summary.warning_count += 1;
                    }
                }

                // Update category summary
                let category = issue
                    .category
                    .clone()
                    .unwrap_or_else(|| "Other".to_string());
                let cat_summary =
                    summary
                        .by_category
                        .entry(category.clone())
                        .or_insert_with(|| CategorySummary {
                            category: category.clone(),
                            ..Default::default()
                        });

                match issue.severity {
                    IssueSeverity::Error | IssueSeverity::Reject => cat_summary.error_count += 1,
                    IssueSeverity::Warning => cat_summary.warning_count += 1,
                }

                // Store samples (limit to 5 per category)
                let samples = summary.samples.entry(category.clone()).or_default();
                if samples.len() < 5 {
                    // Extract sample values from message if present
                    if let Some(values) = extract_sample_values(&issue.message) {
                        for value in values {
                            if samples.len() < 5 && !samples.contains(&value) {
                                samples.push(value);
                            }
                        }
                    }
                }
            }
        }

        summary
    }

    /// Check if there are any errors or rejects.
    pub fn has_errors(&self) -> bool {
        self.total_errors > 0 || self.total_rejects > 0
    }

    /// Get the total issue count.
    pub fn total_issues(&self) -> usize {
        self.total_errors + self.total_warnings + self.total_rejects
    }

    /// Get domains with errors.
    pub fn domains_with_errors(&self) -> Vec<&String> {
        self.by_domain
            .iter()
            .filter(|(_, s)| s.error_count > 0 || s.reject_count > 0)
            .map(|(code, _)| code)
            .collect()
    }
}

/// Extract sample values from an issue message.
/// Messages often contain "values: X, Y, Z" patterns.
fn extract_sample_values(message: &str) -> Option<Vec<String>> {
    // Look for "values: X, Y, Z" pattern
    if let Some(idx) = message.to_lowercase().find("values:") {
        let rest = &message[idx + 7..];
        let values: Vec<String> = rest
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .take(5)
            .collect();
        if !values.is_empty() {
            return Some(values);
        }
    }
    None
}
