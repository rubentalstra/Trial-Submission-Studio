//! ADaM-IG dataset and variable loading.
//!
//! Loads ADaM Implementation Guide v1.3 definitions from CSV files
//! in the `standards/adam/ig/v1.3/` directory.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use cdisc_model::adam::{AdamDataset, AdamDatasetType, AdamVariable, AdamVariableSource};
use cdisc_model::traits::DataType;

use crate::error::{Result, StandardsError};
use crate::paths::adam_ig_path;

/// Load ADaM-IG datasets from the default location.
///
/// Loads from `standards/adam/ig/v1.3/` relative to the standards root.
pub fn load() -> Result<Vec<AdamDataset>> {
    load_from(&adam_ig_path())
}

/// Load ADaM-IG datasets from a custom path.
///
/// # Arguments
///
/// * `base_dir` - Directory containing DataStructures.csv and Variables.csv
pub fn load_from(base_dir: &Path) -> Result<Vec<AdamDataset>> {
    if !base_dir.exists() {
        return Err(StandardsError::DirectoryNotFound {
            path: base_dir.to_path_buf(),
        });
    }

    let datasets_path = base_dir.join("DataStructures.csv");
    let variables_path = base_dir.join("Variables.csv");

    let (datasets, long_to_short) = load_data_structures(&datasets_path)?;
    let variables = load_variables(&variables_path, &long_to_short)?;

    build_datasets(datasets, variables)
}

// =============================================================================
// CSV Row Types
// =============================================================================

/// Row from DataStructures.csv.
#[derive(Debug, Deserialize)]
struct DataStructureCsvRow {
    #[serde(rename = "Data Structure Name")]
    name: String,
    #[serde(rename = "Data Structure Description")]
    description: String,
    #[serde(rename = "Class")]
    class: String,
    #[serde(rename = "CDISC Notes")]
    notes: String,
}

/// Row from Variables.csv.
#[derive(Debug, Deserialize)]
struct VariableCsvRow {
    #[serde(rename = "Data Structure Name")]
    data_structure: String,
    #[serde(rename = "Variable Name")]
    variable_name: String,
    #[serde(rename = "Variable Label")]
    variable_label: String,
    #[serde(rename = "Type")]
    variable_type: String,
    #[serde(rename = "CDISC CT Codelist Code(s)")]
    codelist_code: String,
    #[serde(rename = "Core")]
    core: String,
    #[serde(rename = "CDISC Notes")]
    notes: String,
}

// =============================================================================
// Loading Functions
// =============================================================================

/// Dataset metadata from DataStructures.csv.
struct DatasetMeta {
    dataset_type: AdamDatasetType,
    label: Option<String>,
    structure: Option<String>,
}

/// Load DataStructures.csv into a map of dataset metadata.
/// Also returns a map from long names (description) to short names.
fn load_data_structures(
    path: &Path,
) -> Result<(BTreeMap<String, DatasetMeta>, BTreeMap<String, String>)> {
    if !path.exists() {
        return Err(StandardsError::FileNotFound {
            path: path.to_path_buf(),
        });
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(|e| StandardsError::CsvRead {
            path: path.to_path_buf(),
            source: e,
        })?;

    let mut datasets = BTreeMap::new();
    let mut long_to_short = BTreeMap::new();

    for result in reader.deserialize::<DataStructureCsvRow>() {
        let row = result.map_err(|e| StandardsError::CsvRead {
            path: path.to_path_buf(),
            source: e,
        })?;

        let short_name = row.name.trim().to_uppercase();
        if short_name.is_empty() {
            continue;
        }

        let dataset_type = parse_dataset_type(&row.class);

        // Create mapping from description to short name
        // e.g., "SUBJECT-LEVEL ANALYSIS DATASET STRUCTURE" -> "ADSL"
        let long_name = row.description.trim().to_uppercase();
        if !long_name.is_empty() {
            long_to_short.insert(long_name.clone(), short_name.clone());
            // Also map without "Structure" suffix for Variables.csv compatibility
            // Variables.csv uses "Subject-Level Analysis Dataset" while
            // DataStructures.csv uses "Subject-Level Analysis Dataset Structure"
            if long_name.ends_with(" STRUCTURE") {
                let without_structure = long_name.trim_end_matches(" STRUCTURE").to_string();
                long_to_short.insert(without_structure, short_name.clone());
            }
        }

        datasets.insert(
            short_name,
            DatasetMeta {
                dataset_type,
                label: non_empty(&row.description),
                structure: non_empty(&row.notes),
            },
        );
    }

    Ok((datasets, long_to_short))
}

/// Load Variables.csv into a map of variables grouped by dataset short name.
fn load_variables(
    path: &Path,
    long_to_short: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Vec<AdamVariable>>> {
    if !path.exists() {
        return Err(StandardsError::FileNotFound {
            path: path.to_path_buf(),
        });
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(|e| StandardsError::CsvRead {
            path: path.to_path_buf(),
            source: e,
        })?;

    let mut grouped: BTreeMap<String, Vec<AdamVariable>> = BTreeMap::new();
    let mut order_counter: BTreeMap<String, u32> = BTreeMap::new();

    for result in reader.deserialize::<VariableCsvRow>() {
        let row = result.map_err(|e| StandardsError::CsvRead {
            path: path.to_path_buf(),
            source: e,
        })?;

        let long_name = row.data_structure.trim().to_uppercase();
        let name = row.variable_name.trim().to_string();
        if long_name.is_empty() || name.is_empty() {
            continue;
        }

        // Map long name to short name, or use the long name as-is if not found
        let dataset_name = long_to_short.get(&long_name).cloned().unwrap_or(long_name);

        // Auto-increment order per dataset
        let order = order_counter.entry(dataset_name.clone()).or_insert(0);
        *order += 1;
        let current_order = *order;

        // Parse source from notes (e.g., "DM.STUDYID" -> SdtmSource)
        let source = parse_source(&row.notes);

        let variable = AdamVariable {
            name,
            label: non_empty(&row.variable_label),
            data_type: parse_data_type(&row.variable_type),
            length: None,
            core: non_empty(&row.core).and_then(|v| v.parse().ok()),
            codelist_code: non_empty(&row.codelist_code),
            source,
            order: Some(current_order),
        };

        grouped.entry(dataset_name).or_default().push(variable);
    }

    Ok(grouped)
}

// =============================================================================
// Build Datasets
// =============================================================================

/// Build AdamDataset structs from loaded data.
fn build_datasets(
    datasets: BTreeMap<String, DatasetMeta>,
    mut variables: BTreeMap<String, Vec<AdamVariable>>,
) -> Result<Vec<AdamDataset>> {
    let mut result = Vec::new();

    for (name, vars) in &mut variables {
        let meta = datasets.get(name);

        result.push(AdamDataset {
            name: name.clone(),
            label: meta.and_then(|m| m.label.clone()),
            dataset_type: meta
                .map(|m| m.dataset_type)
                .unwrap_or(AdamDatasetType::Other),
            structure: meta.and_then(|m| m.structure.clone()),
            variables: std::mem::take(vars),
        });
    }

    // Sort datasets alphabetically by name
    result.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(result)
}

// =============================================================================
// Helpers
// =============================================================================

/// Parse dataset type from class string.
fn parse_dataset_type(raw: &str) -> AdamDatasetType {
    let normalized = raw.trim().to_uppercase();
    if normalized.contains("SUBJECT LEVEL") || normalized.contains("ADSL") {
        AdamDatasetType::Adsl
    } else if normalized.contains("BASIC DATA") || normalized.contains("BDS") {
        AdamDatasetType::Bds
    } else if normalized.contains("OCCURRENCE") || normalized.contains("OCCDS") {
        AdamDatasetType::Occds
    } else if normalized.contains("TIME-TO-EVENT") || normalized.contains("TTE") {
        AdamDatasetType::Tte
    } else {
        AdamDatasetType::Other
    }
}

/// Parse data type from string.
fn parse_data_type(raw: &str) -> DataType {
    match raw.trim().to_lowercase().as_str() {
        "num" | "numeric" => DataType::Num,
        _ => DataType::Char,
    }
}

/// Parse source from notes (e.g., "DM.STUDYID" -> Sdtm source).
fn parse_source(notes: &str) -> Option<AdamVariableSource> {
    let trimmed = notes.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check if it looks like an SDTM reference (DOMAIN.VARIABLE)
    if trimmed.contains('.') && trimmed.chars().next()?.is_ascii_uppercase() {
        Some(AdamVariableSource::Sdtm(trimmed.to_string()))
    } else if trimmed.to_lowercase().contains("derived") {
        Some(AdamVariableSource::Derived(trimmed.to_string()))
    } else {
        Some(AdamVariableSource::Assigned)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_adam_ig() {
        let datasets = load().expect("load ADaM-IG");
        assert!(!datasets.is_empty(), "Should load datasets");

        // Check for ADSL structure (mapped from "Subject-Level Analysis Dataset")
        let adsl = datasets.iter().find(|d| d.name == "ADSL");
        assert!(
            adsl.is_some(),
            "ADSL structure should exist, got: {:?}",
            datasets.iter().map(|d| &d.name).collect::<Vec<_>>()
        );

        let adsl = adsl.unwrap();
        assert_eq!(adsl.dataset_type, AdamDatasetType::Adsl);
        assert!(!adsl.variables.is_empty(), "ADSL should have variables");
    }

    #[test]
    fn test_dataset_type_parsing() {
        assert_eq!(
            parse_dataset_type("SUBJECT LEVEL ANALYSIS DATASET"),
            AdamDatasetType::Adsl
        );
        assert_eq!(
            parse_dataset_type("BASIC DATA STRUCTURE"),
            AdamDatasetType::Bds
        );
    }
}
