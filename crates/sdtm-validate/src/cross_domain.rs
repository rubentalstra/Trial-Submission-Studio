//! Cross-domain validation utilities.
//!
//! This module provides utilities for cross-domain analysis.
//! All validation is now CT-based only.

use std::collections::BTreeMap;

use polars::prelude::DataFrame;

use sdtm_model::{ValidationIssue, ValidationReport};

/// Input for cross-domain validation.
pub struct CrossDomainValidationInput<'a> {
    /// All domain frames indexed by domain code (uppercase).
    pub frames: &'a BTreeMap<String, &'a DataFrame>,
    /// Base domain codes (not dataset names) for split detection.
    /// Maps dataset name (e.g., "LBCH") to base domain (e.g., "LB").
    pub split_mappings: Option<&'a BTreeMap<String, String>>,
}

/// Result of cross-domain validation.
#[derive(Debug, Default)]
pub struct CrossDomainValidationResult {
    /// Issues found, grouped by domain code.
    pub issues_by_domain: BTreeMap<String, Vec<ValidationIssue>>,
}

impl CrossDomainValidationResult {
    /// Convert to validation reports.
    pub fn into_reports(self) -> Vec<ValidationReport> {
        self.issues_by_domain
            .into_iter()
            .map(|(domain_code, issues)| ValidationReport {
                domain_code,
                issues,
            })
            .collect()
    }

    /// Merge issues into existing report map.
    pub fn merge_into(self, reports: &mut BTreeMap<String, ValidationReport>) {
        for (domain_code, issues) in self.issues_by_domain {
            reports
                .entry(domain_code.clone())
                .or_insert_with(|| ValidationReport {
                    domain_code,
                    issues: Vec::new(),
                })
                .issues
                .extend(issues);
        }
    }

    /// Check if any violations were found.
    pub fn has_issues(&self) -> bool {
        !self.issues_by_domain.is_empty()
    }

    /// Total issue count.
    pub fn total_issues(&self) -> usize {
        self.issues_by_domain
            .values()
            .map(|issues| issues.len())
            .sum()
    }
}

/// Run cross-domain validation (CT-based only, currently empty).
pub fn validate_cross_domain(
    _input: CrossDomainValidationInput<'_>,
) -> CrossDomainValidationResult {
    // CT validation is done per-domain in lib.rs
    // Cross-domain validation is now empty - CT is our only source of truth
    CrossDomainValidationResult::default()
}

/// Infer base domain from dataset name.
/// E.g., "LBCH" -> "LB", "QSFT" -> "QS", "DM" -> "DM"
pub fn infer_base_domain(dataset_name: &str) -> String {
    let name = dataset_name.to_uppercase();

    // SUPPXX datasets -> base is the XX part
    if name.starts_with("SUPP") && name.len() > 4 {
        return name[4..].to_string();
    }

    // Special relationship datasets
    if name.starts_with("RELREC")
        || name.starts_with("RELSPEC")
        || name.starts_with("RELSUB")
        || name.starts_with("SUPPQUAL")
    {
        return name;
    }

    // Standard 2-letter domains that may have suffixes
    const TWO_LETTER_DOMAINS: &[&str] = &[
        "AE", "AG", "BE", "BS", "CE", "CM", "CO", "CP", "CV", "DA", "DD", "DM", "DS", "DV", "EC",
        "EG", "EX", "FA", "FT", "GF", "HO", "IE", "IS", "LB", "MB", "MH", "MI", "MK", "ML", "MS",
        "NV", "OE", "OI", "PC", "PE", "PP", "PR", "QS", "RE", "RP", "RS", "SC", "SE", "SM", "SR",
        "SS", "SU", "SV", "TA", "TD", "TE", "TI", "TM", "TR", "TS", "TU", "TV", "UR", "VS",
    ];

    // Check if starts with a known 2-letter domain
    if name.len() >= 2 {
        let prefix = &name[..2];
        if TWO_LETTER_DOMAINS.contains(&prefix) {
            return prefix.to_string();
        }
    }

    // Default: return as-is
    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_base_domain() {
        assert_eq!(infer_base_domain("LB"), "LB");
        assert_eq!(infer_base_domain("LBCH"), "LB");
        assert_eq!(infer_base_domain("LBHE"), "LB");
        assert_eq!(infer_base_domain("QS"), "QS");
        assert_eq!(infer_base_domain("QSFT"), "QS");
        assert_eq!(infer_base_domain("DM"), "DM");
        assert_eq!(infer_base_domain("SUPPDM"), "DM");
        assert_eq!(infer_base_domain("SUPPLB"), "LB");
        assert_eq!(infer_base_domain("RELREC"), "RELREC");
    }
}
