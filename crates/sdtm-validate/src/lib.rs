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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, PartialOrd, Ord,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub domain: String,
    pub var: Option<String>,
    pub row_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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
            row_id: None,
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
                row_id: None,
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
                row_id: None,
                message: "unknown variable for domain".to_string(),
            });
            report.warnings += 1;
        }
    }

    // Controlled Terminology (CT) membership checks.
    // Only run CT checks for known variables (present + allowed) that have codelist codes.
    for col in &present {
        if !allowed.contains(col) {
            continue;
        }

        let Some(constraint) = ct_constraint_for_var(registry, &domain, col) else {
            continue;
        };

        let var_name = VarName::new(col.to_string())
            .expect("validated VarName from standards registry and table headers");

        for row in &table.rows {
            let Some(cell) = row.cells.get(&var_name) else {
                continue;
            };

            let value = match cell {
                sdtm_model::CellValue::Text(v) => v.trim(),
                sdtm_model::CellValue::Missing => continue,
            };

            if value.is_empty() {
                continue;
            }

            if !constraint.allowed_values.contains(value) {
                report.issues.push(ValidationIssue {
                    severity: constraint.invalid_severity,
                    domain: domain.clone(),
                    var: Some(col.clone()),
                    row_id: Some(row.id.to_hex()),
                    message: format!(
                        "invalid controlled terminology (codelists: {})",
                        constraint.codelist_codes.join(",")
                    ),
                });
                match constraint.invalid_severity {
                    Severity::Error => report.errors += 1,
                    Severity::Warning => report.warnings += 1,
                }
            }
        }
    }

    report.issues.sort_by(|a, b| {
        a.severity
            .cmp(&b.severity)
            .then_with(|| a.domain.cmp(&b.domain))
            .then_with(|| a.var.cmp(&b.var))
            .then_with(|| a.row_id.cmp(&b.row_id))
            .then_with(|| a.message.cmp(&b.message))
    });

    report
}

#[derive(Debug, Clone)]
struct CtConstraint {
    codelist_codes: Vec<String>,
    allowed_values: BTreeSet<String>,
    invalid_severity: Severity,
}

fn ct_constraint_for_var(
    registry: &StandardsRegistry,
    domain: &str,
    var: &str,
) -> Option<CtConstraint> {
    let mut codelist_codes: Vec<String> = lookup_codelist_codes(registry, domain, var)?;
    codelist_codes.sort();
    codelist_codes.dedup();

    let mut allowed_values: BTreeSet<String> = BTreeSet::new();
    let mut any_non_extensible = false;
    for code in &codelist_codes {
        let extensible = registry
            .ct_sdtm
            .codelists
            .get(code)
            .and_then(|c| c.extensible)
            .unwrap_or(false);
        if !extensible {
            any_non_extensible = true;
        }
        if let Some(values) = registry.ct_sdtm.terms_by_codelist.get(code) {
            allowed_values.extend(values.iter().cloned());
        }
    }

    if allowed_values.is_empty() {
        return None;
    }

    Some(CtConstraint {
        codelist_codes,
        allowed_values,
        invalid_severity: if any_non_extensible {
            Severity::Error
        } else {
            Severity::Warning
        },
    })
}

fn lookup_codelist_codes(
    registry: &StandardsRegistry,
    domain: &str,
    var: &str,
) -> Option<Vec<String>> {
    // NOTE: StandardsRegistry stores variables in a domain -> Vec<VariableMeta> index.
    // Prefer domain-specific over global (*).
    if let Some(vars) = registry.variables_by_domain.get(domain)
        && let Some(m) = vars.iter().find(|m| m.var == var)
        && !m.codelist_codes.is_empty()
    {
        return Some(m.codelist_codes.clone());
    }

    if let Some(vars) = registry.variables_by_domain.get("*")
        && let Some(m) = vars.iter().find(|m| m.var == var)
        && !m.codelist_codes.is_empty()
    {
        return Some(m.codelist_codes.clone());
    }
    None
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
