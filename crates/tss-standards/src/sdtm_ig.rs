//! SDTM-IG domain and variable loading.
//!
//! Loads SDTM Implementation Guide v3.4 definitions from embedded CSV data.
//! All data is compiled into the binary for offline operation.

use std::collections::BTreeMap;
use std::io::Cursor;

use serde::Deserialize;

use crate::embedded;
use crate::error::{Result, StandardsError};
use crate::sdtm::{SdtmDatasetClass, SdtmDomain, SdtmVariable};
use crate::traits::VariableType;

/// Load SDTM-IG domains from embedded data.
///
/// # Example
///
/// ```rust,ignore
/// let domains = tss_standards::sdtm_ig::load()?;
/// let ae = domains.iter().find(|d| d.name == "AE").unwrap();
/// println!("AE has {} variables", ae.variables.len());
/// ```
pub fn load() -> Result<Vec<SdtmDomain>> {
    let datasets = load_datasets_from_str(embedded::SDTM_IG_DATASETS)?;
    let variables = load_variables_from_str(embedded::SDTM_IG_VARIABLES)?;
    build_domains(&datasets, variables)
}

// =============================================================================
// CSV Row Types
// =============================================================================

/// Row from Datasets.csv.
#[derive(Debug, Deserialize)]
struct DatasetCsvRow {
    #[serde(rename = "Class")]
    class: String,
    #[serde(rename = "Dataset Name")]
    dataset_name: String,
    #[serde(rename = "Dataset Label")]
    dataset_label: String,
    #[serde(rename = "Structure")]
    structure: String,
}

/// Row from Variables.csv.
#[derive(Debug, Deserialize)]
struct VariableCsvRow {
    #[serde(rename = "Variable Order")]
    variable_order: String,
    #[serde(rename = "Dataset Name")]
    dataset_name: String,
    #[serde(rename = "Variable Name")]
    variable_name: String,
    #[serde(rename = "Variable Label")]
    variable_label: String,
    #[serde(rename = "Type")]
    variable_type: String,
    #[serde(rename = "CDISC CT Codelist Code(s)")]
    codelist_code: String,
    #[serde(rename = "Described Value Domain(s)")]
    described_value_domain: String,
    #[serde(rename = "Role")]
    role: String,
    #[serde(rename = "Core")]
    core: String,
}

// =============================================================================
// Loading Functions
// =============================================================================

/// Load Datasets.csv from embedded string content.
fn load_datasets_from_str(content: &str) -> Result<BTreeMap<String, DatasetMeta>> {
    let cursor = Cursor::new(content.as_bytes());
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(cursor);

    let mut datasets = BTreeMap::new();

    for result in reader.deserialize::<DatasetCsvRow>() {
        let row = result.map_err(|e| StandardsError::CsvParse {
            file: "SDTM-IG Datasets.csv".to_string(),
            message: e.to_string(),
        })?;

        let name = row.dataset_name.trim().to_uppercase();
        if name.is_empty() {
            continue;
        }

        let class = non_empty(&row.class).and_then(|c| c.parse().ok());

        datasets.insert(
            name,
            DatasetMeta {
                class,
                label: non_empty(&row.dataset_label),
                structure: non_empty(&row.structure),
            },
        );
    }

    Ok(datasets)
}

/// Load Variables.csv from embedded string content.
fn load_variables_from_str(content: &str) -> Result<BTreeMap<String, Vec<SdtmVariable>>> {
    let cursor = Cursor::new(content.as_bytes());
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(cursor);

    let mut grouped: BTreeMap<String, Vec<SdtmVariable>> = BTreeMap::new();

    for result in reader.deserialize::<VariableCsvRow>() {
        let row = result.map_err(|e| StandardsError::CsvParse {
            file: "SDTM-IG Variables.csv".to_string(),
            message: e.to_string(),
        })?;

        let dataset = row.dataset_name.trim().to_uppercase();
        let name = row.variable_name.trim().to_string();
        if dataset.is_empty() || name.is_empty() {
            continue;
        }

        let order = row.variable_order.trim().parse::<u32>().ok();

        let variable = SdtmVariable {
            name,
            label: non_empty(&row.variable_label),
            data_type: parse_variable_type(&row.variable_type),
            length: None,
            role: non_empty(&row.role).and_then(|v| v.parse().ok()),
            core: non_empty(&row.core).and_then(|v| v.parse().ok()),
            codelist_code: non_empty(&row.codelist_code),
            described_value_domain: non_empty(&row.described_value_domain),
            order,
        };

        grouped.entry(dataset).or_default().push(variable);
    }

    Ok(grouped)
}

// =============================================================================
// Build Domains
// =============================================================================

/// Dataset metadata from Datasets.csv.
struct DatasetMeta {
    class: Option<SdtmDatasetClass>,
    label: Option<String>,
    structure: Option<String>,
}

/// Build Domain structs from loaded data.
fn build_domains(
    datasets: &BTreeMap<String, DatasetMeta>,
    mut variables: BTreeMap<String, Vec<SdtmVariable>>,
) -> Result<Vec<SdtmDomain>> {
    let mut domains = Vec::new();

    for (name, vars) in &mut variables {
        let meta = datasets.get(name);

        // Sort variables by order
        vars.sort_by(compare_variable_order);

        domains.push(SdtmDomain {
            name: name.clone(),
            label: meta.and_then(|m| m.label.clone()),
            class: meta.and_then(|m| m.class),
            structure: meta.and_then(|m| m.structure.clone()),
            dataset_name: Some(name.clone()),
            variables: std::mem::take(vars),
        });
    }

    // Sort domains alphabetically by name
    domains.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(domains)
}

// =============================================================================
// Helpers
// =============================================================================

/// Parse variable type from string.
fn parse_variable_type(raw: &str) -> VariableType {
    match raw.trim().to_lowercase().as_str() {
        "num" | "numeric" => VariableType::Num,
        _ => VariableType::Char,
    }
}

/// Return Some(value) if non-empty, None otherwise.
fn non_empty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Compare variables by order (nulls last, then by name).
fn compare_variable_order(left: &SdtmVariable, right: &SdtmVariable) -> std::cmp::Ordering {
    match (left.order, right.order) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.name.to_uppercase().cmp(&right.name.to_uppercase()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_sdtm_ig() {
        let domains = load().expect("load SDTM-IG");
        assert!(!domains.is_empty(), "Should load domains");

        // Check for AE domain
        let ae = domains.iter().find(|d| d.name == "AE");
        assert!(ae.is_some(), "AE domain should exist");

        let ae = ae.unwrap();
        assert!(!ae.variables.is_empty(), "AE should have variables");

        // STUDYID should be first (order 1)
        let studyid = ae.variables.iter().find(|v| v.name == "STUDYID");
        assert!(studyid.is_some(), "STUDYID should exist");
    }

    #[test]
    fn test_domain_count() {
        let domains = load().expect("load SDTM-IG");
        // SDTM-IG v3.4 has 60+ domains
        assert!(
            domains.len() >= 60,
            "Expected at least 60 domains, got {}",
            domains.len()
        );
    }

    #[test]
    fn test_variable_order() {
        let domains = load().expect("load SDTM-IG");
        let dm = domains.iter().find(|d| d.name == "DM").expect("DM domain");

        // Variables should be in order
        for window in dm.variables.windows(2) {
            if let (Some(a), Some(b)) = (window[0].order, window[1].order) {
                assert!(a <= b, "Variables should be ordered: {} vs {}", a, b);
            }
        }
    }
}
