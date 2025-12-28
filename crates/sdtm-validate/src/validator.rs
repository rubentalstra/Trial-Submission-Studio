//! Clean SDTM validation per SDTM_CT_relationships.md
//!
//! This module provides straightforward validation rules:
//!
//! ## Core Designation Rules (from Variables.csv)
//!
//! - **Req** (Required): Column must exist, values cannot be null → **Error**
//! - **Exp** (Expected): Column should exist when applicable → **Warning**
//! - **Perm** (Permissible): Optional, no issue if missing
//!
//! ## CT Validation Rules (from SDTM_CT_relationships.md)
//!
//! - **Extensible=No**: Value not in allowed set → **Error**
//! - **Extensible=Yes**: Value not in allowed set → **Warning**
//!
//! ## Variable Format Rules
//!
//! - **--DTC**: Must be valid ISO 8601 datetime format
//! - **--TESTCD/QNAM**: Must be ≤8 chars, start with letter/underscore, alphanumeric only

use std::collections::BTreeSet;

use polars::prelude::{AnyValue, DataFrame};
use serde::Serialize;

use sdtm_model::ct::CtRegistry;
use sdtm_model::{CaseInsensitiveLookup, Domain, Variable};

/// Issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A validation issue.
#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub severity: Severity,
    pub category: String,
    pub variable: Option<String>,
    pub message: String,
    pub count: Option<u64>,
    pub codelist_code: Option<String>,
}

/// Validation report for a domain.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ValidationReport {
    pub domain: String,
    pub issues: Vec<Issue>,
}

impl ValidationReport {
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            issues: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .count()
    }
}

/// Validation context.
pub struct Validator<'a> {
    ct_registry: Option<&'a CtRegistry>,
    preferred_catalogs: Option<Vec<String>>,
}

impl<'a> Validator<'a> {
    /// Create a new validator.
    pub fn new() -> Self {
        Self {
            ct_registry: None,
            preferred_catalogs: None,
        }
    }

    /// Set the CT registry for codelist validation.
    pub fn with_ct(mut self, registry: &'a CtRegistry) -> Self {
        self.ct_registry = Some(registry);
        self
    }

    /// Set preferred CT catalogs (e.g., ["SDTM CT"] for SDTM studies).
    pub fn with_preferred_catalogs(mut self, catalogs: Vec<String>) -> Self {
        self.preferred_catalogs = Some(catalogs);
        self
    }

    /// Validate a domain DataFrame against its metadata.
    pub fn validate(&self, domain: &Domain, df: &DataFrame) -> ValidationReport {
        let mut report = ValidationReport::new(&domain.code);
        let columns = CaseInsensitiveLookup::new(df.get_column_names_owned());

        for var in &domain.variables {
            // Core designation checks
            report.issues.extend(self.check_core(var, df, &columns));

            // CT validation
            if let Some(registry) = self.ct_registry {
                report
                    .issues
                    .extend(self.check_ct(var, df, &columns, registry));
            }

            // Format checks
            report.issues.extend(self.check_format(var, df, &columns));
        }

        report
    }

    /// Check Core designation rules (Req/Exp/Perm).
    fn check_core(
        &self,
        var: &Variable,
        df: &DataFrame,
        columns: &CaseInsensitiveLookup,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();
        let core = var.core.as_deref().unwrap_or("").to_uppercase();

        match core.as_str() {
            "REQ" => {
                // Required: column must exist
                let Some(col_name) = columns.get(&var.name) else {
                    issues.push(Issue {
                        severity: Severity::Error,
                        category: "Required Variable Missing".to_string(),
                        variable: Some(var.name.clone()),
                        message: format!("Required variable {} not found", var.name),
                        count: None,
                        codelist_code: None,
                    });
                    return issues;
                };

                // Required: values cannot be null
                if let Some(missing) = count_missing(df, col_name)
                    && missing > 0
                {
                    issues.push(Issue {
                        severity: Severity::Error,
                        category: "Required Value Missing".to_string(),
                        variable: Some(var.name.clone()),
                        message: format!(
                            "Required variable {} has {} null value(s)",
                            var.name, missing
                        ),
                        count: Some(missing),
                        codelist_code: None,
                    });
                }
            }
            "EXP" => {
                // Expected: column should exist
                if columns.get(&var.name).is_none() {
                    issues.push(Issue {
                        severity: Severity::Warning,
                        category: "Expected Variable Missing".to_string(),
                        variable: Some(var.name.clone()),
                        message: format!("Expected variable {} not found", var.name),
                        count: None,
                        codelist_code: None,
                    });
                }
            }
            _ => {
                // Permissible: no check needed
            }
        }

        issues
    }

    /// Check CT validation rules.
    fn check_ct(
        &self,
        var: &Variable,
        df: &DataFrame,
        columns: &CaseInsensitiveLookup,
        registry: &CtRegistry,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Get codelist code(s) from variable metadata
        let codelist_codes = match &var.codelist_code {
            Some(codes) if !codes.is_empty() => codes,
            _ => return issues,
        };

        // Get column
        let Some(col_name) = columns.get(&var.name) else {
            return issues;
        };

        // Resolve each codelist and collect valid values
        let mut valid_values: BTreeSet<String> = BTreeSet::new();
        let mut extensible = true;
        let mut codelist_code = String::new();
        let mut codelist_name = String::new();

        for code in codelist_codes
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            if let Some(resolved) = registry.resolve(code, self.preferred_catalogs.as_deref()) {
                // Union of valid values across codelists
                for term in resolved.codelist.terms.values() {
                    valid_values.insert(term.submission_value.to_uppercase());
                    // Also add synonyms
                    for syn in &term.synonyms {
                        valid_values.insert(syn.to_uppercase());
                    }
                }
                // Non-extensible if ANY codelist is non-extensible
                extensible = extensible && resolved.codelist.extensible;
                if codelist_code.is_empty() {
                    codelist_code = resolved.codelist.code.clone();
                    codelist_name = resolved.codelist.name.clone();
                }
            }
        }

        if valid_values.is_empty() {
            return issues;
        }

        // Check values
        let invalid = collect_invalid_values(df, col_name, &valid_values);
        if invalid.is_empty() {
            return issues;
        }

        let severity = if extensible {
            Severity::Warning
        } else {
            Severity::Error
        };

        let mut examples: Vec<_> = invalid.iter().take(5).cloned().collect();
        examples.sort();
        let examples_str = examples.join(", ");

        issues.push(Issue {
            severity,
            category: codelist_code.clone(),
            variable: Some(var.name.clone()),
            message: format!(
                "Variable {} has {} value(s) not in {} codelist ({}): {}",
                var.name,
                invalid.len(),
                codelist_name,
                codelist_code,
                examples_str
            ),
            count: Some(invalid.len() as u64),
            codelist_code: Some(codelist_code),
        });

        issues
    }

    /// Check variable format rules (--DTC, --TESTCD, etc.).
    fn check_format(
        &self,
        var: &Variable,
        df: &DataFrame,
        columns: &CaseInsensitiveLookup,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();
        let name_upper = var.name.to_uppercase();

        // Check --DTC variables for ISO 8601 format
        if name_upper.ends_with("DTC")
            && let Some(col_name) = columns.get(&var.name)
            && let Some(invalid) = count_invalid_iso8601(df, col_name)
            && invalid > 0
        {
            issues.push(Issue {
                severity: Severity::Error,
                category: "Invalid ISO 8601".to_string(),
                variable: Some(var.name.clone()),
                message: format!(
                    "Variable {} has {} value(s) not in ISO 8601 format",
                    var.name, invalid
                ),
                count: Some(invalid),
                codelist_code: None,
            });
        }

        // Check --TESTCD and QNAM for format rules
        if (name_upper.ends_with("TESTCD") || name_upper == "QNAM")
            && let Some(col_name) = columns.get(&var.name)
        {
            let (invalid, examples) = check_testcd_format(df, col_name);
            if invalid > 0 {
                issues.push(Issue {
                        severity: Severity::Error,
                        category: "Invalid TESTCD Format".to_string(),
                        variable: Some(var.name.clone()),
                        message: format!(
                            "Variable {} has {} value(s) with invalid format (must be ≤8 chars, start with letter/underscore): {}",
                            var.name, invalid, examples.join(", ")
                        ),
                        count: Some(invalid),
                        codelist_code: None,
                    });
            }
        }

        issues
    }
}

impl Default for Validator<'_> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper functions
// ============================================================================

fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::Null => String::new(),
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        other => other.to_string(),
    }
}

fn is_missing(value: &AnyValue) -> bool {
    match value {
        AnyValue::Null => true,
        AnyValue::String(s) => s.trim().is_empty(),
        AnyValue::StringOwned(s) => s.trim().is_empty(),
        _ => false,
    }
}

fn count_missing(df: &DataFrame, column: &str) -> Option<u64> {
    let series = df.column(column).ok()?;
    let mut count = 0u64;
    for idx in 0..df.height() {
        if is_missing(&series.get(idx).unwrap_or(AnyValue::Null)) {
            count += 1;
        }
    }
    Some(count)
}

fn collect_invalid_values(
    df: &DataFrame,
    column: &str,
    valid: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut invalid = BTreeSet::new();
    let Ok(series) = df.column(column) else {
        return invalid;
    };

    for idx in 0..df.height() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !valid.contains(&trimmed.to_uppercase()) {
            invalid.insert(trimmed.to_string());
        }
    }
    invalid
}

fn count_invalid_iso8601(df: &DataFrame, column: &str) -> Option<u64> {
    let series = df.column(column).ok()?;
    let mut count = 0u64;
    for idx in 0..df.height() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !is_valid_iso8601(trimmed) {
            count += 1;
        }
    }
    Some(count)
}

/// Basic ISO 8601 validation (date or datetime).
fn is_valid_iso8601(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return true;
    }

    // SDTM allows partial dates: YYYY, YYYY-MM, YYYY-MM-DD, etc.
    // Also datetime: YYYY-MM-DDTHH:MM:SS
    let patterns = [
        r"^\d{4}$",                                              // YYYY
        r"^\d{4}-\d{2}$",                                        // YYYY-MM
        r"^\d{4}-\d{2}-\d{2}$",                                  // YYYY-MM-DD
        r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$",                      // YYYY-MM-DDTHH:MM
        r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}$",                // YYYY-MM-DDTHH:MM:SS
        r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+$",           // YYYY-MM-DDTHH:MM:SS.sss
        r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2}$", // with timezone
    ];

    for pattern in patterns {
        if regex::Regex::new(pattern)
            .map(|r| r.is_match(trimmed))
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

fn check_testcd_format(df: &DataFrame, column: &str) -> (u64, Vec<String>) {
    let mut invalid_count = 0u64;
    let mut examples = Vec::new();
    let Ok(series) = df.column(column) else {
        return (0, Vec::new());
    };

    for idx in 0..df.height() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !is_valid_testcd(trimmed) {
            invalid_count += 1;
            if examples.len() < 5 {
                examples.push(trimmed.to_string());
            }
        }
    }
    (invalid_count, examples)
}

/// TESTCD/QNAM must be ≤8 chars, start with letter or underscore, alphanumeric only.
fn is_valid_testcd(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 8 {
        return false;
    }

    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    // Must start with letter or underscore
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use sdtm_model::VariableType;

    fn make_df(col: &str, values: &[&str]) -> DataFrame {
        df! {
            col => values
        }
        .unwrap()
    }

    fn make_var(name: &str, core: &str) -> Variable {
        Variable {
            name: name.to_string(),
            label: None,
            data_type: VariableType::Char,
            length: None,
            role: None,
            core: Some(core.to_string()),
            codelist_code: None,
            order: None,
        }
    }

    fn make_domain(code: &str, vars: Vec<Variable>) -> Domain {
        Domain {
            code: code.to_string(),
            description: None,
            class_name: None,
            dataset_class: None,
            label: None,
            structure: None,
            dataset_name: None,
            variables: vars,
        }
    }

    #[test]
    fn test_required_missing_column() {
        let domain = make_domain("DM", vec![make_var("USUBJID", "Req")]);
        let df = make_df("OTHER", &["value"]);

        let validator = Validator::new();
        let report = validator.validate(&domain, &df);

        assert!(report.has_errors());
        assert_eq!(report.error_count(), 1);
        assert!(report.issues[0].message.contains("Required variable"));
    }

    #[test]
    fn test_required_null_values() {
        let domain = make_domain("DM", vec![make_var("USUBJID", "Req")]);
        let df = make_df("USUBJID", &["SUBJ001", "", "SUBJ003"]);

        let validator = Validator::new();
        let report = validator.validate(&domain, &df);

        assert!(report.has_errors());
        assert!(report.issues[0].message.contains("null value"));
        assert_eq!(report.issues[0].count, Some(1));
    }

    #[test]
    fn test_expected_missing() {
        let domain = make_domain("DM", vec![make_var("ARM", "Exp")]);
        let df = make_df("OTHER", &["value"]);

        let validator = Validator::new();
        let report = validator.validate(&domain, &df);

        assert!(!report.has_errors());
        assert_eq!(report.warning_count(), 1);
        assert!(report.issues[0].message.contains("Expected variable"));
    }

    #[test]
    fn test_permissible_missing_no_issue() {
        let domain = make_domain("DM", vec![make_var("ARMCD", "Perm")]);
        let df = make_df("OTHER", &["value"]);

        let validator = Validator::new();
        let report = validator.validate(&domain, &df);

        assert!(report.issues.is_empty());
    }

    #[test]
    fn test_iso8601_validation() {
        assert!(is_valid_iso8601("2024-01-15"));
        assert!(is_valid_iso8601("2024"));
        assert!(is_valid_iso8601("2024-01"));
        assert!(is_valid_iso8601("2024-01-15T10:30:00"));
        assert!(is_valid_iso8601(""));

        assert!(!is_valid_iso8601("15-01-2024"));
        assert!(!is_valid_iso8601("01/15/2024"));
        assert!(!is_valid_iso8601("invalid"));
    }

    #[test]
    fn test_testcd_validation() {
        assert!(is_valid_testcd("LBTEST01"));
        assert!(is_valid_testcd("_TEST"));
        assert!(is_valid_testcd("A"));

        assert!(!is_valid_testcd("TOOLONGVALUE")); // > 8 chars
        assert!(!is_valid_testcd("1TEST")); // starts with number
        assert!(!is_valid_testcd("TEST-1")); // contains hyphen
        assert!(!is_valid_testcd("")); // empty
    }
}
