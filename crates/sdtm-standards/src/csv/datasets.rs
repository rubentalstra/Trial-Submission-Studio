#![deny(unsafe_code)]

use std::path::Path;

use crate::error::StandardsError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DatasetKey {
    pub domain: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DatasetMeta {
    pub source: String,
    pub version: String,
    pub class: Option<String>,
    pub domain: String,
    pub label: Option<String>,
    pub structure: Option<String>,
}

pub fn parse_datasets_csv(path: &Path, source: &str) -> Result<Vec<DatasetMeta>, StandardsError> {
    let bytes = std::fs::read(path).map_err(|e| StandardsError::io(path, e))?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes.as_slice());
    let headers = reader
        .headers()
        .map_err(|e| StandardsError::Csv {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?
        .clone();

    let mut results = Vec::new();
    for row in reader.records() {
        let row = row.map_err(|e| StandardsError::Csv {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        let get = |name: &str| -> Option<String> {
            headers
                .iter()
                .position(|h| h == name)
                .and_then(|i| row.get(i))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        };

        let domain = get("Dataset Name").ok_or_else(|| StandardsError::Csv {
            path: path.to_path_buf(),
            message: "missing Dataset Name".to_string(),
        })?;

        results.push(DatasetMeta {
            source: source.to_string(),
            version: get("Version").unwrap_or_default(),
            class: get("Class"),
            domain,
            label: get("Dataset Label"),
            structure: get("Structure"),
        });
    }

    results.sort_by(|a, b| a.domain.cmp(&b.domain));
    Ok(results)
}
