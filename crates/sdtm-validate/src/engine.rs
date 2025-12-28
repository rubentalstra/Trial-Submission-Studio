//! Rule Engine for executing dynamically generated rules.
//!
//! This engine takes rules generated from metadata (via `RuleGenerator`) and
//! executes them against domain DataFrames, producing structured conformance issues.
//!
//! All rules use official P21 rule IDs from the loaded CSV. No internal or
//! custom rule IDs are used.

use std::collections::BTreeSet;

use polars::prelude::{AnyValue, DataFrame};

use sdtm_core::validate_iso8601;
use sdtm_ingest::{any_to_string, is_missing_value};
use sdtm_model::{CaseInsensitiveLookup, ConformanceIssue, ConformanceReport, IssueSeverity};
use sdtm_standards::assumptions::{GeneratedRule, RuleContext, RuleSeverity};

/// Rule engine that executes generated rules against domain data.
#[derive(Debug, Default)]
pub struct RuleEngine {
    /// Rules to execute, keyed by domain code
    rules_by_domain: std::collections::BTreeMap<String, Vec<GeneratedRule>>,
}

impl RuleEngine {
    /// Create a new empty rule engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add rules for a domain.
    pub fn add_rules(&mut self, rules: Vec<GeneratedRule>) {
        for rule in rules {
            self.rules_by_domain
                .entry(rule.domain.clone())
                .or_default()
                .push(rule);
        }
    }

    /// Get rules for a specific domain.
    pub fn rules_for_domain(&self, domain_code: &str) -> &[GeneratedRule] {
        self.rules_by_domain
            .get(&domain_code.to_uppercase())
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Execute all rules for a domain against a DataFrame.
    pub fn execute(&self, domain_code: &str, df: &DataFrame) -> ConformanceReport {
        let rules = self.rules_for_domain(domain_code);
        let column_lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());
        let mut issues = Vec::new();

        for rule in rules {
            let issue = self.execute_rule(rule, df, &column_lookup);
            issues.extend(issue);
        }

        ConformanceReport {
            domain_code: domain_code.to_string(),
            issues,
        }
    }

    /// Execute a single rule against a DataFrame.
    fn execute_rule(
        &self,
        rule: &GeneratedRule,
        df: &DataFrame,
        column_lookup: &CaseInsensitiveLookup,
    ) -> Vec<ConformanceIssue> {
        match &rule.context {
            RuleContext::RequiredPresence => self.check_column_presence(rule, column_lookup),
            RuleContext::RequiredVariable => self.check_null_values(rule, df, column_lookup),
            RuleContext::ExpectedVariable => self.check_column_presence(rule, column_lookup),
            RuleContext::ControlledTerminology {
                valid_values,
                extensible,
                codelist_code,
                ..
            } => self.check_controlled_terminology(
                rule,
                df,
                column_lookup,
                valid_values,
                *extensible,
                codelist_code,
            ),
            RuleContext::DateTimeFormat => self.check_datetime_format(rule, df, column_lookup),
            RuleContext::SequenceUniqueness => {
                self.check_sequence_uniqueness(rule, df, column_lookup)
            }
            RuleContext::Other(_) => Vec::new(),
        }
    }

    /// Check if a column is present in the DataFrame.
    /// Used for RequiredPresence (SD0056) and ExpectedVariable (SD0057).
    fn check_column_presence(
        &self,
        rule: &GeneratedRule,
        column_lookup: &CaseInsensitiveLookup,
    ) -> Vec<ConformanceIssue> {
        if column_lookup.get(&rule.variable).is_some() {
            return Vec::new();
        }

        // Column is missing - emit issue using rule's metadata
        vec![self.create_issue(rule, 1, None)]
    }

    /// Check for null/missing values in a column.
    /// Used for RequiredVariable (SD0002).
    fn check_null_values(
        &self,
        rule: &GeneratedRule,
        df: &DataFrame,
        column_lookup: &CaseInsensitiveLookup,
    ) -> Vec<ConformanceIssue> {
        let column = match column_lookup.get(&rule.variable) {
            Some(col) => col,
            None => return Vec::new(), // Column missing is handled by RequiredPresence rule
        };

        let series = match df.column(column) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut missing_count = 0u64;
        for idx in 0..df.height() {
            let value = series.get(idx).unwrap_or(AnyValue::Null);
            if is_missing_value(&value) {
                missing_count += 1;
            }
        }

        if missing_count == 0 {
            return Vec::new();
        }

        vec![self.create_issue(rule, missing_count, None)]
    }

    /// Check controlled terminology values.
    /// Used for CT2001 (non-extensible) and CT2002 (extensible).
    fn check_controlled_terminology(
        &self,
        rule: &GeneratedRule,
        df: &DataFrame,
        column_lookup: &CaseInsensitiveLookup,
        valid_values: &[String],
        _extensible: bool,
        codelist_code: &str,
    ) -> Vec<ConformanceIssue> {
        let column = match column_lookup.get(&rule.variable) {
            Some(col) => col,
            None => return Vec::new(),
        };

        let series = match df.column(column) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        // Build uppercase set for case-insensitive comparison
        let valid_set: BTreeSet<String> = valid_values.iter().map(|v| v.to_uppercase()).collect();

        let mut invalid_values = BTreeSet::new();
        for idx in 0..df.height() {
            let raw = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !valid_set.contains(&trimmed.to_uppercase()) {
                invalid_values.insert(trimmed.to_string());
            }
        }

        if invalid_values.is_empty() {
            return Vec::new();
        }

        vec![self.create_issue(
            rule,
            invalid_values.len() as u64,
            Some(codelist_code.to_string()),
        )]
    }

    /// Check datetime format for *DTC variables.
    fn check_datetime_format(
        &self,
        rule: &GeneratedRule,
        df: &DataFrame,
        column_lookup: &CaseInsensitiveLookup,
    ) -> Vec<ConformanceIssue> {
        let column = match column_lookup.get(&rule.variable) {
            Some(col) => col,
            None => return Vec::new(),
        };

        let series = match df.column(column) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut invalid_count = 0u64;
        for idx in 0..df.height() {
            let raw = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            // validate_iso8601 returns None if valid, Some(error) if invalid
            if validate_iso8601(trimmed).is_some() {
                invalid_count += 1;
            }
        }

        if invalid_count == 0 {
            return Vec::new();
        }

        vec![self.create_issue(rule, invalid_count, None)]
    }

    /// Check sequence uniqueness for *SEQ variables.
    fn check_sequence_uniqueness(
        &self,
        rule: &GeneratedRule,
        df: &DataFrame,
        column_lookup: &CaseInsensitiveLookup,
    ) -> Vec<ConformanceIssue> {
        let seq_column = match column_lookup.get(&rule.variable) {
            Some(col) => col,
            None => return Vec::new(),
        };

        let usubjid_column = column_lookup.get("USUBJID");

        let seq_series = match df.column(seq_column) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let usubjid_series = usubjid_column.and_then(|col| df.column(col).ok());

        let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
        let mut duplicate_count = 0u64;

        for idx in 0..df.height() {
            let seq_val = any_to_string(seq_series.get(idx).unwrap_or(AnyValue::Null));
            if seq_val.trim().is_empty() {
                continue;
            }

            let usubjid = usubjid_series
                .as_ref()
                .map(|s| any_to_string(s.get(idx).unwrap_or(AnyValue::Null)))
                .unwrap_or_default();

            let key = (usubjid, seq_val);
            if !seen.insert(key) {
                duplicate_count += 1;
            }
        }

        if duplicate_count == 0 {
            return Vec::new();
        }

        vec![self.create_issue(rule, duplicate_count, None)]
    }

    /// Create a ConformanceIssue from a GeneratedRule.
    /// All issue creation uses this method to ensure we only use rule metadata.
    fn create_issue(
        &self,
        rule: &GeneratedRule,
        count: u64,
        codelist_code: Option<String>,
    ) -> ConformanceIssue {
        ConformanceIssue {
            code: rule.rule_id.clone(),
            message: format!("{} ({} occurrence(s))", rule.message, count),
            severity: convert_severity(rule.severity),
            variable: Some(rule.variable.clone()),
            count: Some(count),
            rule_id: Some(rule.rule_id.clone()),
            category: Some(rule.category.clone()),
            codelist_code,
            ct_source: None,
        }
    }
}

/// Convert RuleSeverity to IssueSeverity.
fn convert_severity(severity: RuleSeverity) -> IssueSeverity {
    match severity {
        RuleSeverity::Error => IssueSeverity::Error,
        RuleSeverity::Warning => IssueSeverity::Warning,
        RuleSeverity::Info => IssueSeverity::Warning, // Map Info to Warning
    }
}
