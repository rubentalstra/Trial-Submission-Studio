use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CtRegistry {
    pub by_code: BTreeMap<String, ControlledTerminology>,
    pub by_name: BTreeMap<String, ControlledTerminology>,
    pub by_submission: BTreeMap<String, ControlledTerminology>,
}
