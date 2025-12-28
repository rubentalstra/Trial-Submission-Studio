use polars::prelude::{AnyValue, DataFrame};
use sdtm_ingest::any_to_string;
use sdtm_model::ct::{Codelist, ResolvedCodelist, TerminologyRegistry};
use sdtm_model::{
    CaseInsensitiveSet, Domain, OutputFormat, Severity, ValidationIssue, ValidationReport, Variable,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default)]
pub struct ValidationContext<'a> {
    pub ct_registry: Option<&'a TerminologyRegistry>,
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
    reports: &[ValidationReport],
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
        Self { ct_registry: None }
    }

    pub fn with_ct_registry(mut self, ct_registry: &'a TerminologyRegistry) -> Self {
        self.ct_registry = Some(ct_registry);
        self
    }
}

pub fn validate_domain(
    domain: &Domain,
    df: &DataFrame,
    ctx: &ValidationContext,
) -> ValidationReport {
    let column_lookup = build_column_lookup(df);
    let mut issues = Vec::new();

    // CT validation only - controlled terminology is our source of truth
    if let Some(ct_registry) = ctx.ct_registry {
        for variable in &domain.variables {
            let Some(column) = column_lookup.get(&variable.name) else {
                continue;
            };
            if let Some(resolved) = resolve_ct(ct_registry, variable)
                && let Some(issue) = ct_issue(variable, df, column, &resolved)
            {
                issues.push(issue);
            }
        }
    }

    ValidationReport {
        domain_code: domain.code.clone(),
        issues,
    }
}

pub fn validate_domains(
    domains: &[Domain],
    frames: &[(&str, &DataFrame)],
    ctx: &ValidationContext,
) -> Vec<ValidationReport> {
    let mut domain_map: BTreeMap<String, &Domain> = BTreeMap::new();
    for domain in domains {
        domain_map.insert(domain.code.to_uppercase(), domain);
    }
    let mut report_map: BTreeMap<String, ValidationReport> = BTreeMap::new();
    for (domain_code, df) in frames {
        let code = domain_code.to_uppercase();
        if let Some(domain) = domain_map.get(&code) {
            report_map.insert(code.clone(), validate_domain(domain, df, ctx));
        }
    }
    report_map.into_values().collect()
}

fn build_column_lookup(df: &DataFrame) -> CaseInsensitiveSet {
    CaseInsensitiveSet::new(df.get_column_names_owned())
}

fn ct_issue(
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    resolved: &ResolvedCodelist,
) -> Option<ValidationIssue> {
    let ct = resolved.codelist;
    let invalid = collect_invalid_ct_values(df, column, ct);
    if invalid.is_empty() {
        return None;
    }
    const MAX_ALLOWED_VALUES_IN_MESSAGE: usize = 12;
    const MAX_INVALID_EXAMPLES: usize = 5;
    const MAX_CT_EXAMPLES: usize = 5;
    let severity = if ct.extensible {
        Severity::Warning
    } else {
        Severity::Error
    };
    let observed_values: Vec<String> = invalid.iter().take(MAX_INVALID_EXAMPLES).cloned().collect();
    let mut message = format!(
        "CT check failed. {} has {} value(s) not in {} ({}) from {}.",
        variable.name,
        invalid.len(),
        ct.name,
        ct.code,
        resolved.source()
    );
    if ct.extensible {
        message.push_str(" Codelist is extensible; invalid values are warnings.");
    }
    let allowed_values = ct.submission_values();
    let allowed_values_list = if allowed_values.len() <= MAX_ALLOWED_VALUES_IN_MESSAGE {
        Some(
            allowed_values
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        )
    } else {
        None
    };
    let allowed_count = if allowed_values.len() > MAX_ALLOWED_VALUES_IN_MESSAGE {
        Some(allowed_values.len() as u64)
    } else {
        None
    };
    let ct_examples = if allowed_values.len() > MAX_ALLOWED_VALUES_IN_MESSAGE {
        let mut examples = allowed_values.to_vec();
        examples.sort_unstable();
        examples.truncate(MAX_CT_EXAMPLES);
        Some(examples.into_iter().map(String::from).collect())
    } else {
        None
    };
    Some(ValidationIssue {
        code: ct.code.clone(),
        message,
        severity,
        variable: Some(variable.name.clone()),
        count: Some(invalid.len() as u64),
        ct_source: Some(resolved.source().to_string()),
        observed_values: if observed_values.is_empty() {
            None
        } else {
            Some(observed_values)
        },
        allowed_values: allowed_values_list,
        allowed_count,
        ct_examples,
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
    registry: &'a TerminologyRegistry,
    variable: &Variable,
) -> Option<ResolvedCodelist<'a>> {
    // Get codelist code from variable metadata
    let codelist_code = variable.codelist_code.as_ref()?;
    let code = codelist_code.split(';').next()?.trim();
    if code.is_empty() {
        return None;
    }
    registry.resolve(code, None)
}
