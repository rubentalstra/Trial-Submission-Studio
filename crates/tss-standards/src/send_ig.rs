//! SEND-IG domain and variable loading.
//!
//! Loads SEND Implementation Guide v3.1.1 definitions from embedded CSV data.
//! All data is compiled into the binary for offline operation.

use std::collections::BTreeMap;
use std::io::Cursor;

use serde::Deserialize;

use crate::embedded;
use crate::error::{Result, StandardsError};
use crate::send::{SendDatasetClass, SendDomain, SendVariable};
use crate::traits::VariableType;

/// Load SEND-IG domains from embedded data.
pub fn load() -> Result<Vec<SendDomain>> {
    let datasets = load_datasets_from_str(embedded::SEND_IG_DATASETS)?;
    let variables = load_variables_from_str(embedded::SEND_IG_VARIABLES)?;
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
    #[serde(rename = "Core")]
    core: String,
}

// =============================================================================
// Loading Functions
// =============================================================================

/// Domain metadata from Datasets.csv.
struct DatasetMeta {
    class: Option<SendDatasetClass>,
    label: Option<String>,
    structure: Option<String>,
}

/// Load Datasets.csv from embedded string content.
fn load_datasets_from_str(content: &str) -> Result<BTreeMap<String, DatasetMeta>> {
    let cursor = Cursor::new(content.as_bytes());
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(cursor);

    let mut datasets = BTreeMap::new();

    for result in reader.deserialize::<DatasetCsvRow>() {
        let row = result.map_err(|e| StandardsError::CsvParse {
            file: "SEND-IG Datasets.csv".to_string(),
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
fn load_variables_from_str(content: &str) -> Result<BTreeMap<String, Vec<SendVariable>>> {
    let cursor = Cursor::new(content.as_bytes());
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(cursor);

    let mut grouped: BTreeMap<String, Vec<SendVariable>> = BTreeMap::new();

    for result in reader.deserialize::<VariableCsvRow>() {
        let row = result.map_err(|e| StandardsError::CsvParse {
            file: "SEND-IG Variables.csv".to_string(),
            message: e.to_string(),
        })?;

        let dataset = row.dataset_name.trim().to_uppercase();
        let name = row.variable_name.trim().to_string();
        if dataset.is_empty() || name.is_empty() {
            continue;
        }

        let order = row.variable_order.trim().parse::<u32>().ok();

        let variable = SendVariable {
            name,
            label: non_empty(&row.variable_label),
            data_type: parse_variable_type(&row.variable_type),
            length: None,
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

/// Build SendDomain structs from loaded data.
fn build_domains(
    datasets: &BTreeMap<String, DatasetMeta>,
    mut variables: BTreeMap<String, Vec<SendVariable>>,
) -> Result<Vec<SendDomain>> {
    let mut domains = Vec::new();

    for (name, vars) in &mut variables {
        let meta = datasets.get(name);

        // Sort variables by order
        vars.sort_by(compare_variable_order);

        domains.push(SendDomain {
            name: name.clone(),
            label: meta.and_then(|m| m.label.clone()),
            class: meta.and_then(|m| m.class),
            structure: meta.and_then(|m| m.structure.clone()),
            study_type: None, // Could be inferred from context
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
fn compare_variable_order(left: &SendVariable, right: &SendVariable) -> std::cmp::Ordering {
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
    fn test_load_send_ig() {
        let domains = load().expect("load SEND-IG");
        assert!(!domains.is_empty(), "Should load domains");

        // Check for EX domain (Exposure)
        let ex = domains.iter().find(|d| d.name == "EX");
        assert!(ex.is_some(), "EX domain should exist");

        let ex = ex.unwrap();
        assert!(!ex.variables.is_empty(), "EX should have variables");
    }

    #[test]
    fn test_domain_count() {
        let domains = load().expect("load SEND-IG");
        // SEND-IG v3.1.1 has multiple domains
        assert!(
            domains.len() >= 10,
            "Expected at least 10 domains, got {}",
            domains.len()
        );
    }
}
