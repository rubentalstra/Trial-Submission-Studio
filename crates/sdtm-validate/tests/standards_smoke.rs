use std::collections::BTreeSet;
use std::path::PathBuf;

use sdtm_model::{DomainCode, Table, VarName};
use sdtm_standards::StandardsRegistry;
use sdtm_validate::validate_table_against_standards;

fn required_vars_for(registry: &StandardsRegistry, domain: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
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

fn expected_vars_for(registry: &StandardsRegistry, domain: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
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

#[test]
fn dm_smoke_required_and_unknown_columns() -> anyhow::Result<()> {
    let standards_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards");
    let (registry, _summary) = StandardsRegistry::verify_and_load(&standards_dir)?;

    let domain = "DM";
    let required = required_vars_for(&registry, domain);
    let expected = expected_vars_for(&registry, domain);
    assert!(
        !required.is_empty(),
        "expected at least one required variable"
    );

    let domain_code = DomainCode::new(domain)?;

    // Build a table with all required + expected columns plus a clearly-unknown extra column.
    let mut columns: Vec<VarName> = required
        .iter()
        .chain(expected.iter())
        .map(|c| VarName::new(c.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    columns.push(VarName::new("ZZZ")?);

    let table = Table::new(domain_code.clone(), columns);
    let report = validate_table_against_standards(&registry, &table);
    assert_eq!(report.errors, 0);
    assert_eq!(report.warnings, 2);

    // Now remove one required variable and ensure we get exactly one error.
    let missing = required.iter().next().unwrap().clone();
    let columns_missing: Vec<VarName> = required
        .iter()
        .filter(|c| *c != &missing)
        .map(|c| VarName::new(c.clone()))
        .collect::<Result<Vec<_>, _>>()?;

    let table_missing = Table::new(domain_code, columns_missing);
    let report_missing = validate_table_against_standards(&registry, &table_missing);
    assert_eq!(report_missing.errors, 1);

    Ok(())
}
