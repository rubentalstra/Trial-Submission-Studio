mod cross_domain;
mod engine;
pub mod rule_mapping;

pub use cross_domain::{
    CrossDomainValidationInput, CrossDomainValidationResult, validate_cross_domain,
};
pub use engine::RuleEngine;
pub use rule_mapping::RuleResolver;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use polars::prelude::{AnyValue, DataFrame};
use serde::Serialize;

use sdtm_core::{ProvenanceTracker, validate_column_order};
use sdtm_ingest::{any_to_string, is_missing_value};
use sdtm_model::{
    CaseInsensitiveLookup, ConformanceIssue, ConformanceReport, ControlledTerminology, CtRegistry,
    Domain, IssueSeverity, OutputFormat, ResolvedCt, Variable, VariableType,
};
use sdtm_standards::assumptions::RuleGenerator;
use sdtm_standards::loaders::P21Rule;

#[derive(Debug, Clone, Default)]
pub struct ValidationContext<'a> {
    pub ct_registry: Option<&'a CtRegistry>,
    pub p21_rules: Option<&'a [P21Rule]>,
    pub ct_catalogs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct GatingDecision {
    pub block_strict_outputs: bool,
    pub blocking_domains: Vec<String>,
}

pub fn strict_outputs_requested(output_formats: &[OutputFormat]) -> bool {
    output_formats
        .iter()
        .any(|format| matches!(format, OutputFormat::Xpt))
}

pub fn gate_strict_outputs(
    output_formats: &[OutputFormat],
    fail_on_conformance_errors: bool,
    reports: &[ConformanceReport],
) -> GatingDecision {
    if !fail_on_conformance_errors || !strict_outputs_requested(output_formats) {
        return GatingDecision::default();
    }
    let mut blocking = BTreeSet::new();
    for report in reports {
        if report.has_errors() {
            blocking.insert(report.domain_code.clone());
        }
    }
    GatingDecision {
        block_strict_outputs: !blocking.is_empty(),
        blocking_domains: blocking.into_iter().collect(),
    }
}

impl<'a> ValidationContext<'a> {
    pub fn new() -> Self {
        Self {
            ct_registry: None,
            p21_rules: None,
            ct_catalogs: None,
        }
    }

    pub fn with_ct_registry(mut self, ct_registry: &'a CtRegistry) -> Self {
        self.ct_registry = Some(ct_registry);
        self
    }

    pub fn with_p21_rules(mut self, p21_rules: &'a [P21Rule]) -> Self {
        self.p21_rules = Some(p21_rules);
        self
    }

    pub fn with_ct_catalogs(mut self, catalogs: Vec<String>) -> Self {
        self.ct_catalogs = Some(catalogs);
        self
    }

    /// Build a RuleEngine from dynamically generated rules.
    ///
    /// This creates rules from metadata sources (Variables.csv, CT files)
    /// rather than manually coding them.
    pub fn build_rule_engine(&self, domains: &[Domain]) -> RuleEngine {
        let ct_registry = self.ct_registry.cloned().unwrap_or_default();
        let p21_rules = self
            .p21_rules
            .map(|rules| rules.to_vec())
            .unwrap_or_default();

        let generator = RuleGenerator::new().with_p21_rules(p21_rules);

        let mut engine = RuleEngine::new();
        for domain in domains {
            let rules = generator.generate_rules_for_domain(domain, &ct_registry);
            engine.add_rules(rules);
        }
        engine
    }
}

/// Validate a domain using dynamically generated rules.
///
/// This is the preferred validation approach per AGENTS.md:
/// rules are generated from metadata, not manually coded.
pub fn validate_domain_with_rules(
    domain: &Domain,
    df: &DataFrame,
    ctx: &ValidationContext,
) -> ConformanceReport {
    let engine = ctx.build_rule_engine(std::slice::from_ref(domain));
    engine.execute(&domain.code, df)
}

/// Validate multiple domains using dynamically generated rules.
pub fn validate_domains_with_rules(
    domains: &[Domain],
    frames: &[(&str, &DataFrame)],
    ctx: &ValidationContext,
) -> Vec<ConformanceReport> {
    let engine = ctx.build_rule_engine(domains);

    let mut reports = Vec::new();
    for (domain_code, df) in frames {
        reports.push(engine.execute(domain_code, df));
    }
    reports
}

/// Known derived variable patterns (typically have Origin="Derived" in Define-XML).
/// These variables should have provenance tracking if they contain values.
const DERIVED_VARIABLE_SUFFIXES: &[&str] = &["SEQ", "DY", "STDY", "ENDY"];

/// Variables that are always derived (not collected).
const ALWAYS_DERIVED_VARIABLES: &[&str] = &["USUBJID", "STUDYID", "DOMAIN"];

/// Check if a variable name indicates a derived variable.
fn is_derived_variable(name: &str) -> bool {
    let upper = name.to_uppercase();
    // Check known derived suffixes
    for suffix in DERIVED_VARIABLE_SUFFIXES {
        if upper.ends_with(suffix) {
            return true;
        }
    }
    // Check always-derived variables
    ALWAYS_DERIVED_VARIABLES
        .iter()
        .any(|v| upper.eq_ignore_ascii_case(v))
}

/// Validate that derived variables have documented provenance.
///
/// This function checks variables that should be derived (based on naming patterns
/// like --SEQ, --DY, --STDY, --ENDY) and flags any that have values but no
/// provenance tracking. This implements the no-imputation check: all derived
/// values should have a documented source or derivation rule.
///
/// # Arguments
///
/// * `domain` - The domain metadata
/// * `df` - The domain data
/// * `provenance` - Optional provenance tracker from processing
///
/// # Returns
///
/// Conformance issues for any derived variables without documented provenance.
pub fn validate_provenance(
    domain: &Domain,
    df: &DataFrame,
    provenance: Option<&ProvenanceTracker>,
) -> Vec<ConformanceIssue> {
    let mut issues = Vec::new();
    let column_lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());

    for variable in &domain.variables {
        // Only check variables that should be derived
        if !is_derived_variable(&variable.name) {
            continue;
        }

        // Check if column exists and has values
        let column_name = match column_lookup.get(&variable.name) {
            Some(name) => name,
            None => continue, // Column not present
        };

        let series = match df.column(column_name) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Check if column has any non-null values
        let has_values = (0..series.len()).any(|idx| {
            let value = series.get(idx).unwrap_or(AnyValue::Null);
            let value_str = any_to_string(value);
            !value_str.trim().is_empty()
        });

        if !has_values {
            continue; // No values to validate
        }

        // Check if provenance exists for this variable
        let has_provenance = provenance
            .map(|p| p.has_provenance(&domain.code, &variable.name))
            .unwrap_or(false);

        if !has_provenance {
            issues.push(ConformanceIssue {
                severity: IssueSeverity::Warning,
                code: format!(
                    "{}.{}: Derived variable has values but no documented provenance",
                    domain.code, variable.name
                ),
                variable: Some(variable.name.clone()),
                message: format!(
                    "Derived variable {} contains values but derivation method is not \
                     tracked. Enable provenance tracking to document how this variable was \
                     populated.",
                    variable.name
                ),
                rule_id: Some("TRANS0001".to_string()),
                category: Some("Provenance".to_string()),
                count: None,
                codelist_code: None,
                ct_source: None,
            });
        }
    }

    issues
}

/// Validate provenance for multiple domains.
pub fn validate_domains_provenance(
    domains: &[Domain],
    frames: &[(&str, &DataFrame)],
    provenance: Option<&ProvenanceTracker>,
) -> Vec<ConformanceReport> {
    let mut reports = Vec::new();

    for (domain_code, df) in frames {
        // Find the domain metadata
        let domain = domains
            .iter()
            .find(|d| d.code.eq_ignore_ascii_case(domain_code));

        if let Some(domain) = domain {
            let issues = validate_provenance(domain, df, provenance);
            if !issues.is_empty() {
                reports.push(ConformanceReport {
                    domain_code: domain_code.to_string(),
                    issues,
                });
            }
        }
    }

    reports
}

#[derive(Debug, Serialize)]
pub struct ConformanceReportPayload {
    pub schema: &'static str,
    pub schema_version: u32,
    pub generated_at: String,
    pub study_id: String,
    pub reports: Vec<ConformanceReportSummary>,
}

#[derive(Debug, Serialize)]
pub struct ConformanceReportSummary {
    pub domain: String,
    pub error_count: usize,
    pub warning_count: usize,
    pub issues: Vec<ConformanceIssueJson>,
}

#[derive(Debug, Serialize)]
pub struct ConformanceIssueJson {
    pub severity: IssueSeverity,
    pub code: String,
    pub domain: String,
    pub variable: Option<String>,
    pub message: String,
    pub rule_id: Option<String>,
    pub category: Option<String>,
    pub count: Option<u64>,
    pub codelist_code: Option<String>,
    pub ct_source: Option<String>,
}

const REPORT_SCHEMA: &str = "cdisc-transpiler.conformance-report";
const REPORT_SCHEMA_VERSION: u32 = 1;

pub fn validate_domain(
    domain: &Domain,
    df: &DataFrame,
    ctx: &ValidationContext,
) -> ConformanceReport {
    let column_lookup = build_column_lookup(df);
    let p21_lookup = build_p21_lookup(ctx.p21_rules);
    let mut issues = Vec::new();
    for variable in &domain.variables {
        let column = column_lookup.get(&variable.name);
        if column.is_none() {
            issues.extend(missing_column_issues(domain, variable, &p21_lookup));
            continue;
        }
        let column = column.expect("column lookup");
        if let Some(issue) = missing_value_issue(domain, variable, df, column, &p21_lookup) {
            issues.push(issue);
        }
        if let Some(issue) = type_issue(domain, variable, df, column, &p21_lookup) {
            issues.push(issue);
        }
        if let Some(issue) = length_issue(domain, variable, df, column, &p21_lookup) {
            issues.push(issue);
        }
        if let Some(issue) = test_code_issue(domain, variable, df, column, &p21_lookup) {
            issues.push(issue);
        }
        if let Some(ct_registry) = ctx.ct_registry
            && let Some(resolved) = resolve_ct(ct_registry, variable, ctx.ct_catalogs.as_deref())
            && let Some(issue) = ct_issue(variable, df, column, &resolved, &p21_lookup)
        {
            issues.push(issue);
        }
    }

    // Validate column order by SDTM role (Identifiers, Topic, Qualifiers, Rule, Timing)
    // Per SDTMIG v3.4 Chapter 2.1 and P21 Rule SD1079
    let column_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let order_result = validate_column_order(&column_names, domain);
    if !order_result.is_valid {
        // Get rule metadata from P21 Rules.csv
        let p21_rule = p21_lookup.get("SD1079");
        let category = p21_rule
            .and_then(|r| r.category.clone())
            .unwrap_or_else(|| "Metadata".to_string());
        for violation in &order_result.violations {
            issues.push(ConformanceIssue {
                code: domain.code.clone(),
                message: violation.clone(),
                severity: IssueSeverity::Warning,
                variable: None,
                count: Some(1),
                rule_id: Some("SD1079".to_string()),
                category: Some(category.clone()),
                codelist_code: None,
                ct_source: None,
            });
        }
    }

    ConformanceReport {
        domain_code: domain.code.clone(),
        issues,
    }
}

pub fn validate_domains(
    domains: &[Domain],
    frames: &[(&str, &DataFrame)],
    ctx: &ValidationContext,
) -> Vec<ConformanceReport> {
    let mut domain_map: BTreeMap<String, &Domain> = BTreeMap::new();
    for domain in domains {
        domain_map.insert(domain.code.to_uppercase(), domain);
    }
    let mut frame_map: BTreeMap<String, &DataFrame> = BTreeMap::new();
    for (domain_code, df) in frames {
        frame_map.insert(domain_code.to_uppercase(), *df);
    }
    let mut report_map: BTreeMap<String, ConformanceReport> = BTreeMap::new();
    for (domain_code, df) in frames {
        let code = domain_code.to_uppercase();
        if let Some(domain) = domain_map.get(&code) {
            report_map.insert(code.clone(), validate_domain(domain, df, ctx));
        }
    }
    if let Some(p21_rules) = ctx.p21_rules {
        apply_missing_dataset_issues(&domain_map, &frame_map, p21_rules, &mut report_map);
    }
    report_map.into_values().collect()
}

pub fn has_conformance_errors(reports: &[ConformanceReport]) -> bool {
    reports.iter().any(|report| report.has_errors())
}

pub fn write_conformance_report_json(
    output_dir: &Path,
    study_id: &str,
    reports: &[ConformanceReport],
) -> Result<PathBuf> {
    std::fs::create_dir_all(output_dir)?;
    let output_path = output_dir.join("conformance_report.json");
    let payload = ConformanceReportPayload {
        schema: REPORT_SCHEMA,
        schema_version: REPORT_SCHEMA_VERSION,
        generated_at: Utc::now().to_rfc3339(),
        study_id: study_id.to_string(),
        reports: reports
            .iter()
            .map(|report| ConformanceReportSummary {
                domain: report.domain_code.clone(),
                error_count: report.error_count(),
                warning_count: report.warning_count(),
                issues: report
                    .issues
                    .iter()
                    .map(|issue| ConformanceIssueJson {
                        severity: issue.severity,
                        code: issue.code.clone(),
                        domain: report.domain_code.clone(),
                        variable: issue.variable.clone(),
                        message: issue.message.clone(),
                        rule_id: issue.rule_id.clone(),
                        category: issue.category.clone(),
                        count: issue.count,
                        codelist_code: issue.codelist_code.clone(),
                        ct_source: issue.ct_source.clone(),
                    })
                    .collect(),
            })
            .collect(),
    };
    let json = serde_json::to_string_pretty(&payload)?;
    std::fs::write(&output_path, format!("{json}\n"))?;
    Ok(output_path)
}

fn build_column_lookup(df: &DataFrame) -> CaseInsensitiveLookup {
    CaseInsensitiveLookup::new(df.get_column_names_owned())
}

fn missing_column_issues(
    _domain: &Domain,
    variable: &Variable,
    p21_lookup: &BTreeMap<String, &P21Rule>,
) -> Vec<ConformanceIssue> {
    if is_required(variable.core.as_deref()) {
        let rule = p21_lookup.get("SD0056").copied();
        let base = rule_base_message(rule, "SDTM Required variable not found");
        let message = format!("{base}: {}", variable.name);
        return vec![issue_from_rule(
            "SD0056",
            p21_lookup,
            IssueSeverity::Error,
            message,
            Some(variable.name.clone()),
            None,
            None,
        )];
    }
    if is_expected(variable.core.as_deref()) {
        let rule = p21_lookup.get("SD0057").copied();
        let base = rule_base_message(rule, "SDTM Expected variable not found");
        let message = format!("{base}: {}", variable.name);
        return vec![issue_from_rule(
            "SD0057",
            p21_lookup,
            IssueSeverity::Warning,
            message,
            Some(variable.name.clone()),
            None,
            None,
        )];
    }
    Vec::new()
}

fn missing_value_issue(
    _domain: &Domain,
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    p21_lookup: &BTreeMap<String, &P21Rule>,
) -> Option<ConformanceIssue> {
    if !is_required(variable.core.as_deref()) {
        return None;
    }
    let series = df.column(column).ok()?;
    let mut missing = 0u64;
    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        if is_missing_value(&value) {
            missing += 1;
        }
    }
    if missing == 0 {
        return None;
    }
    let rule = p21_lookup.get("SD0002").copied();
    let base = rule_base_message(rule, "Null value in variable marked as Required");
    let message = format!(
        "{base}: {} has {} missing/blank value(s)",
        variable.name, missing
    );
    Some(issue_from_rule(
        "SD0002",
        p21_lookup,
        IssueSeverity::Error,
        message,
        Some(variable.name.clone()),
        Some(missing),
        None,
    ))
}

fn type_issue(
    _domain: &Domain,
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    p21_lookup: &BTreeMap<String, &P21Rule>,
) -> Option<ConformanceIssue> {
    if variable.data_type != VariableType::Num {
        return None;
    }
    let series = df.column(column).ok()?;
    let mut invalid = 0u64;
    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        if is_missing_value(&value) {
            continue;
        }
        if is_numeric_value(&value) {
            continue;
        }
        let text = any_to_string(value);
        if text.trim().is_empty() {
            continue;
        }
        if text.trim().parse::<f64>().is_err() {
            invalid += 1;
        }
    }
    if invalid == 0 {
        return None;
    }
    let rule = p21_lookup.get("SD1230").copied();
    let base = rule_base_message(rule, "Variable datatype is not the expected SDTM datatype");
    let message = format!(
        "{base}: {} has {} non-numeric value(s)",
        variable.name, invalid
    );
    Some(issue_from_rule(
        "SD1230",
        p21_lookup,
        IssueSeverity::Error,
        message,
        Some(variable.name.clone()),
        Some(invalid),
        None,
    ))
}

fn length_issue(
    _domain: &Domain,
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    p21_lookup: &BTreeMap<String, &P21Rule>,
) -> Option<ConformanceIssue> {
    let limit = variable.length?;
    if variable.data_type != VariableType::Char {
        return None;
    }
    let series = df.column(column).ok()?;
    let mut over = 0u64;
    for idx in 0..df.height() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        if value.trim().is_empty() {
            continue;
        }
        if value.chars().count() > limit as usize {
            over += 1;
        }
    }
    if over == 0 {
        return None;
    }
    let rule = p21_lookup.get("SD1231").copied();
    let base = rule_base_message(rule, "Variable value is longer than defined max length");
    let message = format!(
        "{base}: {} exceeds length {} in {} value(s)",
        variable.name, limit, over
    );
    Some(issue_from_rule(
        "SD1231",
        p21_lookup,
        IssueSeverity::Error,
        message,
        Some(variable.name.clone()),
        Some(over),
        None,
    ))
}

fn test_code_issue(
    _domain: &Domain,
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    p21_lookup: &BTreeMap<String, &P21Rule>,
) -> Option<ConformanceIssue> {
    if !is_testcd_variable(&variable.name) {
        return None;
    }
    let series = df.column(column).ok()?;
    let mut invalid = 0u64;
    let mut examples = BTreeSet::new();
    for idx in 0..df.height() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if is_valid_test_code(trimmed) {
            continue;
        }
        invalid += 1;
        if examples.len() < 5 {
            examples.insert(trimmed.to_string());
        }
    }
    if invalid == 0 {
        return None;
    }
    let mut example_list: Vec<String> = examples.into_iter().collect();
    example_list.sort();
    let examples = example_list.join(", ");
    let base = "Invalid TESTCD/QNAM value (must be <=8 chars, start with a letter or underscore, and contain only letters, numbers, or underscores)";
    let mut message = format!("{base}: {} has {invalid} invalid value(s)", variable.name);
    if !examples.is_empty() {
        message.push_str(&format!(" values: {}", examples));
    }
    Some(issue_from_rule(
        "SD1022",
        p21_lookup,
        IssueSeverity::Error,
        message,
        Some(variable.name.clone()),
        Some(invalid),
        None,
    ))
}

fn is_testcd_variable(name: &str) -> bool {
    let upper = name.to_uppercase();
    upper == "QNAM" || upper.ends_with("TESTCD")
}

fn is_valid_test_code(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 8 {
        return false;
    }
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first.is_ascii_digit() {
        return false;
    }
    if !first.is_ascii_alphanumeric() && first != '_' {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn ct_issue(
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    resolved: &ResolvedCt,
    p21_lookup: &BTreeMap<String, &P21Rule>,
) -> Option<ConformanceIssue> {
    let ct = resolved.ct;
    let invalid = collect_invalid_ct_values(df, column, ct);
    if invalid.is_empty() {
        return None;
    }
    let (rule_id, default_severity) = if ct.extensible {
        ("CT2002", IssueSeverity::Warning)
    } else {
        ("CT2001", IssueSeverity::Error)
    };
    let mut examples = invalid.iter().take(5).cloned().collect::<Vec<_>>();
    examples.sort();
    let examples = examples.join(", ");
    let rule = p21_lookup.get(rule_id).copied();
    let base = rule_base_message(rule, "Variable value not found in codelist");
    let mut message = format!(
        "{}. {} contains {} value(s) not found in {} for {} ({}).",
        base,
        variable.name,
        invalid.len(),
        resolved.source,
        ct.codelist_name,
        ct.codelist_code
    );
    if !examples.is_empty() {
        message.push_str(&format!(" values: {}", examples));
    }
    let rule = p21_lookup.get(rule_id).copied();
    Some(ConformanceIssue {
        code: ct.codelist_code.clone(),
        message,
        severity: rule_severity(rule, default_severity),
        variable: Some(variable.name.clone()),
        count: Some(invalid.len() as u64),
        rule_id: Some(rule_id.to_string()),
        category: rule.and_then(|rule| rule.category.clone()),
        codelist_code: Some(ct.codelist_code.clone()),
        ct_source: Some(resolved.source.to_string()),
    })
}

fn build_p21_lookup(rules: Option<&[P21Rule]>) -> BTreeMap<String, &P21Rule> {
    let mut lookup = BTreeMap::new();
    if let Some(rules) = rules {
        for rule in rules {
            lookup.insert(rule.rule_id.to_uppercase(), rule);
        }
    }
    lookup
}

fn parse_severity(rule: &P21Rule) -> Option<IssueSeverity> {
    let raw = rule.severity.as_ref()?.trim().to_lowercase();
    match raw.as_str() {
        "reject" => Some(IssueSeverity::Reject),
        "error" => Some(IssueSeverity::Error),
        "warning" => Some(IssueSeverity::Warning),
        _ => None,
    }
}

fn rule_severity(rule: Option<&P21Rule>, fallback: IssueSeverity) -> IssueSeverity {
    rule.and_then(parse_severity).unwrap_or(fallback)
}

fn rule_base_message(rule: Option<&P21Rule>, fallback: &str) -> String {
    if let Some(rule) = rule {
        if !rule.message.trim().is_empty() {
            return rule.message.clone();
        }
        if !rule.description.trim().is_empty() {
            return rule.description.clone();
        }
    }
    fallback.to_string()
}

fn issue_from_rule(
    rule_id: &str,
    p21_lookup: &BTreeMap<String, &P21Rule>,
    fallback_severity: IssueSeverity,
    message: String,
    variable: Option<String>,
    count: Option<u64>,
    codelist_code: Option<String>,
) -> ConformanceIssue {
    let rule = p21_lookup.get(&rule_id.to_uppercase()).copied();
    let severity = rule_severity(rule, fallback_severity);
    let category = rule.and_then(|rule| rule.category.clone());
    let resolved_id = rule
        .map(|rule| rule.rule_id.clone())
        .unwrap_or_else(|| rule_id.to_string());
    ConformanceIssue {
        code: resolved_id.clone(),
        message,
        severity,
        variable,
        count,
        rule_id: Some(resolved_id),
        category,
        codelist_code,
        ct_source: None,
    }
}

fn apply_missing_dataset_issues(
    domain_map: &BTreeMap<String, &Domain>,
    frame_map: &BTreeMap<String, &DataFrame>,
    p21_rules: &[P21Rule],
    report_map: &mut BTreeMap<String, ConformanceReport>,
) {
    for rule in p21_rules {
        let Some(code) = missing_dataset_code(&rule.message) else {
            continue;
        };
        if !domain_map.contains_key(&code) {
            continue;
        }
        if frame_map.contains_key(&code) {
            continue;
        }
        let severity = match parse_severity(rule) {
            Some(IssueSeverity::Warning) => IssueSeverity::Warning,
            _ => IssueSeverity::Error,
        };
        let issue = ConformanceIssue {
            code: rule.rule_id.clone(),
            message: rule_message(rule, None),
            severity,
            variable: None,
            count: Some(1),
            rule_id: Some(rule.rule_id.clone()),
            category: rule.category.clone(),
            codelist_code: None,
            ct_source: None,
        };
        add_report_issue(report_map, &code, issue);
    }
}

fn missing_dataset_code(message: &str) -> Option<String> {
    let prefix = "Missing ";
    let suffix = " dataset";
    let msg = message.trim();
    if !msg.starts_with(prefix) || !msg.ends_with(suffix) {
        return None;
    }
    let raw = msg[prefix.len()..msg.len() - suffix.len()].trim();
    if raw.is_empty() {
        return None;
    }
    Some(raw.to_uppercase())
}

fn add_report_issue(
    report_map: &mut BTreeMap<String, ConformanceReport>,
    domain_code: &str,
    issue: ConformanceIssue,
) {
    report_map
        .entry(domain_code.to_string())
        .or_insert_with(|| ConformanceReport {
            domain_code: domain_code.to_string(),
            issues: Vec::new(),
        })
        .issues
        .push(issue);
}

fn rule_message(rule: &P21Rule, count: Option<usize>) -> String {
    let base = if rule.message.trim().is_empty() {
        rule.description.clone()
    } else {
        rule.message.clone()
    };
    match count {
        Some(value) => format!("{base} ({value} value(s))"),
        None => base,
    }
}

fn collect_invalid_ct_values(
    df: &DataFrame,
    column: &str,
    ct: &ControlledTerminology,
) -> BTreeSet<String> {
    let mut invalid = BTreeSet::new();
    let series = match df.column(column) {
        Ok(series) => series,
        Err(_) => return invalid,
    };
    let submission_values: BTreeSet<String> = ct
        .submission_values
        .iter()
        .map(|value| value.to_uppercase())
        .collect();

    for idx in 0..df.height() {
        let raw = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let normalized = normalize_ct_value(ct, trimmed);
        if normalized.is_empty() {
            continue;
        }
        let key = normalized.to_uppercase();
        if submission_values.contains(&key) {
            continue;
        }
        if ct.extensible {
            invalid.insert(trimmed.to_string());
            continue;
        }
        invalid.insert(trimmed.to_string());
    }
    invalid
}

fn normalize_ct_value(ct: &ControlledTerminology, raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lookup = trimmed.to_uppercase();
    ct.synonyms
        .get(&lookup)
        .cloned()
        .unwrap_or_else(|| trimmed.to_string())
}

fn resolve_ct<'a>(
    registry: &'a CtRegistry,
    variable: &Variable,
    preferred: Option<&[String]>,
) -> Option<ResolvedCt<'a>> {
    registry.resolve_for_variable(variable, preferred)
}

fn is_required(core: Option<&str>) -> bool {
    matches!(
        core.map(|value| value.trim().to_lowercase()).as_deref(),
        Some("req")
    )
}

fn is_expected(core: Option<&str>) -> bool {
    matches!(
        core.map(|value| value.trim().to_lowercase()).as_deref(),
        Some("exp")
    )
}

fn is_permissible(core: Option<&str>) -> bool {
    matches!(
        core.map(|value| value.trim().to_lowercase()).as_deref(),
        Some("perm")
    )
}

fn is_numeric_value(value: &AnyValue) -> bool {
    matches!(
        value,
        AnyValue::Float32(_)
            | AnyValue::Float64(_)
            | AnyValue::Int8(_)
            | AnyValue::Int16(_)
            | AnyValue::Int32(_)
            | AnyValue::Int64(_)
            | AnyValue::UInt8(_)
            | AnyValue::UInt16(_)
            | AnyValue::UInt32(_)
            | AnyValue::UInt64(_)
    )
}
