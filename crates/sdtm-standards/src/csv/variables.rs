#![deny(unsafe_code)]

use std::path::Path;

use crate::error::StandardsError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariableKey {
    pub domain: String,
    pub var: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VariableMeta {
    pub source: String,
    pub version: String,
    pub class: Option<String>,
    pub domain: String,
    pub var: String,
    pub label: Option<String>,
    pub data_type: Option<String>,
    pub role: Option<String>,
    pub required: Option<bool>,
    pub core: Option<String>,
    pub codelist_codes: Vec<String>,
}

fn header_index(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|h| h == name)
}

fn get_string(row: &csv::StringRecord, idx: Option<usize>) -> Option<String> {
    idx.and_then(|i| row.get(i))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn split_codes(s: Option<String>) -> Vec<String> {
    let Some(s) = s else {
        return Vec::new();
    };

    s.split([';', ',', ' '])
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect()
}

pub fn parse_variables_csv(path: &Path, source: &str) -> Result<Vec<VariableMeta>, StandardsError> {
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

    let idx_version = header_index(&headers, "Version");
    let idx_class = header_index(&headers, "Class");
    let idx_domain = header_index(&headers, "Dataset Name");
    let idx_var = header_index(&headers, "Variable Name");
    let idx_label = header_index(&headers, "Variable Label");
    let idx_type = header_index(&headers, "Type");
    let idx_role = header_index(&headers, "Role");
    let idx_core = header_index(&headers, "Core");
    let idx_codelist = header_index(&headers, "CDISC CT Codelist Code(s)");

    let mut results = Vec::new();
    for row in reader.records() {
        let row = row.map_err(|e| StandardsError::Csv {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        let var = get_string(&row, idx_var).ok_or_else(|| StandardsError::Csv {
            path: path.to_path_buf(),
            message: "missing Variable Name".to_string(),
        })?;
        let domain = get_string(&row, idx_domain).unwrap_or_else(|| "*".to_string());

        let core = get_string(&row, idx_core);
        let required = core.as_deref().map(|c| c.eq_ignore_ascii_case("req"));

        results.push(VariableMeta {
            source: source.to_string(),
            version: get_string(&row, idx_version).unwrap_or_default(),
            class: get_string(&row, idx_class),
            domain,
            var,
            label: get_string(&row, idx_label),
            data_type: get_string(&row, idx_type),
            role: get_string(&row, idx_role),
            required,
            core,
            codelist_codes: split_codes(get_string(&row, idx_codelist)),
        });
    }

    results.sort_by(|a, b| a.domain.cmp(&b.domain).then_with(|| a.var.cmp(&b.var)));
    Ok(results)
}
