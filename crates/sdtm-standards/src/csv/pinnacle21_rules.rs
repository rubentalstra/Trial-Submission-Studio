#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use crate::error::StandardsError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct P21RuleMeta {
    pub p21_id: String,
    pub publisher_id: Option<String>,
    pub message: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub severity: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct P21RulesIndex {
    pub by_id: BTreeMap<String, P21RuleMeta>,
}

impl P21RulesIndex {
    pub fn get(&self, id: &str) -> Option<&P21RuleMeta> {
        self.by_id.get(id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

pub fn parse_pinnacle21_rules_csv(path: &Path) -> Result<P21RulesIndex, StandardsError> {
    let bytes = std::fs::read(path).map_err(|e| StandardsError::io(path, e))?;

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_reader(bytes.as_slice());

    let headers = reader
        .headers()
        .map_err(|e| StandardsError::Csv {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?
        .clone();

    let header_idx = |name: &str| -> Option<usize> { headers.iter().position(|h| h == name) };

    let id_i = header_idx("Pinnacle 21 ID").ok_or_else(|| StandardsError::Csv {
        path: path.to_path_buf(),
        message: "missing header: Pinnacle 21 ID".to_string(),
    })?;
    let publisher_i = header_idx("Publisher ID");
    let message_i = header_idx("Message").ok_or_else(|| StandardsError::Csv {
        path: path.to_path_buf(),
        message: "missing header: Message".to_string(),
    })?;
    let description_i = header_idx("Description");
    let category_i = header_idx("Category");
    let severity_i = header_idx("Severity");

    let mut by_id: BTreeMap<String, P21RuleMeta> = BTreeMap::new();

    for row in reader.records() {
        let row = row.map_err(|e| StandardsError::Csv {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        let get = |i: usize| -> Option<String> {
            row.get(i)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        };

        let p21_id = get(id_i).ok_or_else(|| StandardsError::Csv {
            path: path.to_path_buf(),
            message: "missing Pinnacle 21 ID".to_string(),
        })?;

        let message = get(message_i).ok_or_else(|| StandardsError::Csv {
            path: path.to_path_buf(),
            message: format!("missing Message for rule {p21_id}"),
        })?;

        let meta = P21RuleMeta {
            p21_id: p21_id.clone(),
            publisher_id: publisher_i.and_then(get),
            message,
            description: description_i.and_then(get),
            category: category_i.and_then(get),
            severity: severity_i.and_then(get),
        };

        // Keep first occurrence deterministically (BTreeMap ordering + stable insert guard).
        // The input file should not have duplicates, but this avoids non-deterministic overwrites.
        by_id.entry(p21_id).or_insert(meta);
    }

    Ok(P21RulesIndex { by_id })
}
