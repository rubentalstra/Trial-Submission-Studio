#![deny(unsafe_code)]

use sdtm_model::{CellValue, Table, VarName};

pub struct SimpleMapper;

impl SimpleMapper {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl sdtm_core::pipeline::Mapper for SimpleMapper {
    fn map(&self, input: Table) -> anyhow::Result<Vec<Table>> {
        let mut out = input;

        // Phase 1: minimal, deterministic scaffolding.
        // For DM, set DOMAIN="DM" if missing.
        if out.domain.as_str().eq_ignore_ascii_case("DM") {
            ensure_constant_column(&mut out, "DOMAIN", "DM")?;
        }

        Ok(vec![out])
    }
}

fn ensure_constant_column(table: &mut Table, var: &str, value: &str) -> anyhow::Result<()> {
    let var_name = VarName::new(var.to_string())?;

    if !table.columns.iter().any(|c| c == &var_name) {
        table.columns.push(var_name.clone());
        table.columns.sort();
        table.columns.dedup();
    }

    for row in &mut table.rows {
        row.cells
            .entry(var_name.clone())
            .or_insert_with(|| CellValue::Text(value.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sdtm_core::pipeline::Mapper;
    use sdtm_model::{DomainCode, Row, RowId};

    use super::*;

    #[test]
    fn dm_mapper_sets_domain_column() {
        let mut table = Table::new(
            DomainCode::new("DM").unwrap(),
            vec![VarName::new("USUBJID").unwrap()],
        );

        let mut cells = BTreeMap::new();
        cells.insert(
            VarName::new("USUBJID").unwrap(),
            CellValue::Text("01-001".to_string()),
        );

        table.push_row(Row {
            id: RowId::from_first_16_bytes_of_sha256([0u8; 32]),
            cells,
        });

        let mapped = SimpleMapper::new().map(table).unwrap();
        assert_eq!(mapped.len(), 1);

        let dm = &mapped[0];
        let cols: Vec<String> = dm.columns.iter().map(|c| c.as_str().to_string()).collect();
        assert!(cols.contains(&"DOMAIN".to_string()));

        let domain_cell = dm.rows[0]
            .cells
            .get(&VarName::new("DOMAIN").unwrap())
            .unwrap();
        assert_eq!(domain_cell, &CellValue::Text("DM".to_string()));
    }
}
