//! ADaM dataset and variable definitions per ADaMIG v1.3.
//!
//! This module provides types for representing ADaM analysis datasets.

use super::enums::{AdamDatasetType, AdamVariableSource};
use crate::traits::{CoreDesignation, DataType};
use serde::{Deserialize, Serialize};

/// ADaM variable definition per ADaMIG v1.3.
///
/// Represents a single variable (column) within an ADaM analysis dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdamVariable {
    /// Variable name (e.g., "USUBJID", "AVAL", "PARAMCD").
    pub name: String,

    /// Human-readable label (max 40 characters for SAS).
    pub label: Option<String>,

    /// Data type (Char or Num).
    pub data_type: DataType,

    /// Maximum length for character variables (in bytes).
    pub length: Option<u32>,

    /// Core designation: Required, Expected, or Permissible.
    pub core: Option<CoreDesignation>,

    /// NCI codelist code(s) for controlled terminology validation.
    pub codelist_code: Option<String>,

    /// Variable source/derivation information.
    pub source: Option<AdamVariableSource>,

    /// Variable ordering within the dataset.
    #[serde(default)]
    pub order: Option<u32>,
}

/// ADaM dataset definition per ADaMIG v1.3.
///
/// A dataset represents an analysis-ready collection of data derived from SDTM.
/// ADaM datasets support traceability back to source SDTM data.
///
/// # Example
///
/// ```
/// use tss_model::adam::{AdamDataset, AdamDatasetType};
///
/// let dataset = AdamDataset {
///     name: "ADSL".to_string(),
///     label: Some("Subject Level Analysis Dataset".to_string()),
///     dataset_type: AdamDatasetType::Adsl,
///     structure: Some("One record per subject".to_string()),
///     variables: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdamDataset {
    /// Dataset name (e.g., "ADSL", "ADAE", "ADLB").
    pub name: String,

    /// Human-readable dataset label.
    pub label: Option<String>,

    /// ADaM dataset type (ADSL, BDS, OCCDS, etc.).
    pub dataset_type: AdamDatasetType,

    /// Dataset structure description.
    pub structure: Option<String>,

    /// Variables belonging to this dataset.
    pub variables: Vec<AdamVariable>,
}

impl AdamDataset {
    /// Returns true if this is the subject-level dataset (ADSL).
    pub fn is_adsl(&self) -> bool {
        self.dataset_type == AdamDatasetType::Adsl
    }

    /// Returns true if this is a BDS (Basic Data Structure) dataset.
    pub fn is_bds(&self) -> bool {
        matches!(
            self.dataset_type,
            AdamDatasetType::Bds | AdamDatasetType::Tte
        )
    }

    /// Find a variable by name (case-insensitive).
    pub fn find_variable(&self, name: &str) -> Option<&AdamVariable> {
        self.variables
            .iter()
            .find(|v| v.name.eq_ignore_ascii_case(name))
    }

    /// Returns all derived variables.
    pub fn derived_variables(&self) -> Vec<&AdamVariable> {
        self.variables
            .iter()
            .filter(|v| v.source.as_ref().map(|s| s.is_derived()).unwrap_or(false))
            .collect()
    }

    /// Returns all variables sourced from SDTM.
    pub fn sdtm_sourced_variables(&self) -> Vec<&AdamVariable> {
        self.variables
            .iter()
            .filter(|v| v.source.as_ref().map(|s| s.is_from_sdtm()).unwrap_or(false))
            .collect()
    }
}
