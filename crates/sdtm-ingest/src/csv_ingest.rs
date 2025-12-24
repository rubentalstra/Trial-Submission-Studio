#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use sha2::Digest;

use sdtm_model::{CellValue, DomainCode, Row, RowId, Table, VarName};

#[derive(Debug, Clone)]
pub struct CsvIngestOptions {
    /// Stable source identifier for provenance/RowId derivation (e.g. repo-relative path).
    pub source_id: String,
}

impl CsvIngestOptions {
    pub fn new(source_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
        }
    }
}

fn derive_row_id(source_id: &str, record_number: u64) -> RowId {
    // Deterministic: sha256("<source_id>\0<record_number>") and take first 16 bytes.
    let mut hasher = sha2::Sha256::new();
    hasher.update(source_id.as_bytes());
    hasher.update([0u8]);
    hasher.update(record_number.to_string().as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    RowId::from_first_16_bytes_of_sha256(digest)
}

pub fn ingest_csv_file(
    domain: DomainCode,
    csv_path: &Path,
    options: CsvIngestOptions,
) -> anyhow::Result<Table> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(csv_path)?;
    let headers = reader.headers()?.clone();

    let mut columns: Vec<VarName> = headers
        .iter()
        .map(|h| VarName::new(h.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    columns.sort();

    let mut table = Table::new(domain, columns.clone());

    for (idx, record) in reader.records().enumerate() {
        let record = record?;
        let record_number = (idx as u64) + 1;

        let mut cells: BTreeMap<VarName, CellValue> = BTreeMap::new();
        for (h, v) in headers.iter().zip(record.iter()) {
            let name = VarName::new(h.to_string())?;
            let value = v.trim();
            let cell = if value.is_empty() {
                CellValue::Missing
            } else {
                CellValue::Text(value.to_string())
            };
            cells.insert(name, cell);
        }

        table.push_row(Row {
            id: derive_row_id(&options.source_id, record_number),
            cells,
        });
    }

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_id_is_deterministic() {
        let a = derive_row_id("inputs/dm.csv", 1);
        let b = derive_row_id("inputs/dm.csv", 1);
        let c = derive_row_id("inputs/dm.csv", 2);
        let d = derive_row_id("inputs/ae.csv", 1);

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn ingest_csv_sorts_columns_but_preserves_rows() {
        let dir = std::env::temp_dir().join(format!(
            "sdtm-ingest-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let csv_path = dir.join("dm.csv");
        std::fs::write(&csv_path, "B,A\n2,1\n4,3\n").unwrap();

        let table = ingest_csv_file(
            DomainCode::new("DM").unwrap(),
            &csv_path,
            CsvIngestOptions::new("inputs/dm.csv"),
        )
        .unwrap();

        let cols: Vec<String> = table
            .columns
            .iter()
            .map(|c| c.as_str().to_string())
            .collect();
        assert_eq!(cols, vec!["A".to_string(), "B".to_string()]);
        assert_eq!(table.rows.len(), 2);
    }
}
