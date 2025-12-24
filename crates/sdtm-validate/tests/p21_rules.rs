use std::collections::BTreeMap;
use std::path::PathBuf;

use sdtm_model::{CellValue, DomainCode, Row, RowId, Table, VarName};
use sdtm_standards::StandardsRegistry;
use sdtm_validate::validate_table_against_standards;

fn required_vars_for(registry: &StandardsRegistry, domain: &str) -> Vec<String> {
    let mut out = Vec::new();
    for key in ["*", domain] {
        if let Some(vars) = registry.variables_by_domain.get(key) {
            for v in vars {
                if v.required.unwrap_or(false) {
                    out.push(v.var.clone());
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn expected_vars_for(registry: &StandardsRegistry, domain: &str) -> Vec<String> {
    let mut out = Vec::new();
    for key in ["*", domain] {
        if let Some(vars) = registry.variables_by_domain.get(key) {
            for v in vars {
                if v.core
                    .as_deref()
                    .is_some_and(|c| c.eq_ignore_ascii_case("exp"))
                {
                    out.push(v.var.clone());
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn find_domain_with_expected(registry: &StandardsRegistry) -> Option<(String, String)> {
    // Returns (domain, expected_var)
    let mut domains: Vec<String> = registry.datasets_by_domain.keys().cloned().collect();
    domains.sort();
    for domain in domains {
        let exp = expected_vars_for(registry, &domain);
        if let Some(first) = exp.into_iter().next() {
            return Some((domain, first));
        }
    }
    None
}

fn derive_row_id(source_id: &str, record_number: u64) -> RowId {
    // Deterministic test-only RowId, no extra dependencies.
    // Keep it stable and distinct per record; include source_id length to avoid
    // accidental collisions if a test ever reuses record_number.
    let b = (record_number as u8).wrapping_add(source_id.len() as u8);
    RowId::from_first_16_bytes_of_sha256([b; 32])
}

fn make_row(source_id: &str, record_number: u64, cells: &[(&str, &str)]) -> Row {
    let mut map: BTreeMap<VarName, CellValue> = BTreeMap::new();
    for (k, v) in cells {
        map.insert(
            VarName::new((*k).to_string()).unwrap(),
            if v.trim().is_empty() {
                CellValue::Missing
            } else {
                CellValue::Text((*v).to_string())
            },
        );
    }

    Row {
        id: derive_row_id(source_id, record_number),
        cells: map,
    }
}

fn has_issue(report: &sdtm_validate::ValidationReport, p21_id: &str) -> bool {
    report
        .issues
        .iter()
        .any(|i| i.p21_rule_id.as_deref() == Some(p21_id))
}

#[test]
fn emits_sd0004_domain_mismatch_when_domain_column_present() -> anyhow::Result<()> {
    let standards_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards");
    let (registry, _summary) = StandardsRegistry::verify_and_load(&standards_dir)?;

    let domain_code = DomainCode::new("DM")?;
    let columns = vec![VarName::new("DOMAIN")?];
    let mut table = Table::new(domain_code, columns);
    table.push_row(make_row("sd0004", 1, &[("DOMAIN", "AE")]));

    let report = validate_table_against_standards(&registry, &table);
    assert!(has_issue(&report, "SD0004"));

    Ok(())
}

#[test]
fn emits_sd0005_duplicate_seq_within_usubjid() -> anyhow::Result<()> {
    let standards_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards");
    let (registry, _summary) = StandardsRegistry::verify_and_load(&standards_dir)?;

    let domain_code = DomainCode::new("AE")?;
    let columns = vec![VarName::new("USUBJID")?, VarName::new("AESEQ")?];
    let mut table = Table::new(domain_code, columns);

    table.push_row(make_row("sd0005", 1, &[("USUBJID", "01"), ("AESEQ", "1")]));
    table.push_row(make_row("sd0005", 2, &[("USUBJID", "01"), ("AESEQ", "1")]));

    let report = validate_table_against_standards(&registry, &table);
    assert!(has_issue(&report, "SD0005"));

    Ok(())
}

#[test]
fn emits_sd0003_invalid_iso8601_for_dtc_value() -> anyhow::Result<()> {
    let standards_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards");
    let (registry, _summary) = StandardsRegistry::verify_and_load(&standards_dir)?;

    // Pick a common domain and a *DTC variable from it.
    let domain = "DM";
    let dtc_var = registry
        .variables_by_domain
        .get(domain)
        .and_then(|vars| {
            vars.iter()
                .map(|v| v.var.as_str())
                .find(|name| name.ends_with("DTC"))
        })
        .ok_or_else(|| anyhow::anyhow!("no *DTC variable found for domain {domain}"))?
        .to_string();

    let required = required_vars_for(&registry, domain);
    assert!(
        !required.is_empty(),
        "expected at least one required variable for {domain}"
    );

    // Build columns: required vars + chosen DTC var (if not already required)
    let mut columns: Vec<VarName> = required
        .iter()
        .map(|c| VarName::new(c.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    if !required.iter().any(|v| v == &dtc_var) {
        columns.push(VarName::new(dtc_var.clone())?);
    }

    let domain_code = DomainCode::new(domain)?;
    let mut table = Table::new(domain_code, columns);

    // Provide a value for every required var to avoid SD0002 noise.
    let mut cells: Vec<(String, String)> = required
        .iter()
        .map(|v| (v.clone(), "X".to_string()))
        .collect();
    cells.push((dtc_var.clone(), "not-a-date".to_string()));
    let cells_ref: Vec<(&str, &str)> = cells
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    table.push_row(make_row("sd0003", 1, &cells_ref));

    let report = validate_table_against_standards(&registry, &table);
    assert!(has_issue(&report, "SD0003"));

    Ok(())
}

#[test]
fn emits_sd0057_expected_variable_missing() -> anyhow::Result<()> {
    let standards_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards");
    let (registry, _summary) = StandardsRegistry::verify_and_load(&standards_dir)?;

    let (domain, expected_var) = find_domain_with_expected(&registry)
        .ok_or_else(|| anyhow::anyhow!("no domain with expected variables found in standards"))?;

    let required = required_vars_for(&registry, &domain);
    assert!(
        !required.is_empty(),
        "expected at least one required variable for {domain}"
    );

    // Build table with only required variables (intentionally omitting an expected variable).
    let columns: Vec<VarName> = required
        .iter()
        .map(|c| VarName::new(c.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    let domain_code = DomainCode::new(&domain)?;
    let mut table = Table::new(domain_code, columns);

    // Add one row with values for required vars.
    let cells: Vec<(String, String)> = required
        .iter()
        .map(|v| (v.clone(), "X".to_string()))
        .collect();
    let cells_ref: Vec<(&str, &str)> = cells
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    table.push_row(make_row("sd0057", 1, &cells_ref));

    let report = validate_table_against_standards(&registry, &table);
    assert!(
        has_issue(&report, "SD0057"),
        "expected SD0057 when omitting expected var {expected_var} in domain {domain}"
    );

    Ok(())
}
