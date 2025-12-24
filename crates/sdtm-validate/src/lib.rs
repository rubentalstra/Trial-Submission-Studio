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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p21_rule_id: Option<String>,
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
            p21_rule_id: Some("SD9999".to_string()),
            domain: domain.clone(),
            var: None,
            row_id: None,
            message: p21_message(registry, "SD9999")
                .unwrap_or_else(|| "unknown SDTM domain".to_string()),
        });
        report.errors += 1;
        return report;
    }

    // SD0001: No records in data source.
    // Note: The validator is used both for structural checks and ingested data.
    // Emit as a warning so structural-only tables (no rows) still work, while
    // callers validating real data can surface it.
    if table.rows.is_empty() {
        report.issues.push(ValidationIssue {
            severity: Severity::Warning,
            p21_rule_id: Some("SD0001".to_string()),
            domain: domain.clone(),
            var: None,
            row_id: None,
            message: p21_message(registry, "SD0001")
                .unwrap_or_else(|| "no records in data source".to_string()),
        });
        report.warnings += 1;
    }

    let required = required_vars_for_domain(registry, &domain);
    let expected = expected_vars_for_domain(registry, &domain);
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
                p21_rule_id: Some("SD0056".to_string()),
                domain: domain.clone(),
                var: Some(req.clone()),
                row_id: None,
                message: p21_message(registry, "SD0056")
                    .unwrap_or_else(|| "missing required variable".to_string()),
            });
            report.errors += 1;
        }
    }

    // SD0057: SDTM Expected variable not found.
    for exp in &expected {
        if !present.contains(exp) {
            report.issues.push(ValidationIssue {
                severity: Severity::Warning,
                p21_rule_id: Some("SD0057".to_string()),
                domain: domain.clone(),
                var: Some(exp.clone()),
                row_id: None,
                message: p21_message(registry, "SD0057")
                    .unwrap_or_else(|| "expected variable not found".to_string()),
            });
            report.warnings += 1;
        }
    }

    for col in &present {
        if !allowed.contains(col) {
            report.issues.push(ValidationIssue {
                severity: Severity::Warning,
                p21_rule_id: Some("SD0058".to_string()),
                domain: domain.clone(),
                var: Some(col.clone()),
                row_id: None,
                message: p21_message(registry, "SD0058")
                    .unwrap_or_else(|| "unknown variable for domain".to_string()),
            });
            report.warnings += 1;
        }
    }

    // SD0004: Inconsistent value for DOMAIN.
    // Only applies when the DOMAIN column is present.
    if present.contains("DOMAIN") {
        let var_name = VarName::new("DOMAIN")
            .expect("validated VarName from standards registry and table headers");
        for row in &table.rows {
            let value = match row.cells.get(&var_name) {
                Some(sdtm_model::CellValue::Text(v)) => v.trim(),
                _ => continue,
            };
            if value.is_empty() {
                continue;
            }
            if value != domain {
                report.issues.push(ValidationIssue {
                    severity: Severity::Error,
                    p21_rule_id: Some("SD0004".to_string()),
                    domain: domain.clone(),
                    var: Some("DOMAIN".to_string()),
                    row_id: Some(row.id.to_hex()),
                    message: p21_message(registry, "SD0004")
                        .unwrap_or_else(|| "inconsistent value for DOMAIN".to_string()),
                });
                report.errors += 1;
            }
        }
    }

    // SD0005: Duplicate value for --SEQ variable.
    // Exclusions: DE, DO, DT, DU, DX.
    if !matches!(domain.as_str(), "DE" | "DO" | "DT" | "DU" | "DX") {
        let seq_var = format!("{domain}SEQ");
        if present.contains(&seq_var) {
            let seq_name = VarName::new(seq_var.clone())
                .expect("validated VarName from standards registry and table headers");
            let usubjid_name = VarName::new("USUBJID")
                .expect("validated VarName from standards registry and table headers");
            let poolid_name = VarName::new("POOLID")
                .expect("validated VarName from standards registry and table headers");

            let group_by = if present.contains("USUBJID") {
                Some(usubjid_name)
            } else if present.contains("POOLID") {
                Some(poolid_name)
            } else {
                None
            };

            // group_key -> seq_value -> first_row_id_hex
            let mut seen: std::collections::BTreeMap<
                String,
                std::collections::BTreeMap<String, String>,
            > = std::collections::BTreeMap::new();

            for row in &table.rows {
                let seq_value = match row.cells.get(&seq_name) {
                    Some(sdtm_model::CellValue::Text(v)) => v.trim(),
                    _ => continue,
                };
                if seq_value.is_empty() {
                    continue;
                }

                let group_key = if let Some(ref g) = group_by {
                    match row.cells.get(g) {
                        Some(sdtm_model::CellValue::Text(v)) => v.trim().to_string(),
                        _ => String::new(),
                    }
                } else {
                    String::new()
                };

                let group = seen.entry(group_key).or_default();
                if group.contains_key(seq_value) {
                    report.issues.push(ValidationIssue {
                        severity: Severity::Error,
                        p21_rule_id: Some("SD0005".to_string()),
                        domain: domain.clone(),
                        var: Some(seq_var.clone()),
                        row_id: Some(row.id.to_hex()),
                        message: p21_message(registry, "SD0005")
                            .unwrap_or_else(|| "duplicate value for --SEQ variable".to_string()),
                    });
                    report.errors += 1;
                } else {
                    group.insert(seq_value.to_string(), row.id.to_hex());
                }
            }
        }
    }

    // SD0002: Null value in variable marked as Required (row-level enforcement).
    // Only applies when the variable exists as a column.
    for req in &required {
        if !present.contains(req) {
            continue;
        }
        let var_name = VarName::new(req.to_string())
            .expect("validated VarName from standards registry and table headers");
        for row in &table.rows {
            let cell = row.cells.get(&var_name);
            let is_missing = match cell {
                None => true,
                Some(sdtm_model::CellValue::Missing) => true,
                Some(sdtm_model::CellValue::Text(v)) => v.trim().is_empty(),
            };

            if is_missing {
                report.issues.push(ValidationIssue {
                    severity: Severity::Error,
                    p21_rule_id: Some("SD0002".to_string()),
                    domain: domain.clone(),
                    var: Some(req.clone()),
                    row_id: Some(row.id.to_hex()),
                    message: p21_message(registry, "SD0002")
                        .unwrap_or_else(|| "required variable is missing".to_string()),
                });
                report.errors += 1;
            }
        }
    }

    // SD0003: Invalid ISO 8601 value for variable.
    // Applies to all present *DTC variables.
    for col in &present {
        if !col.ends_with("DTC") {
            continue;
        }

        let Ok(var_name) = VarName::new(col.to_string()) else {
            continue;
        };

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

            if !is_iso8601_datetime(value) {
                report.issues.push(ValidationIssue {
                    severity: Severity::Error,
                    p21_rule_id: Some("SD0003".to_string()),
                    domain: domain.clone(),
                    var: Some(col.clone()),
                    row_id: Some(row.id.to_hex()),
                    message: p21_message(registry, "SD0003")
                        .unwrap_or_else(|| "invalid ISO 8601 value for variable".to_string()),
                });
                report.errors += 1;
            }
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
                let p21_id = if constraint.invalid_severity == Severity::Error {
                    "CT2001"
                } else {
                    "CT2002"
                };
                report.issues.push(ValidationIssue {
                    severity: constraint.invalid_severity,
                    p21_rule_id: Some(p21_id.to_string()),
                    domain: domain.clone(),
                    var: Some(col.clone()),
                    row_id: Some(row.id.to_hex()),
                    message: p21_message(registry, p21_id).unwrap_or_else(|| {
                        format!(
                            "invalid controlled terminology (codelists: {})",
                            constraint.codelist_codes.join(",")
                        )
                    }),
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
            .then_with(|| a.p21_rule_id.cmp(&b.p21_rule_id))
            .then_with(|| a.domain.cmp(&b.domain))
            .then_with(|| a.var.cmp(&b.var))
            .then_with(|| a.row_id.cmp(&b.row_id))
            .then_with(|| a.message.cmp(&b.message))
    });

    report
}

fn p21_message(registry: &StandardsRegistry, p21_id: &str) -> Option<String> {
    registry.p21_rules.get(p21_id).map(|m| m.message.clone())
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

fn expected_vars_for_domain(registry: &StandardsRegistry, domain: &str) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();

    // SD0057 refers to variables described as Expected in SDTM IG.
    // We infer Expected from Core == "Exp" (case-insensitive).
    // Prefer domain-specific, but also consider global (*) if present.
    for key in ["*", domain] {
        if let Some(vars) = registry.variables_by_domain.get(key) {
            for v in vars {
                if v.core
                    .as_deref()
                    .is_some_and(|c| c.eq_ignore_ascii_case("exp"))
                {
                    out.insert(v.var.clone());
                }
            }
        }
    }

    out
}

fn is_iso8601_datetime(value: &str) -> bool {
    // Accept a pragmatic ISO 8601 subset commonly used in SDTM *DTC variables:
    // - Date only: YYYY | YYYY-MM | YYYY-MM-DD
    // - Date-time: YYYY-MM-DDThh | YYYY-MM-DDThh:mm | YYYY-MM-DDThh:mm:ss[.frac][Z|±hh:mm]
    let s = value.trim();
    if s.is_empty() {
        return false;
    }

    let (date_part, time_part_opt) = match s.split_once('T') {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    };

    if !is_iso8601_date(date_part) {
        return false;
    }

    let Some(time_part) = time_part_opt else {
        return true;
    };

    is_iso8601_time_and_zone(time_part)
}

fn is_iso8601_date(date_part: &str) -> bool {
    let parts: Vec<&str> = date_part.split('-').collect();
    match parts.as_slice() {
        [yyyy] => is_n_digits(yyyy, 4),
        [yyyy, mm] => is_n_digits(yyyy, 4) && is_month(mm),
        [yyyy, mm, dd] => is_n_digits(yyyy, 4) && is_month(mm) && is_day(dd),
        _ => false,
    }
}

fn is_iso8601_time_and_zone(time_part: &str) -> bool {
    // Split timezone suffix.
    // - Z
    // - ±hh:mm
    let (time_only, zone_ok) = if let Some(rest) = time_part.strip_suffix('Z') {
        (rest, true)
    } else if let Some((time_only, zone)) = split_zone(time_part) {
        (time_only, is_zone(zone))
    } else {
        (time_part, true)
    };

    if !zone_ok {
        return false;
    }

    is_time_hms_with_optional_fraction(time_only)
}

fn split_zone(s: &str) -> Option<(&str, &str)> {
    // Find a '+' or '-' that starts the zone offset. We search from the end to
    // avoid catching the date part (which is already split off).
    let bytes = s.as_bytes();
    for i in (0..bytes.len()).rev() {
        match bytes[i] {
            b'+' | b'-' => {
                if i == 0 {
                    return None;
                }
                return Some((&s[..i], &s[i..]));
            }
            _ => {}
        }
    }
    None
}

fn is_zone(zone: &str) -> bool {
    // ±hh:mm
    if zone.len() != 6 {
        return false;
    }
    let sign = &zone[0..1];
    if sign != "+" && sign != "-" {
        return false;
    }
    let hh = &zone[1..3];
    let colon = &zone[3..4];
    let mm = &zone[4..6];
    colon == ":" && is_hour(hh) && is_minute(mm)
}

fn is_time_hms_with_optional_fraction(time_only: &str) -> bool {
    // hh | hh:mm | hh:mm:ss, optionally with fractional seconds.
    // Note: we only allow fractional seconds on the seconds component.
    let (base, frac_opt) = match time_only.split_once('.') {
        Some((a, b)) => (a, Some(b)),
        None => (time_only, None),
    };

    let parts: Vec<&str> = base.split(':').collect();
    let ok = match parts.as_slice() {
        [hh] => is_hour(hh),
        [hh, mm] => is_hour(hh) && is_minute(mm),
        [hh, mm, ss] => is_hour(hh) && is_minute(mm) && is_second(ss),
        _ => false,
    };

    if !ok {
        return false;
    }

    if let Some(frac) = frac_opt {
        // Fractional seconds must be 1+ digits.
        return !frac.is_empty() && frac.chars().all(|c| c.is_ascii_digit());
    }

    true
}

fn is_n_digits(s: &str, n: usize) -> bool {
    s.len() == n && s.chars().all(|c| c.is_ascii_digit())
}

fn is_month(mm: &str) -> bool {
    if !is_n_digits(mm, 2) {
        return false;
    }
    matches!(
        mm,
        "01" | "02" | "03" | "04" | "05" | "06" | "07" | "08" | "09" | "10" | "11" | "12"
    )
}

fn is_day(dd: &str) -> bool {
    if !is_n_digits(dd, 2) {
        return false;
    }
    let v: u8 = dd.parse().unwrap_or(0);
    (1..=31).contains(&v)
}

fn is_hour(hh: &str) -> bool {
    if !is_n_digits(hh, 2) {
        return false;
    }
    let v: u8 = hh.parse().unwrap_or(255);
    v <= 23
}

fn is_minute(mm: &str) -> bool {
    if !is_n_digits(mm, 2) {
        return false;
    }
    let v: u8 = mm.parse().unwrap_or(255);
    v <= 59
}

fn is_second(ss: &str) -> bool {
    if !is_n_digits(ss, 2) {
        return false;
    }
    let v: u8 = ss.parse().unwrap_or(255);
    v <= 59
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
