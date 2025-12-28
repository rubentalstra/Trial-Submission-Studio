pub mod cross_domain;
mod engine;
mod validator;

pub use cross_domain::{
    CrossDomainValidationInput, CrossDomainValidationResult, validate_cross_domain,
};
pub use engine::RuleEngine;
// Clean validator (per SDTM_CT_relationships.md)
pub use validator::{Issue, Severity, ValidationReport, Validator};

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use polars::prelude::{AnyValue, DataFrame};
use serde::Serialize;

use sdtm_core::ProvenanceTracker;
use sdtm_ingest::any_to_string;
use sdtm_model::ct::{Codelist, CtRegistry, ResolvedCodelist};
use sdtm_model::{
    CaseInsensitiveLookup, ConformanceIssue, ConformanceReport, Domain, IssueSeverity,
    OutputFormat, Variable,
};
use sdtm_standards::assumptions::RuleGenerator;

#[derive(Debug, Clone, Default)]
pub struct ValidationContext<'a> {
    pub ct_registry: Option<&'a CtRegistry>,
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
            ct_catalogs: None,
        }
    }

    pub fn with_ct_registry(mut self, ct_registry: &'a CtRegistry) -> Self {
        self.ct_registry = Some(ct_registry);
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

        let generator = RuleGenerator::new();

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
    let mut issues = Vec::new();

    // CT validation only - controlled terminology is our source of truth
    if let Some(ct_registry) = ctx.ct_registry {
        for variable in &domain.variables {
            let Some(column) = column_lookup.get(&variable.name) else {
                continue;
            };
            if let Some(resolved) = resolve_ct(ct_registry, variable, ctx.ct_catalogs.as_deref())
                && let Some(issue) = ct_issue(variable, df, column, &resolved)
            {
                issues.push(issue);
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

fn ct_issue(
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    resolved: &ResolvedCodelist,
) -> Option<ConformanceIssue> {
    let ct = resolved.codelist;
    let invalid = collect_invalid_ct_values(df, column, ct);
    if invalid.is_empty() {
        return None;
    }
    let severity = if ct.extensible {
        IssueSeverity::Warning
    } else {
        IssueSeverity::Error
    };
    let mut examples = invalid.iter().take(5).cloned().collect::<Vec<_>>();
    examples.sort();
    let examples = examples.join(", ");
    let mut message = format!(
        "Variable value not found in codelist. {} contains {} value(s) not found in {} for {} ({}).",
        variable.name,
        invalid.len(),
        resolved.source(),
        ct.name,
        ct.code
    );
    if !examples.is_empty() {
        message.push_str(&format!(" values: {}", examples));
    }
    Some(ConformanceIssue {
        code: ct.code.clone(),
        message,
        severity,
        variable: Some(variable.name.clone()),
        count: Some(invalid.len() as u64),
        category: Some(ct.code.clone()),
        codelist_code: Some(ct.code.clone()),
        ct_source: Some(resolved.source().to_string()),
    })
}

fn collect_invalid_ct_values(df: &DataFrame, column: &str, ct: &Codelist) -> BTreeSet<String> {
    let mut invalid = BTreeSet::new();
    let series = match df.column(column) {
        Ok(series) => series,
        Err(_) => return invalid,
    };
    let submission_values: BTreeSet<String> = ct
        .submission_values()
        .iter()
        .map(|value| value.to_uppercase())
        .collect();

    for idx in 0..df.height() {
        let raw = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let normalized = ct.normalize(trimmed);
        if normalized.is_empty() {
            continue;
        }
        let key = normalized.to_uppercase();
        if submission_values.contains(&key) {
            continue;
        }
        invalid.insert(trimmed.to_string());
    }
    invalid
}

fn resolve_ct<'a>(
    registry: &'a CtRegistry,
    variable: &Variable,
    preferred: Option<&[String]>,
) -> Option<ResolvedCodelist<'a>> {
    // Get codelist code from variable metadata
    let codelist_code = variable.codelist_code.as_ref()?;
    let code = codelist_code.split(';').next()?.trim();
    if code.is_empty() {
        return None;
    }
    registry.resolve(code, preferred)
}
