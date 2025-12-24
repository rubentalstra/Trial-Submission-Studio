use std::collections::BTreeMap;
use std::path::PathBuf;

use sdtm_model::{CellValue, DomainCode, Row, RowId, Table, VarName};
use sdtm_standards::StandardsRegistry;
use sdtm_validate::validate_table_against_standards;

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
