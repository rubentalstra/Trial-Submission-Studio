//! Rule Engine for executing dynamically generated rules.
//!
//! This engine takes rules generated from metadata (via `RuleGenerator`) and
//! executes them against domain DataFrames, producing structured conformance issues.
//!
//! Per SDTMIG v3.4 and AGENTS.md: rules are never manually coded—they are
//! derived dynamically from metadata sources.

use std::collections::BTreeSet;

use polars::prelude::{AnyValue, DataFrame};

use sdtm_core::{DatePairOrder, can_compute_study_day, validate_date_pair, validate_iso8601};
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
            RuleContext::DatePairOrder {
                start_variable,
                end_variable,
            } => self.check_date_pair_order(rule, df, column_lookup, start_variable, end_variable),
            RuleContext::StudyDayCompleteness {
                dtc_variable,
                dy_variable,
            } => self.check_study_day_completeness(
                rule,
                df,
                column_lookup,
                dtc_variable,
                dy_variable,
            ),
            RuleContext::FindingsTimingVariable {
                variable_suffix: _,
                is_allowed,
            } => self.check_findings_timing_variable(rule, column_lookup, *is_allowed),
            RuleContext::RelativeTimingVariable {
                variable: _,
                allowed_values,
                anchor_variable,
            } => self.check_relative_timing_variable(rule, df, column_lookup, allowed_values, anchor_variable),
            RuleContext::DurationUsage {
                dur_variable,
                stdtc_variable,
                endtc_variable,
            } => self.check_duration_usage(rule, df, column_lookup, dur_variable, stdtc_variable, endtc_variable),
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

    /// Check study day completeness per SDTMIG v3.4 Section 4.4.4.
    ///
    /// Per SDTMIG 4.4.4: Study day (--DY) requires complete date components
    /// (year, month, day) for both the observation date and reference date.
    /// This validation flags records where --DY is missing because the
    /// associated date (--DTC) is partial/incomplete.
    fn check_study_day_completeness(
        &self,
        rule: &GeneratedRule,
        df: &DataFrame,
        column_lookup: &CaseInsensitiveLookup,
        dtc_variable: &str,
        dy_variable: &str,
    ) -> Vec<ConformanceIssue> {
        let dtc_col = match column_lookup.get(dtc_variable) {
            Some(col) => col,
            None => return Vec::new(), // DTC column not present
        };

        let dy_col = match column_lookup.get(dy_variable) {
            Some(col) => col,
            None => return Vec::new(), // DY column not present
        };

        let dtc_series = match df.column(dtc_col) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let dy_series = match df.column(dy_col) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut partial_date_count = 0u64;

        for idx in 0..df.height() {
            let dtc_val = any_to_string(dtc_series.get(idx).unwrap_or(AnyValue::Null));
            let dy_val = any_to_string(dy_series.get(idx).unwrap_or(AnyValue::Null));

            let dtc_trimmed = dtc_val.trim();
            let dy_trimmed = dy_val.trim();

            // Flag when DTC has a value but DY is empty and date is not complete
            // per SDTMIG 4.4.4: Study day requires complete dates
            if !dtc_trimmed.is_empty()
                && dy_trimmed.is_empty()
                && !can_compute_study_day(dtc_trimmed)
            {
                // Partial/incomplete date cannot be used for study day calculation
                partial_date_count += 1;
            }
        }

        if partial_date_count == 0 {
            return Vec::new();
        }

        vec![self.create_issue(rule, partial_date_count, None)]
    }

    /// Check date pair ordering per SDTMIG v3.4 Section 4.4.
    /// End date must not precede start date.
    fn check_date_pair_order(
        &self,
        rule: &GeneratedRule,
        df: &DataFrame,
        column_lookup: &CaseInsensitiveLookup,
        start_variable: &str,
        end_variable: &str,
    ) -> Vec<ConformanceIssue> {
        let start_col = match column_lookup.get(start_variable) {
            Some(col) => col,
            None => return Vec::new(), // Start column not present
        };

        let end_col = match column_lookup.get(end_variable) {
            Some(col) => col,
            None => return Vec::new(), // End column not present
        };

        let start_series = match df.column(start_col) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let end_series = match df.column(end_col) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut invalid_count = 0u64;

        for idx in 0..df.height() {
            let start_val = any_to_string(start_series.get(idx).unwrap_or(AnyValue::Null));
            let end_val = any_to_string(end_series.get(idx).unwrap_or(AnyValue::Null));

            // Only check when both values are present and non-empty
            let start_trimmed = start_val.trim();
            let end_trimmed = end_val.trim();

            if start_trimmed.is_empty() || end_trimmed.is_empty() {
                continue;
            }

            // Per SDTMIG v3.4 Section 4.4, end date must not precede start date
            if validate_date_pair(start_trimmed, end_trimmed) == DatePairOrder::EndBeforeStart {
                invalid_count += 1;
            }
        }

        if invalid_count == 0 {
            return Vec::new();
        }

        vec![self.create_issue(rule, invalid_count, None)]
    }

    /// Check Findings class timing variable rules per SDTMIG v3.4 Section 4.4.8.
    ///
    /// Per SDTMIG 4.4.8:
    /// - --DTC is used for collection date/time in Findings class
    /// - --STDTC should NOT be used in Findings class domains
    fn check_findings_timing_variable(
        &self,
        rule: &GeneratedRule,
        column_lookup: &CaseInsensitiveLookup,
        is_allowed: bool,
    ) -> Vec<ConformanceIssue> {
        // Check if the specific variable from the rule is present
        let has_variable = column_lookup.contains(&rule.variable);

        if !is_allowed && has_variable {
            // Variable is present but not allowed in Findings class
            vec![self.create_issue(rule, 1, None)]
        } else {
            Vec::new()
        }
    }

    /// Check relative timing variable rules per SDTMIG v3.4 Section 4.4.7.
    ///
    /// This checks:
    /// 1. If the anchor variable is missing (structure-level check)
    /// 2. If both relative timing and actual dates are populated (data-level check)
    fn check_relative_timing_variable(
        &self,
        rule: &GeneratedRule,
        df: &DataFrame,
        column_lookup: &CaseInsensitiveLookup,
        allowed_values: &[String],
        anchor_variable: &str,
    ) -> Vec<ConformanceIssue> {
        // Structure check: anchor variable should be present
        // This is already captured during rule generation - we only generate
        // the rule if anchor is missing

        let rel_col = match column_lookup.get(&rule.variable) {
            Some(col) => col,
            None => return Vec::new(), // Variable not present, no issue
        };

        let rel_series = match df.column(rel_col) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        // Check if there are populated values in the relative timing variable
        let mut populated_count = 0u64;
        let valid_upper: BTreeSet<String> = allowed_values.iter().map(|v| v.to_uppercase()).collect();

        for idx in 0..df.height() {
            let val = any_to_string(rel_series.get(idx).unwrap_or(AnyValue::Null));
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                populated_count += 1;

                // Optional: check if value is in allowed values
                // This is soft because CT validation will catch invalid values
                if !valid_upper.contains(&trimmed.to_uppercase()) {
                    // Value not in allowed list - CT validation handles this
                }
            }
        }

        // Check if anchor variable exists when relative timing is populated
        if populated_count > 0 && column_lookup.get(anchor_variable).is_none() {
            // Anchor is missing but relative timing is populated
            return vec![self.create_issue(rule, populated_count, None)];
        }

        // Check if both relative timing and actual dates are populated
        // This is a warning about redundant data
        let actual_date_var = if rule.variable.ends_with("STRF") {
            rule.variable.replace("STRF", "STDTC")
        } else if rule.variable.ends_with("ENRF") {
            rule.variable.replace("ENRF", "ENDTC")
        } else {
            return Vec::new();
        };

        if let Some(date_col) = column_lookup.get(&actual_date_var) {
            if let Ok(date_series) = df.column(date_col) {
                let mut both_populated = 0u64;
                for idx in 0..df.height() {
                    let rel_val = any_to_string(rel_series.get(idx).unwrap_or(AnyValue::Null));
                    let date_val = any_to_string(date_series.get(idx).unwrap_or(AnyValue::Null));
                    if !rel_val.trim().is_empty() && !date_val.trim().is_empty() {
                        both_populated += 1;
                    }
                }
                if both_populated > 0 {
                    return vec![self.create_issue(rule, both_populated, None)];
                }
            }
        }

        Vec::new()
    }

    /// Check duration usage rules per SDTMIG v3.4 Section 4.4.3.
    ///
    /// Per SDTMIG 4.4.3, --DUR should not be populated when both --STDTC and
    /// --ENDTC are collected and populated for a record.
    fn check_duration_usage(
        &self,
        rule: &GeneratedRule,
        df: &DataFrame,
        column_lookup: &CaseInsensitiveLookup,
        dur_variable: &str,
        stdtc_variable: &str,
        endtc_variable: &str,
    ) -> Vec<ConformanceIssue> {
        let dur_col = match column_lookup.get(dur_variable) {
            Some(col) => col,
            None => return Vec::new(),
        };
        let stdtc_col = match column_lookup.get(stdtc_variable) {
            Some(col) => col,
            None => return Vec::new(),
        };
        let endtc_col = match column_lookup.get(endtc_variable) {
            Some(col) => col,
            None => return Vec::new(),
        };

        let dur_series = match df.column(dur_col) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let stdtc_series = match df.column(stdtc_col) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let endtc_series = match df.column(endtc_col) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        // Count records where all three (DUR, STDTC, ENDTC) are populated
        let mut redundant_count = 0u64;

        for idx in 0..df.height() {
            let dur_val = any_to_string(dur_series.get(idx).unwrap_or(AnyValue::Null));
            let stdtc_val = any_to_string(stdtc_series.get(idx).unwrap_or(AnyValue::Null));
            let endtc_val = any_to_string(endtc_series.get(idx).unwrap_or(AnyValue::Null));

            // Per SDTMIG 4.4.3: DUR should not be populated when both STDTC and ENDTC are
            if !dur_val.trim().is_empty()
                && !stdtc_val.trim().is_empty()
                && !endtc_val.trim().is_empty()
            {
                redundant_count += 1;
            }
        }

        if redundant_count == 0 {
            return Vec::new();
        }

        vec![self.create_issue(rule, redundant_count, None)]
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
