#![deny(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::error::StandardsError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CtCodelist {
    pub code: String,
    pub name: Option<String>,
    pub extensible: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CtTerm {
    pub codelist_code: String,
    pub submission_value: String,
    pub code: Option<String>,
    pub decode: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CtIndex {
    pub codelists: BTreeMap<String, CtCodelist>,
    pub terms_by_codelist: BTreeMap<String, BTreeSet<String>>,
    pub term_details: BTreeMap<(String, String), CtTerm>,
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

fn parse_yes_no(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

/// Parses the CT export format used in `standards/ct/<date>/SDTM_CT_*.csv`.
///
/// We treat rows without a `Codelist Code` as "codelist header" rows.
pub fn parse_ct_csv(path: &Path) -> Result<CtIndex, StandardsError> {
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

    let idx_code = header_index(&headers, "Code");
    let idx_codelist_code = header_index(&headers, "Codelist Code");
    let idx_codelist_name = header_index(&headers, "Codelist Name");
    let idx_extensible = header_index(&headers, "Codelist Extensible (Yes/No)");
    let idx_submission_value = header_index(&headers, "CDISC Submission Value");
    let idx_decode = header_index(&headers, "NCI Preferred Term");

    let mut codelists: BTreeMap<String, CtCodelist> = BTreeMap::new();
    let mut terms_by_codelist: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut term_details: BTreeMap<(String, String), CtTerm> = BTreeMap::new();

    for row in reader.records() {
        let row = row.map_err(|e| StandardsError::Csv {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        let codelist_code = get_string(&row, idx_codelist_code);
        let codelist_name = get_string(&row, idx_codelist_name);
        let extensible = get_string(&row, idx_extensible)
            .as_deref()
            .and_then(parse_yes_no);

        match codelist_code {
            None => {
                // Codelist header row: create a synthetic code from the term group name.
                // In this CT export, the actual code appears on the *next* rows.
                // We skip indexing here because membership uses the code from term rows.
                // Still, this row can be used to infer whether lists are extensible.
                // No-op.
                let _ = (codelist_name, extensible);
            }
            Some(code) => {
                codelists
                    .entry(code.clone())
                    .and_modify(|c| {
                        if c.name.is_none() {
                            c.name = codelist_name.clone();
                        }
                        if c.extensible.is_none() {
                            c.extensible = extensible;
                        }
                    })
                    .or_insert(CtCodelist {
                        code: code.clone(),
                        name: codelist_name.clone(),
                        extensible,
                    });

                if let Some(submission_value) = get_string(&row, idx_submission_value) {
                    terms_by_codelist
                        .entry(code.clone())
                        .or_default()
                        .insert(submission_value.clone());

                    let term = CtTerm {
                        codelist_code: code.clone(),
                        submission_value: submission_value.clone(),
                        code: get_string(&row, idx_code),
                        decode: get_string(&row, idx_decode),
                    };
                    term_details.insert((code.clone(), submission_value), term);
                }
            }
        }
    }

    Ok(CtIndex {
        codelists,
        terms_by_codelist,
        term_details,
    })
}
