use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Variable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlledTerminology {
    pub codelist_code: String,
    pub codelist_name: String,
    pub extensible: bool,
    pub submission_values: Vec<String>,
    pub synonyms: BTreeMap<String, String>,
    pub submission_value_synonyms: BTreeMap<String, Vec<String>>,
    pub nci_codes: BTreeMap<String, String>,
    pub definitions: BTreeMap<String, String>,
    pub preferred_terms: BTreeMap<String, String>,
    pub standards: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtCatalog {
    /// Display label (e.g., "SDTM CT")
    pub label: String,
    /// Release version/date (e.g., "2024-03-29")
    pub version: Option<String>,
    /// Publishing set (e.g., "SDTM", "SEND", "DEFINE-XML")
    pub publishing_set: Option<String>,
    pub by_code: BTreeMap<String, ControlledTerminology>,
    pub by_name: BTreeMap<String, ControlledTerminology>,
    pub by_submission: BTreeMap<String, ControlledTerminology>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CtRegistry {
    pub catalogs: BTreeMap<String, CtCatalog>,
}

pub struct ResolvedCt<'a> {
    pub ct: &'a ControlledTerminology,
    /// Catalog label (e.g., "SDTM CT")
    pub source: &'a str,
    /// Reference to the catalog for full metadata access
    pub catalog: &'a CtCatalog,
}

impl CtRegistry {
    pub fn resolve_by_code<'a>(
        &'a self,
        code: &str,
        preferred: Option<&[String]>,
    ) -> Option<ResolvedCt<'a>> {
        let catalogs = catalogs_in_order(&self.catalogs, preferred);
        let code_key = code.trim().to_uppercase();
        if code_key.is_empty() {
            return None;
        }
        for catalog in catalogs {
            if let Some(ct) = catalog.by_code.get(&code_key) {
                return Some(ResolvedCt {
                    ct,
                    source: catalog.label.as_str(),
                    catalog,
                });
            }
        }
        None
    }

    pub fn resolve_for_variable<'a>(
        &'a self,
        variable: &Variable,
        preferred: Option<&[String]>,
    ) -> Option<ResolvedCt<'a>> {
        let catalogs = catalogs_in_order(&self.catalogs, preferred);
        if let Some(code_raw) = variable.codelist_code.as_ref() {
            for code in split_codelist_codes(code_raw) {
                let code_key = code.to_uppercase();
                for catalog in &catalogs {
                    if let Some(ct) = catalog.by_code.get(&code_key) {
                        return Some(ResolvedCt {
                            ct,
                            source: catalog.label.as_str(),
                            catalog,
                        });
                    }
                }
            }
        }
        let name_key = variable.name.to_uppercase();
        for catalog in &catalogs {
            if let Some(ct) = catalog.by_submission.get(&name_key) {
                return Some(ResolvedCt {
                    ct,
                    source: catalog.label.as_str(),
                    catalog,
                });
            }
            if let Some(ct) = catalog.by_name.get(&name_key) {
                return Some(ResolvedCt {
                    ct,
                    source: catalog.label.as_str(),
                    catalog,
                });
            }
        }
        None
    }
}

fn catalogs_in_order<'a>(
    catalogs: &'a BTreeMap<String, CtCatalog>,
    preferred: Option<&[String]>,
) -> Vec<&'a CtCatalog> {
    if let Some(preferred) = preferred {
        let mut ordered = Vec::new();
        for label in preferred {
            let key = label.to_uppercase();
            if let Some(catalog) = catalogs.get(&key) {
                ordered.push(catalog);
            }
        }
        return ordered;
    }
    let mut values: Vec<&CtCatalog> = catalogs.values().collect();
    values
        .sort_by(|left, right| catalog_sort_key(&left.label).cmp(&catalog_sort_key(&right.label)));
    values
}

fn catalog_sort_key(label: &str) -> (u8, String) {
    let upper = label.to_uppercase();
    let rank = match upper.as_str() {
        "SDTM CT" => 0,
        "SEND CT" => 1,
        _ => 2,
    };
    (rank, upper)
}

fn split_codelist_codes(raw: &str) -> Vec<String> {
    let text = raw.trim();
    if text.is_empty() {
        return Vec::new();
    }
    for sep in [';', ',', ' '] {
        if text.contains(sep) {
            return text
                .split(sep)
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect();
        }
    }
    vec![text.to_string()]
}
