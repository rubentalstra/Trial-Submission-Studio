use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use csv::ReaderBuilder;

use sdtm_model::{DatasetClass, DatasetMetadata, Domain, Variable, VariableType};

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
        let class_name = row.get("Class").filter(|v| !v.is_empty()).cloned();
        // Parse the class name into the strongly-typed DatasetClass enum
        let dataset_class = class_name
            .as_ref()
            .and_then(|c| c.parse::<DatasetClass>().ok());
        meta.insert(
            name.to_uppercase(),
            DatasetMetadata {
                dataset_name: name.to_uppercase(),
                class_name,
                dataset_class,
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
        let order = row
            .get("Variable Order")
            .and_then(|value| value.trim().parse::<u32>().ok());
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
            order,
        };
        grouped.entry(dataset).or_default().push(variable);
    }

    let mut domains = Vec::new();
    for (code, mut vars) in grouped {
        let metadata = meta.get(&code);
        vars.sort_by(compare_variable_order);
        domains.push(Domain {
            code: code.clone(),
            description: metadata.and_then(|m| m.label.clone()),
            class_name: metadata.and_then(|m| m.class_name.clone()),
            dataset_class: metadata.and_then(|m| m.dataset_class),
            label: metadata.and_then(|m| m.label.clone()),
            structure: metadata.and_then(|m| m.structure.clone()),
            dataset_name: Some(code.clone()),
            variables: vars,
        });
    }
    domains.sort_by(|a, b| a.code.cmp(&b.code));
    Ok(domains)
}

fn compare_variable_order(left: &Variable, right: &Variable) -> std::cmp::Ordering {
    match (left.order, right.order) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.name.to_uppercase().cmp(&right.name.to_uppercase()),
    }
}

/// A registry of SDTM domains that allows querying by code and class.
/// Per SDTMIG v3.4 Chapter 2, domains are organized by observation class.
#[derive(Debug, Clone, Default)]
pub struct DomainRegistry {
    /// All domains indexed by uppercase code
    domains_by_code: BTreeMap<String, Domain>,
    /// Domain codes grouped by dataset class
    domains_by_class: BTreeMap<DatasetClass, Vec<String>>,
}

impl DomainRegistry {
    /// Create a new registry from a list of domains.
    pub fn new(domains: Vec<Domain>) -> Self {
        let mut registry = Self::default();
        for domain in domains {
            registry.insert(domain);
        }
        registry
    }

    /// Insert a domain into the registry.
    pub fn insert(&mut self, domain: Domain) {
        let code = domain.code.to_uppercase();
        if let Some(class) = domain.dataset_class {
            self.domains_by_class
                .entry(class)
                .or_default()
                .push(code.clone());
        }
        self.domains_by_code.insert(code, domain);
    }

    /// Get a domain by its code (case-insensitive).
    pub fn get(&self, code: &str) -> Option<&Domain> {
        self.domains_by_code.get(&code.to_uppercase())
    }

    /// Get all domains of a specific class.
    pub fn get_by_class(&self, class: DatasetClass) -> Vec<&Domain> {
        self.domains_by_class
            .get(&class)
            .map(|codes| {
                codes
                    .iter()
                    .filter_map(|code| self.domains_by_code.get(code))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all General Observation class domains (Interventions, Events, Findings, Findings About).
    /// Per SDTMIG v3.4 Section 2.1.
    pub fn get_general_observation_domains(&self) -> Vec<&Domain> {
        let mut domains = Vec::new();
        for class in [
            DatasetClass::Interventions,
            DatasetClass::Events,
            DatasetClass::Findings,
            DatasetClass::FindingsAbout,
        ] {
            domains.extend(self.get_by_class(class));
        }
        domains
    }

    /// Get all Special-Purpose domains (CO, DM, SE, SM, SV).
    /// Per SDTMIG v3.4 Chapter 5.
    pub fn get_special_purpose_domains(&self) -> Vec<&Domain> {
        self.get_by_class(DatasetClass::SpecialPurpose)
    }

    /// Get all Trial Design domains (TA, TD, TE, TI, TM, TS, TV).
    /// Per SDTMIG v3.4 Chapter 7.
    pub fn get_trial_design_domains(&self) -> Vec<&Domain> {
        self.get_by_class(DatasetClass::TrialDesign)
    }

    /// Get all Relationship datasets (RELREC, RELSPEC, RELSUB, SUPPQUAL).
    /// Per SDTMIG v3.4 Chapter 8.
    pub fn get_relationship_domains(&self) -> Vec<&Domain> {
        self.get_by_class(DatasetClass::Relationship)
    }

    /// Get the dataset class for a domain code.
    pub fn get_class(&self, code: &str) -> Option<DatasetClass> {
        self.get(code).and_then(|d| d.dataset_class)
    }

    /// Check if a domain code belongs to a specific class.
    pub fn is_class(&self, code: &str, class: DatasetClass) -> bool {
        self.get_class(code) == Some(class)
    }

    /// Check if a domain is a General Observation domain.
    pub fn is_general_observation(&self, code: &str) -> bool {
        self.get(code)
            .map(|d| d.is_general_observation())
            .unwrap_or(false)
    }

    /// Get all domain codes.
    pub fn codes(&self) -> impl Iterator<Item = &String> {
        self.domains_by_code.keys()
    }

    /// Get all domains.
    pub fn domains(&self) -> impl Iterator<Item = &Domain> {
        self.domains_by_code.values()
    }

    /// Get the number of domains in the registry.
    pub fn len(&self) -> usize {
        self.domains_by_code.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.domains_by_code.is_empty()
    }
}

/// Load the default SDTMIG domain registry.
pub fn load_default_domain_registry() -> Result<DomainRegistry> {
    let domains = load_default_sdtm_ig_domains()?;
    Ok(DomainRegistry::new(domains))
}
