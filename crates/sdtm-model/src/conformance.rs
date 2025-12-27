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
    pub rule_id: Option<String>,
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
    /// Issues grouped by rule ID.
    pub by_rule: BTreeMap<String, RuleSummary>,
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
    /// Rule IDs contributing to this category.
    pub rule_ids: Vec<String>,
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

/// Summary of a specific rule's issues.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleSummary {
    /// Rule ID (e.g., "SD0002", "CT2001").
    pub rule_id: String,
    /// Rule description/message.
    pub description: String,
    /// Rule category.
    pub category: Option<String>,
    /// Severity of this rule's violations.
    pub severity: Option<IssueSeverity>,
    /// Number of violations across all domains.
    pub violation_count: u64,
    /// Domains affected by this rule.
    pub affected_domains: Vec<String>,
    /// Sample values (limited).
    pub samples: Vec<String>,
}

impl IssueSummary {
    /// Create a new empty summary.
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

                if let Some(rule_id) = &issue.rule_id
                    && !cat_summary.rule_ids.contains(rule_id)
                {
                    cat_summary.rule_ids.push(rule_id.clone());
                }

                // Update rule summary
                if let Some(rule_id) = &issue.rule_id {
                    let rule_summary =
                        summary
                            .by_rule
                            .entry(rule_id.clone())
                            .or_insert_with(|| RuleSummary {
                                rule_id: rule_id.clone(),
                                description: issue.message.clone(),
                                category: issue.category.clone(),
                                severity: Some(issue.severity),
                                ..Default::default()
                            });

                    rule_summary.violation_count += issue.count.unwrap_or(1);

                    if !rule_summary.affected_domains.contains(&domain_code) {
                        rule_summary.affected_domains.push(domain_code.clone());
                    }
                }

                // Store samples (limit to 5 per issue type)
                if let Some(rule_id) = &issue.rule_id {
                    let samples = summary.samples.entry(rule_id.clone()).or_default();
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

    /// Get the most common rule violations (by count).
    pub fn top_rules(&self, limit: usize) -> Vec<&RuleSummary> {
        let mut rules: Vec<&RuleSummary> = self.by_rule.values().collect();
        rules.sort_by(|a, b| b.violation_count.cmp(&a.violation_count));
        rules.into_iter().take(limit).collect()
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
