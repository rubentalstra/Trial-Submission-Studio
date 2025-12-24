use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use csv::ReaderBuilder;

use sdtm_model::{
    ControlledTerminology, CtRegistry, DatasetMetadata, Domain, Variable, VariableType,
};

#[derive(Debug, Clone)]
pub struct P21Rule {
    pub rule_id: String,
    pub publisher_id: Option<String>,
    pub message: String,
    pub description: String,
    pub category: Option<String>,
    pub severity: Option<String>,
}

const DEFAULT_CT_VERSION: &str = "2024-03-29";
const DEFAULT_SDTMIG_VERSION: &str = "v3_4";
const DEFAULT_SDTM_VERSION: &str = "v2_0";
const STANDARDS_ENV_VAR: &str = "CDISC_STANDARDS_DIR";

pub fn default_standards_root() -> PathBuf {
    if let Ok(root) = std::env::var(STANDARDS_ENV_VAR) {
        return PathBuf::from(root);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards")
}

pub fn load_default_sdtm_ig_domains() -> Result<Vec<Domain>> {
    let root = default_standards_root();
    load_sdtm_ig_domains(&root.join("sdtmig").join(DEFAULT_SDTMIG_VERSION))
}

pub fn load_default_sdtm_domains() -> Result<Vec<Domain>> {
    let root = default_standards_root();
    load_sdtm_domains(&root.join("sdtm").join(DEFAULT_SDTM_VERSION))
}

pub fn load_default_ct_registry() -> Result<CtRegistry> {
    let root = default_standards_root();
    load_ct_registry(&root.join("ct").join(DEFAULT_CT_VERSION))
}

pub fn load_default_p21_rules() -> Result<Vec<P21Rule>> {
    let root = default_standards_root();
    load_p21_rules(&root.join("p21").join("Rules.csv"))
}

fn read_csv_rows(path: &Path) -> Result<Vec<BTreeMap<String, String>>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .with_context(|| format!("read csv: {}", path.display()))?;
    let headers = reader
        .headers()
        .with_context(|| format!("read headers: {}", path.display()))?
        .clone();
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.with_context(|| format!("read record: {}", path.display()))?;
        let mut row = BTreeMap::new();
        for (idx, value) in record.iter().enumerate() {
            let key = headers
                .get(idx)
                .unwrap_or("")
                .trim_matches('\u{feff}')
                .to_string();
            row.insert(key, value.trim().to_string());
        }
        rows.push(row);
    }
    Ok(rows)
}

fn csv_glob(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let mut matches = Vec::new();
    if !dir.exists() {
        return Ok(matches);
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if name.contains(pattern) && name.ends_with(".csv") {
            matches.push(path);
        }
    }
    matches.sort();
    Ok(matches)
}

fn parse_variable_type(raw: &str) -> VariableType {
    match raw.trim().to_lowercase().as_str() {
        "num" | "numeric" => VariableType::Num,
        _ => VariableType::Char,
    }
}

pub fn load_sdtm_ig_domains(base_dir: &Path) -> Result<Vec<Domain>> {
    let datasets = read_csv_rows(&base_dir.join("Datasets.csv"))?;
    let variables = read_csv_rows(&base_dir.join("Variables.csv"))?;
    build_domains(&datasets, &variables, "Dataset Name")
}

pub fn load_sdtm_domains(base_dir: &Path) -> Result<Vec<Domain>> {
    let datasets = read_csv_rows(&base_dir.join("Datasets.csv"))?;
    let variables = read_csv_rows(&base_dir.join("Variables.csv"))?;
    build_domains(&datasets, &variables, "Dataset Name")
}

fn build_domains(
    datasets: &[BTreeMap<String, String>],
    variables: &[BTreeMap<String, String>],
    dataset_key: &str,
) -> Result<Vec<Domain>> {
    let mut meta = BTreeMap::new();
    for row in datasets {
        let name = row.get(dataset_key).cloned().unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        meta.insert(
            name.to_uppercase(),
            DatasetMetadata {
                dataset_name: name.to_uppercase(),
                class_name: row.get("Class").filter(|v| !v.is_empty()).cloned(),
                label: row.get("Dataset Label").filter(|v| !v.is_empty()).cloned(),
                structure: row.get("Structure").filter(|v| !v.is_empty()).cloned(),
            },
        );
    }

    let mut grouped: BTreeMap<String, Vec<Variable>> = BTreeMap::new();
    for row in variables {
        let dataset = row
            .get(dataset_key)
            .cloned()
            .unwrap_or_default()
            .to_uppercase();
        let var_name = row.get("Variable Name").cloned().unwrap_or_default();
        if dataset.is_empty() || var_name.is_empty() {
            continue;
        }
        let variable = Variable {
            name: var_name,
            label: row.get("Variable Label").filter(|v| !v.is_empty()).cloned(),
            data_type: parse_variable_type(row.get("Type").map(String::as_str).unwrap_or("")),
            length: None,
            role: row.get("Role").filter(|v| !v.is_empty()).cloned(),
            core: row.get("Core").filter(|v| !v.is_empty()).cloned(),
            codelist_code: row
                .get("CDISC CT Codelist Code(s)")
                .filter(|v| !v.is_empty())
                .cloned(),
        };
        grouped.entry(dataset).or_default().push(variable);
    }

    let mut domains = Vec::new();
    for (code, vars) in grouped {
        let metadata = meta.get(&code);
        domains.push(Domain {
            code: code.clone(),
            description: metadata.and_then(|m| m.label.clone()),
            class_name: metadata.and_then(|m| m.class_name.clone()),
            label: metadata.and_then(|m| m.label.clone()),
            structure: metadata.and_then(|m| m.structure.clone()),
            dataset_name: Some(code.clone()),
            variables: vars,
        });
    }
    domains.sort_by(|a, b| a.code.cmp(&b.code));
    Ok(domains)
}

pub fn load_ct_registry(ct_dir: &Path) -> Result<CtRegistry> {
    let mut by_code = BTreeMap::new();
    let mut by_name = BTreeMap::new();
    let mut files = csv_glob(ct_dir, "CT_")?;
    files.sort();

    for path in files {
        let rows = read_csv_rows(&path)?;
        for row in rows {
            let codelist_code = row
                .get("Codelist Code")
                .cloned()
                .unwrap_or_default()
                .to_uppercase();
            if codelist_code.is_empty() {
                continue;
            }
            let codelist_name = row
                .get("Codelist Name")
                .cloned()
                .unwrap_or_else(|| codelist_code.clone());
            let extensible = row
                .get("Codelist Extensible (Yes/No)")
                .map(|v| v.eq_ignore_ascii_case("yes"))
                .unwrap_or(false);
            let submission_value = row
                .get("CDISC Submission Value")
                .cloned()
                .unwrap_or_default();
            if submission_value.is_empty() {
                continue;
            }
            let mut entry = by_code
                .remove(&codelist_code)
                .unwrap_or(ControlledTerminology {
                    codelist_code: codelist_code.clone(),
                    codelist_name: codelist_name.clone(),
                    extensible,
                    submission_values: Vec::new(),
                    synonyms: BTreeMap::new(),
                    submission_value_synonyms: BTreeMap::new(),
                    nci_codes: BTreeMap::new(),
                    definitions: BTreeMap::new(),
                    preferred_terms: BTreeMap::new(),
                    standards: Vec::new(),
                    sources: Vec::new(),
                });

            entry.extensible |= extensible;
            if !entry.submission_values.contains(&submission_value) {
                entry.submission_values.push(submission_value.clone());
            }
            if let Some(def) = row.get("CDISC Definition").filter(|v| !v.is_empty()) {
                entry
                    .definitions
                    .insert(submission_value.clone(), def.clone());
            }
            if let Some(pref) = row.get("NCI Preferred Term").filter(|v| !v.is_empty()) {
                entry
                    .preferred_terms
                    .insert(submission_value.clone(), pref.clone());
            }
            if let Some(code) = row.get("Code").filter(|v| !v.is_empty()) {
                entry
                    .nci_codes
                    .insert(submission_value.clone(), code.clone());
            }
            if let Some(standard) = row.get("Standard and Date").filter(|v| !v.is_empty()) {
                if !entry.standards.contains(standard) {
                    entry.standards.push(standard.clone());
                }
            }
            if let Some(source) = path.file_name().and_then(|v| v.to_str()) {
                if !entry.sources.contains(&source.to_string()) {
                    entry.sources.push(source.to_string());
                }
            }
            if let Some(synonyms_raw) = row.get("CDISC Synonym(s)").filter(|v| !v.is_empty()) {
                let mut syns = BTreeSet::new();
                if synonyms_raw.contains(';') {
                    for syn in synonyms_raw.split(';') {
                        syns.insert(syn.trim().to_string());
                    }
                } else if synonyms_raw.contains(',') {
                    for syn in synonyms_raw.split(',') {
                        syns.insert(syn.trim().to_string());
                    }
                } else {
                    syns.insert(synonyms_raw.trim().to_string());
                }
                if !syns.is_empty() {
                    entry
                        .submission_value_synonyms
                        .entry(submission_value.clone())
                        .or_insert_with(Vec::new)
                        .extend(syns.iter().cloned());
                    for syn in syns {
                        entry
                            .synonyms
                            .insert(syn.to_uppercase(), submission_value.clone());
                    }
                }
            }

            by_code.insert(codelist_code.clone(), entry);
        }
    }

    for entry in by_code.values() {
        let name_key = entry.codelist_name.to_uppercase();
        by_name.insert(name_key, entry.clone());
    }

    Ok(CtRegistry { by_code, by_name })
}

pub fn load_p21_rules(path: &Path) -> Result<Vec<P21Rule>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(path)
        .with_context(|| format!("read p21 rules: {}", path.display()))?;
    let headers = reader
        .headers()
        .with_context(|| format!("read p21 headers: {}", path.display()))?
        .clone();
    let mut rules = Vec::new();
    for record in reader.records() {
        let record = record.with_context(|| format!("read p21 record: {}", path.display()))?;
        let mut row = BTreeMap::new();
        for (idx, value) in record.iter().enumerate() {
            let key = headers
                .get(idx)
                .unwrap_or("")
                .trim_matches('\u{feff}')
                .to_string();
            row.insert(key, value.trim().to_string());
        }
        let rule_id = row.get("Pinnacle 21 ID").cloned().unwrap_or_default();
        if rule_id.is_empty() {
            continue;
        }
        rules.push(P21Rule {
            rule_id,
            publisher_id: row.get("Publisher ID").filter(|v| !v.is_empty()).cloned(),
            message: row.get("Message").cloned().unwrap_or_default(),
            description: row.get("Description").cloned().unwrap_or_default(),
            category: row.get("Category").filter(|v| !v.is_empty()).cloned(),
            severity: row.get("Severity").filter(|v| !v.is_empty()).cloned(),
        });
    }
    Ok(rules)
}
