#![deny(unsafe_code)]

use std::collections::BTreeSet;
use std::path::Path;

use sdtm_core::pipeline::ValidateOutput;
use sdtm_model::{DomainCode, Table, VarName};
use sdtm_standards::StandardsRegistry;

#[derive(Debug, thiserror::Error)]
pub enum ValidateError {
    #[error("invalid domain code: {0}")]
    InvalidDomain(String),

    #[error("invalid variable name: {0}")]
    InvalidVarName(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub domain: String,
    pub var: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub errors: usize,
    pub warnings: usize,
    pub issues: Vec<ValidationIssue>,
}

pub fn validate_table_against_standards(
    registry: &StandardsRegistry,
    table: &Table,
) -> ValidationReport {
    let domain = table.domain.as_str().to_string();

    let mut report = ValidationReport::default();

    if !registry.datasets_by_domain.contains_key(&domain) {
        report.issues.push(ValidationIssue {
            severity: Severity::Error,
            domain: domain.clone(),
            var: None,
            message: "unknown SDTM domain".to_string(),
        });
        report.errors += 1;
        return report;
    }

    let required = required_vars_for_domain(registry, &domain);
    let allowed = allowed_vars_for_domain(registry, &domain);

    let present: BTreeSet<String> = table
        .columns
        .iter()
        .map(|v| v.as_str().to_string())
        .collect();

    for req in &required {
        if !present.contains(req) {
            report.issues.push(ValidationIssue {
                severity: Severity::Error,
                domain: domain.clone(),
                var: Some(req.clone()),
                message: "missing required variable".to_string(),
            });
            report.errors += 1;
        }
    }

    for col in &present {
        if !allowed.contains(col) {
            report.issues.push(ValidationIssue {
                severity: Severity::Warning,
                domain: domain.clone(),
                var: Some(col.clone()),
                message: "unknown variable for domain".to_string(),
            });
            report.warnings += 1;
        }
    }

    report
}

fn required_vars_for_domain(registry: &StandardsRegistry, domain: &str) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();

    for key in ["*", domain] {
        if let Some(vars) = registry.variables_by_domain.get(key) {
            for v in vars {
                if v.required.unwrap_or(false) {
                    out.insert(v.var.clone());
                }
            }
        }
    }

    out
}

fn allowed_vars_for_domain(registry: &StandardsRegistry, domain: &str) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    for key in ["*", domain] {
        if let Some(vars) = registry.variables_by_domain.get(key) {
            for v in vars {
                out.insert(v.var.clone());
            }
        }
    }
    out
}

pub struct StandardsValidator {
    registry: StandardsRegistry,
}

impl StandardsValidator {
    pub fn from_standards_dir(standards_dir: &Path) -> anyhow::Result<Self> {
        let (registry, _summary) = StandardsRegistry::verify_and_load(standards_dir)?;
        Ok(Self { registry })
    }

    pub fn validate_with_report(&self, tables: &[Table]) -> ValidationReport {
        let mut out = ValidationReport::default();
        for t in tables {
            let r = validate_table_against_standards(&self.registry, t);
            out.errors += r.errors;
            out.warnings += r.warnings;
            out.issues.extend(r.issues);
        }
        out
    }
}

impl sdtm_core::pipeline::Validator for StandardsValidator {
    fn validate(&self, tables: &[Table]) -> anyhow::Result<ValidateOutput> {
        let report = self.validate_with_report(tables);
        Ok(ValidateOutput {
            errors: report.errors,
            warnings: report.warnings,
        })
    }
}

pub fn table_from_columns(domain: &str, columns: &[&str]) -> Result<Table, ValidateError> {
    let domain_code =
        DomainCode::new(domain).map_err(|_| ValidateError::InvalidDomain(domain.into()))?;
    let mut out_cols: Vec<VarName> = Vec::with_capacity(columns.len());
    for c in columns {
        out_cols
            .push(VarName::new(*c).map_err(|_| ValidateError::InvalidVarName((*c).to_string()))?);
    }
    Ok(Table::new(domain_code, out_cols))
}
