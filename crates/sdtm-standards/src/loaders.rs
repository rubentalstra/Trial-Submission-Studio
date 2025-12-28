use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use csv::ReaderBuilder;

use sdtm_model::{
    ControlledTerminology, CtCatalog, CtRegistry, DatasetClass, DatasetMetadata, Domain, Variable,
    VariableType,
};

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
    let mut ct_dirs = Vec::new();
    ct_dirs.push(root.join("ct").join(DEFAULT_CT_VERSION));
    let docs_ct = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Controlled_Terminology")
        .join(DEFAULT_CT_VERSION);
    ct_dirs.push(docs_ct);
    load_ct_registry(&ct_dirs)
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

pub fn load_ct_registry(ct_dirs: &[PathBuf]) -> Result<CtRegistry> {
    let mut catalogs = BTreeMap::new();
    let files = collect_ct_files(ct_dirs)?;

    for path in files {
        let catalog = load_ct_catalog(&path)?;
        catalogs.insert(catalog.label.to_uppercase(), catalog);
    }

    Ok(CtRegistry { catalogs })
}

pub fn load_ct_catalog(path: &Path) -> Result<CtCatalog> {
    let rows = read_csv_rows(path)?;
    let label = ct_label_from_filename(path);
    let mut by_code = BTreeMap::new();
    let mut by_name = BTreeMap::new();
    let mut by_submission = BTreeMap::new();
    let mut submission_by_code = BTreeMap::new();

    for row in rows {
        let code = row.get("Code").cloned().unwrap_or_default().to_uppercase();
        let codelist_code = row
            .get("Codelist Code")
            .cloned()
            .unwrap_or_default()
            .to_uppercase();
        let codelist_name = row.get("Codelist Name").cloned().unwrap_or_else(|| {
            if codelist_code.is_empty() {
                code.clone()
            } else {
                codelist_code.clone()
            }
        });
        let extensible = row
            .get("Codelist Extensible (Yes/No)")
            .map(|v| v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false);
        if codelist_code.is_empty() {
            if code.is_empty() {
                continue;
            }
            if let Some(submission) = row
                .get("CDISC Submission Value")
                .filter(|value| !value.is_empty())
            {
                submission_by_code.insert(code.clone(), submission.to_uppercase());
            }
            let mut entry = by_code.remove(&code).unwrap_or(ControlledTerminology {
                codelist_code: code.clone(),
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
            entry.codelist_name = codelist_name.clone();
            entry.extensible |= extensible;
            if let Some(standard) = row.get("Standard and Date").filter(|v| !v.is_empty())
                && !entry.standards.contains(standard)
            {
                entry.standards.push(standard.clone());
            }
            if let Some(source) = path.file_name().and_then(|v| v.to_str())
                && !entry.sources.contains(&source.to_string())
            {
                entry.sources.push(source.to_string());
            }
            by_code.insert(code.clone(), entry);
            continue;
        }
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
            insert_ct_alias(&mut entry, &submission_value, pref);
        }
        if let Some(code) = row.get("Code").filter(|v| !v.is_empty()) {
            entry
                .nci_codes
                .insert(submission_value.clone(), code.clone());
            insert_ct_alias(&mut entry, &submission_value, code);
        }
        if let Some(standard) = row.get("Standard and Date").filter(|v| !v.is_empty())
            && !entry.standards.contains(standard)
        {
            entry.standards.push(standard.clone());
        }
        if let Some(source) = path.file_name().and_then(|v| v.to_str())
            && !entry.sources.contains(&source.to_string())
        {
            entry.sources.push(source.to_string());
        }
        if let Some(synonyms_raw) = row.get("CDISC Synonym(s)").filter(|v| !v.is_empty()) {
            for syn in split_synonyms(synonyms_raw) {
                insert_ct_alias(&mut entry, &submission_value, &syn);
            }
        }

        by_code.insert(codelist_code.clone(), entry);
    }

    for entry in by_code.values() {
        let name_key = entry.codelist_name.to_uppercase();
        by_name.insert(name_key, entry.clone());
        if let Some(submission) = submission_by_code.get(&entry.codelist_code) {
            by_submission.insert(submission.to_uppercase(), entry.clone());
        }
    }

    let (publishing_set, version) = parse_ct_metadata_from_filename(path);

    Ok(CtCatalog {
        label,
        version,
        publishing_set,
        by_code,
        by_name,
        by_submission,
    })
}

fn collect_ct_files(ct_dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut seen = BTreeSet::new();
    for dir in ct_dirs {
        let candidates = csv_glob(dir, "CT_")?;
        for path in candidates {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() || !seen.insert(name) {
                continue;
            }
            files.push(path);
        }
    }
    Ok(files)
}

fn ct_label_from_filename(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if let Some((prefix, _)) = stem.split_once("_CT_") {
        format!("{} CT", prefix)
    } else {
        stem.to_string()
    }
}

/// Parse CT metadata from filename pattern like `SDTM_CT_2024-03-29.csv`.
/// Returns (publishing_set, version) where:
/// - publishing_set: "SDTM", "SEND", "DEFINE-XML", etc.
/// - version: "2024-03-29" (release date)
fn parse_ct_metadata_from_filename(path: &Path) -> (Option<String>, Option<String>) {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("");

    // Pattern: PREFIX_CT_YYYY-MM-DD (e.g., SDTM_CT_2024-03-29)
    if let Some((prefix, date_part)) = stem.split_once("_CT_") {
        let publishing_set = match prefix.to_uppercase().as_str() {
            "SDTM" => Some("SDTM".to_string()),
            "SEND" => Some("SEND".to_string()),
            "ADAM" => Some("ADaM".to_string()),
            "DEFINE-XML" | "DEFINEXML" => Some("DEFINE-XML".to_string()),
            "PROTOCOL" => Some("Protocol".to_string()),
            "DDF" => Some("DDF".to_string()),
            "MRCT" => Some("MRCT".to_string()),
            other => Some(other.to_string()),
        };
        // Version is the date part (e.g., "2024-03-29")
        let version = if !date_part.is_empty() {
            Some(date_part.to_string())
        } else {
            None
        };
        return (publishing_set, version);
    }

    (None, None)
}

fn split_synonyms(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let mut values = Vec::new();
    for sep in [';', ','] {
        if trimmed.contains(sep) {
            for part in trimmed.split(sep) {
                let item = part.trim();
                if !item.is_empty() {
                    values.push(item.to_string());
                }
            }
            return values;
        }
    }
    values.push(trimmed.to_string());
    values
}

fn insert_ct_alias(entry: &mut ControlledTerminology, submission_value: &str, alias: &str) {
    let trimmed = alias.trim();
    if trimmed.is_empty() {
        return;
    }
    if trimmed.eq_ignore_ascii_case(submission_value) {
        return;
    }
    let key = trimmed.to_uppercase();
    entry.synonyms.insert(key, submission_value.to_string());
    let list = entry
        .submission_value_synonyms
        .entry(submission_value.to_string())
        .or_default();
    if !list.iter().any(|value| value.eq_ignore_ascii_case(trimmed)) {
        list.push(trimmed.to_string());
    }
}

// ============================================================================
// Rule Metadata for Missing Dataset Detection
// ============================================================================

/// Metadata about validation rules that map to SDTMIG requirements.
///
/// Per SDTMIG v3.4 Chapter 4, certain rules apply to specific domains or
/// dataset classes. This structure captures that mapping for use in validation.
#[derive(Debug, Clone)]
pub struct RuleMetadata {
    /// Domain codes this rule applies to (empty = all domains)
    pub applicable_domains: Vec<String>,
    /// Dataset classes this rule applies to
    pub applicable_classes: Vec<DatasetClass>,
    /// Whether this rule detects missing datasets
    pub detects_missing_dataset: bool,
    /// The domain code expected to be present (for missing dataset rules)
    pub expected_domain: Option<String>,
    /// Source chapter in SDTMIG
    pub sdtmig_reference: Option<String>,
}

/// Registry of rule metadata for validation.
#[derive(Debug, Clone, Default)]
pub struct RuleMetadataRegistry {
    /// Rules that detect missing datasets, indexed by expected domain
    missing_dataset_rules: BTreeMap<String, RuleMetadata>,
    /// Rules indexed by applicable domain
    rules_by_domain: BTreeMap<String, Vec<RuleMetadata>>,
}

impl RuleMetadataRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert rule metadata.
    pub fn insert(&mut self, metadata: RuleMetadata) {
        // Index by domain
        for domain in &metadata.applicable_domains {
            self.rules_by_domain
                .entry(domain.to_uppercase())
                .or_default()
                .push(metadata.clone());
        }

        // Index missing dataset rules
        if metadata.detects_missing_dataset
            && let Some(expected) = &metadata.expected_domain
        {
            self.missing_dataset_rules
                .insert(expected.to_uppercase(), metadata);
        }
    }

    /// Get rules applicable to a domain.
    pub fn get_rules_for_domain(&self, domain_code: &str) -> Vec<&RuleMetadata> {
        self.rules_by_domain
            .get(&domain_code.to_uppercase())
            .map(|rules| rules.iter().collect())
            .unwrap_or_default()
    }

    /// Get missing dataset rules for a domain.
    pub fn get_missing_dataset_rule(&self, domain_code: &str) -> Option<&RuleMetadata> {
        self.missing_dataset_rules.get(&domain_code.to_uppercase())
    }

    /// Get all domains that have "missing dataset" rules.
    pub fn domains_with_missing_rules(&self) -> Vec<String> {
        self.missing_dataset_rules.keys().cloned().collect()
    }

    /// Check if a domain has any associated rules.
    pub fn has_rules(&self, domain_code: &str) -> bool {
        self.rules_by_domain
            .contains_key(&domain_code.to_uppercase())
    }

    /// Get the count of rules in the registry.
    pub fn len(&self) -> usize {
        self.rules_by_domain.len() + self.missing_dataset_rules.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.rules_by_domain.is_empty() && self.missing_dataset_rules.is_empty()
    }
}

/// Load default rule metadata registry.
pub fn load_default_rule_metadata() -> Result<RuleMetadataRegistry> {
    // Return empty registry - CT-based validation only
    Ok(RuleMetadataRegistry::new())
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

    /// Get all Study Reference domains (OI, DI).
    /// Per SDTMIG v3.4 Chapter 9.
    pub fn get_study_reference_domains(&self) -> Vec<&Domain> {
        self.get_by_class(DatasetClass::StudyReference)
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

/// Load a domain registry from a directory.
pub fn load_domain_registry(base_dir: &Path) -> Result<DomainRegistry> {
    let domains = load_sdtm_ig_domains(base_dir)?;
    Ok(DomainRegistry::new(domains))
}
