use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use polars::prelude::{AnyValue, DataFrame};
use serde::Serialize;

use sdtm_model::{
    ConformanceIssue, ConformanceReport, ControlledTerminology, CtRegistry, Domain, IssueSeverity,
    OutputFormat, Variable, VariableType,
};
use sdtm_standards::loaders::P21Rule;

#[derive(Debug, Clone, Default)]
pub struct ValidationContext<'a> {
    pub ct_registry: Option<&'a CtRegistry>,
    pub p21_rules: Option<&'a [P21Rule]>,
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
}

const REPORT_SCHEMA: &str = "cdisc-transpiler.conformance-report";
const REPORT_SCHEMA_VERSION: u32 = 1;
const RULE_REQUIRED_VAR_MISSING: &str = "SD0056";
const RULE_EXPECTED_VAR_MISSING: &str = "SD0057";
const RULE_REQUIRED_VALUE_MISSING: &str = "SD0002";
const RULE_DATATYPE_MISMATCH: &str = "SD1230";
const RULE_LENGTH_EXCEEDED: &str = "SD1231";
const RULE_CT_NON_EXTENSIBLE: &str = "CT2001";
const RULE_CT_EXTENSIBLE: &str = "CT2002";

pub fn validate_domain(
    domain: &Domain,
    df: &DataFrame,
    ctx: &ValidationContext,
) -> ConformanceReport {
    let column_lookup = build_column_lookup(df);
    let p21_lookup = build_p21_lookup(ctx.p21_rules);
    let mut issues = Vec::new();
    for variable in &domain.variables {
        let column = column_lookup.get(&variable.name.to_uppercase());
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
        if let Some(ct_registry) = ctx.ct_registry {
            if let Some(ct) = resolve_ct(ct_registry, variable) {
                if let Some(issue) = ct_issue(
                    domain,
                    variable,
                    df,
                    column,
                    ct,
                    &p21_lookup,
                    &column_lookup,
                ) {
                    issues.push(issue);
                }
            }
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
        apply_reject_rules(&domain_map, &frame_map, p21_rules, &mut report_map);
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
                    })
                    .collect(),
            })
            .collect(),
    };
    let json = serde_json::to_string_pretty(&payload)?;
    std::fs::write(&output_path, format!("{json}\n"))?;
    Ok(output_path)
}

fn build_column_lookup(df: &DataFrame) -> BTreeMap<String, String> {
    df.get_column_names_owned()
        .into_iter()
        .map(|name| (name.to_uppercase(), name.to_string()))
        .collect()
}

fn missing_column_issues(
    _domain: &Domain,
    variable: &Variable,
    p21_lookup: &BTreeMap<String, &P21Rule>,
) -> Vec<ConformanceIssue> {
    if is_required(variable.core.as_deref()) {
        let rule = p21_lookup.get(RULE_REQUIRED_VAR_MISSING);
        let base = rule_base_message(rule, "SDTM Required variable not found");
        let message = format!("{base}: {}", variable.name);
        return vec![issue_from_rule(
            RULE_REQUIRED_VAR_MISSING,
            p21_lookup,
            IssueSeverity::Error,
            message,
            Some(variable.name.clone()),
            None,
            None,
        )];
    }
    if is_expected(variable.core.as_deref()) {
        let rule = p21_lookup.get(RULE_EXPECTED_VAR_MISSING);
        let base = rule_base_message(rule, "SDTM Expected variable not found");
        let message = format!("{base}: {}", variable.name);
        return vec![issue_from_rule(
            RULE_EXPECTED_VAR_MISSING,
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
    let rule = p21_lookup.get(RULE_REQUIRED_VALUE_MISSING);
    let base = rule_base_message(rule, "Null value in variable marked as Required");
    let message = format!(
        "{base}: {} has {} missing/blank value(s)",
        variable.name, missing
    );
    Some(issue_from_rule(
        RULE_REQUIRED_VALUE_MISSING,
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
    let rule = p21_lookup.get(RULE_DATATYPE_MISMATCH);
    let base = rule_base_message(
        rule,
        "Variable datatype is not the expected SDTM datatype",
    );
    let message = format!(
        "{base}: {} has {} non-numeric value(s)",
        variable.name, invalid
    );
    Some(issue_from_rule(
        RULE_DATATYPE_MISMATCH,
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
    let Some(limit) = variable.length else {
        return None;
    };
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
    let rule = p21_lookup.get(RULE_LENGTH_EXCEEDED);
    let base = rule_base_message(rule, "Variable value is longer than defined max length");
    let message = format!(
        "{base}: {} exceeds length {} in {} value(s)",
        variable.name, limit, over
    );
    Some(issue_from_rule(
        RULE_LENGTH_EXCEEDED,
        p21_lookup,
        IssueSeverity::Error,
        message,
        Some(variable.name.clone()),
        Some(over),
        None,
    ))
}

fn ct_issue(
    domain: &Domain,
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    ct: &ControlledTerminology,
    p21_lookup: &BTreeMap<String, &P21Rule>,
    column_lookup: &BTreeMap<String, String>,
) -> Option<ConformanceIssue> {
    let invalid = collect_invalid_ct_values(domain, variable, df, column, ct, column_lookup);
    if invalid.is_empty() {
        return None;
    }
    let (rule_id, default_severity) = if ct.extensible {
        (RULE_CT_EXTENSIBLE, IssueSeverity::Warning)
    } else {
        (RULE_CT_NON_EXTENSIBLE, IssueSeverity::Error)
    };
    let rule = p21_lookup.get(rule_id);
    let mut examples = invalid.iter().take(5).cloned().collect::<Vec<_>>();
    examples.sort();
    let examples = examples.join(", ");
    let base = rule_base_message(rule, "Variable value not found in codelist");
    let mut message = format!(
        "{}. {} contains {} value(s) not found in CT for {} ({}).",
        base,
        variable.name,
        invalid.len(),
        ct.codelist_name,
        ct.codelist_code
    );
    if !examples.is_empty() {
        message.push_str(&format!(" examples: {}", examples));
    }
    Some(issue_from_rule(
        rule_id,
        p21_lookup,
        default_severity,
        message,
        Some(variable.name.clone()),
        Some(invalid.len() as u64),
        Some(ct.codelist_code.clone()),
    ))
}

fn build_p21_lookup<'a>(rules: Option<&'a [P21Rule]>) -> BTreeMap<String, &'a P21Rule> {
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
    let rule = p21_lookup.get(&rule_id.to_uppercase());
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
    }
}

fn apply_reject_rules(
    domain_map: &BTreeMap<String, &Domain>,
    frame_map: &BTreeMap<String, &DataFrame>,
    p21_rules: &[P21Rule],
    report_map: &mut BTreeMap<String, ConformanceReport>,
) {
    let p21_lookup = build_p21_lookup(Some(p21_rules));
    apply_missing_dataset_rejects(domain_map, frame_map, p21_rules, report_map);
    apply_ts_sstdtc_rejects(domain_map, frame_map, &p21_lookup, report_map);
}

fn apply_missing_dataset_rejects(
    domain_map: &BTreeMap<String, &Domain>,
    frame_map: &BTreeMap<String, &DataFrame>,
    p21_rules: &[P21Rule],
    report_map: &mut BTreeMap<String, ConformanceReport>,
) {
    for rule in p21_rules {
        if parse_severity(rule) != Some(IssueSeverity::Reject) {
            continue;
        }
        let Some(code) = missing_dataset_code(&rule.message) else {
            continue;
        };
        if !domain_map.contains_key(&code) {
            continue;
        }
        if frame_map.contains_key(&code) {
            continue;
        }
        let issue = ConformanceIssue {
            code: rule.rule_id.clone(),
            message: rule_message(rule, None),
            severity: IssueSeverity::Reject,
            variable: None,
            count: None,
            rule_id: Some(rule.rule_id.clone()),
            category: rule.category.clone(),
            codelist_code: None,
        };
        add_report_issue(report_map, &code, issue);
    }
}

fn apply_ts_sstdtc_rejects(
    domain_map: &BTreeMap<String, &Domain>,
    frame_map: &BTreeMap<String, &DataFrame>,
    p21_lookup: &BTreeMap<String, &P21Rule>,
    report_map: &mut BTreeMap<String, ConformanceReport>,
) {
    let ts_domain = match domain_map.get("TS") {
        Some(domain) => *domain,
        None => return,
    };
    let df = match frame_map.get("TS") {
        Some(df) => *df,
        None => return,
    };
    let tsp_var = match standard_variable_name(ts_domain, "TSPARMCD") {
        Some(name) => name,
        None => return,
    };
    let tsval_var = match standard_variable_name(ts_domain, "TSVAL") {
        Some(name) => name,
        None => return,
    };
    let column_lookup = build_column_lookup(df);
    let tsp_col = match column_lookup.get(&tsp_var.to_uppercase()) {
        Some(name) => name.as_str(),
        None => return,
    };
    let tsval_col = match column_lookup.get(&tsval_var.to_uppercase()) {
        Some(name) => name.as_str(),
        None => return,
    };
    let tsp_series = match df.column(tsp_col) {
        Ok(series) => series,
        Err(_) => return,
    };
    let tsval_series = match df.column(tsval_col) {
        Ok(series) => series,
        Err(_) => return,
    };

    let mut sstdtc_rows = 0usize;
    let mut invalid_format = 0usize;
    let mut incomplete_date = 0usize;

    for idx in 0..df.height() {
        let tsp_value = any_to_string(tsp_series.get(idx).unwrap_or(AnyValue::Null));
        if tsp_value.trim().eq_ignore_ascii_case("SSTDTC") {
            sstdtc_rows += 1;
            let tsval = any_to_string(tsval_series.get(idx).unwrap_or(AnyValue::Null));
            let status = sstdtc_status(&tsval);
            if !status.iso_valid {
                invalid_format += 1;
            } else if !status.full_date {
                incomplete_date += 1;
            }
        }
    }

    if sstdtc_rows == 0 {
        if let Some(rule) = reject_rule(p21_lookup, "SD2232") {
            let issue = ConformanceIssue {
                code: rule.rule_id.clone(),
                message: rule_message(rule, None),
                severity: IssueSeverity::Reject,
                variable: Some(tsp_var.clone()),
                count: None,
                rule_id: Some(rule.rule_id.clone()),
                category: rule.category.clone(),
                codelist_code: None,
            };
            add_report_issue(report_map, "TS", issue);
        }
        return;
    }

    if invalid_format > 0 {
        if let Some(rule) = reject_rule(p21_lookup, "SD2247") {
            let issue = ConformanceIssue {
                code: rule.rule_id.clone(),
                message: rule_message(rule, Some(invalid_format)),
                severity: IssueSeverity::Reject,
                variable: Some(tsval_var.clone()),
                count: Some(invalid_format as u64),
                rule_id: Some(rule.rule_id.clone()),
                category: rule.category.clone(),
                codelist_code: None,
            };
            add_report_issue(report_map, "TS", issue);
        }
    }

    if incomplete_date > 0 {
        if let Some(rule) = reject_rule(p21_lookup, "SD2247A") {
            let issue = ConformanceIssue {
                code: rule.rule_id.clone(),
                message: rule_message(rule, Some(incomplete_date)),
                severity: IssueSeverity::Reject,
                variable: Some(tsval_var.clone()),
                count: Some(incomplete_date as u64),
                rule_id: Some(rule.rule_id.clone()),
                category: rule.category.clone(),
                codelist_code: None,
            };
            add_report_issue(report_map, "TS", issue);
        }
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

fn reject_rule<'a>(
    p21_lookup: &'a BTreeMap<String, &'a P21Rule>,
    rule_id: &str,
) -> Option<&'a P21Rule> {
    let rule = p21_lookup.get(&rule_id.to_uppercase())?;
    if parse_severity(rule) == Some(IssueSeverity::Reject) {
        Some(*rule)
    } else {
        None
    }
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

fn standard_variable_name(domain: &Domain, target: &str) -> Option<String> {
    domain
        .variables
        .iter()
        .find(|var| var.name.eq_ignore_ascii_case(target))
        .map(|var| var.name.clone())
}

struct SstdtcStatus {
    iso_valid: bool,
    full_date: bool,
}

fn sstdtc_status(value: &str) -> SstdtcStatus {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return SstdtcStatus {
            iso_valid: false,
            full_date: false,
        };
    }
    let date_part = trimmed
        .split('T')
        .next()
        .unwrap_or(trimmed)
        .split_whitespace()
        .next()
        .unwrap_or(trimmed);
    let parts: Vec<&str> = date_part.split('-').collect();
    match parts.len() {
        1 => SstdtcStatus {
            iso_valid: is_year(parts[0]),
            full_date: false,
        },
        2 => SstdtcStatus {
            iso_valid: is_year(parts[0]) && is_two_digit(parts[1]),
            full_date: false,
        },
        3 => {
            let iso_valid = is_year(parts[0]) && is_two_digit(parts[1]) && is_two_digit(parts[2]);
            SstdtcStatus {
                iso_valid,
                full_date: iso_valid,
            }
        }
        _ => SstdtcStatus {
            iso_valid: false,
            full_date: false,
        },
    }
}

fn is_year(value: &str) -> bool {
    value.len() == 4 && value.chars().all(|c| c.is_ascii_digit())
}

fn is_two_digit(value: &str) -> bool {
    value.len() == 2 && value.chars().all(|c| c.is_ascii_digit())
}

fn collect_invalid_ct_values(
    domain: &Domain,
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    ct: &ControlledTerminology,
    column_lookup: &BTreeMap<String, String>,
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

    let dscat_col = if domain.code.eq_ignore_ascii_case("DS")
        && variable.name.eq_ignore_ascii_case("DSDECOD")
    {
        column_lookup.get("DSCAT").cloned()
    } else {
        None
    };

    for idx in 0..df.height() {
        if let Some(dscat_col) = dscat_col.as_ref() {
            let dscat = column_value(df, dscat_col, idx).to_uppercase();
            if !dscat.is_empty() && dscat != "DISPOSITION EVENT" {
                continue;
            }
        }
        let raw = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if domain.code.eq_ignore_ascii_case("LB")
            && variable.name.eq_ignore_ascii_case("LBSTRESC")
            && is_numeric_like_text(trimmed)
        {
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
) -> Option<&'a ControlledTerminology> {
    if let Some(code_raw) = variable.codelist_code.as_ref() {
        for code in split_codelist_codes(code_raw) {
            let code_key = code.to_uppercase();
            if let Some(ct) = registry.by_code.get(&code_key) {
                return Some(ct);
            }
        }
    }
    let name_key = variable.name.to_uppercase();
    registry.by_name.get(&name_key)
}

fn split_codelist_codes(raw: &str) -> Vec<String> {
    let text = raw.trim();
    if text.is_empty() {
        return Vec::new();
    }
    for sep in [';', ',', ' '] {
        if text.contains(sep) {
            return text
                .split(sep)
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect();
        }
    }
    vec![text.to_string()]
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

fn is_missing_value(value: &AnyValue) -> bool {
    match value {
        AnyValue::Null => true,
        AnyValue::String(value) => value.trim().is_empty(),
        AnyValue::StringOwned(value) => value.trim().is_empty(),
        _ => false,
    }
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

fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        AnyValue::Null => String::new(),
        _ => value.to_string(),
    }
}

fn column_value(df: &DataFrame, name: &str, idx: usize) -> String {
    match df.column(name) {
        Ok(series) => any_to_string(series.get(idx).unwrap_or(AnyValue::Null)),
        Err(_) => String::new(),
    }
}

fn is_numeric_like_text(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    let cleaned = trimmed
        .strip_prefix("<=")
        .or_else(|| trimmed.strip_prefix(">="))
        .or_else(|| trimmed.strip_prefix('<'))
        .or_else(|| trimmed.strip_prefix('>'))
        .unwrap_or(trimmed)
        .trim();
    cleaned.parse::<f64>().is_ok()
}
