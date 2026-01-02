//! SEND domain and variable definitions per SENDIG v3.1.
//!
//! This module provides types for representing SEND nonclinical study domains.

use super::enums::{SendDatasetClass, SendStudyType};
use crate::traits::{CoreDesignation, DataType};
use serde::{Deserialize, Serialize};

/// SEND variable definition per SENDIG v3.1.
///
/// Represents a single variable (column) within a SEND domain dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendVariable {
    /// Variable name (e.g., "USUBJID", "BWSTRESN").
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

    /// Described value domain (e.g., "ISO 8601 datetime").
    #[serde(default)]
    pub described_value_domain: Option<String>,

    /// Variable ordering within the domain.
    #[serde(default)]
    pub order: Option<u32>,
}

/// SEND domain definition per SENDIG v3.1.
///
/// A domain represents a collection of nonclinical observations.
/// SEND uses similar structure to SDTM but with animal-study-specific domains.
///
/// # Example
///
/// ```
/// use cdisc_model::send::{SendDomain, SendDatasetClass};
///
/// let domain = SendDomain {
///     name: "BW".to_string(),
///     label: Some("Body Weight".to_string()),
///     class: Some(SendDatasetClass::Findings),
///     structure: Some("One record per test per observation time per subject".to_string()),
///     study_type: None,
///     variables: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendDomain {
    /// Two-character domain name (e.g., "BW", "DM", "LB").
    pub name: String,

    /// Human-readable domain label (e.g., "Body Weight").
    pub label: Option<String>,

    /// Dataset class (e.g., Events, Findings, Interventions).
    #[serde(default)]
    pub class: Option<SendDatasetClass>,

    /// Dataset structure description (e.g., "One record per subject").
    pub structure: Option<String>,

    /// Study type this domain is commonly used in.
    #[serde(default)]
    pub study_type: Option<SendStudyType>,

    /// Variables belonging to this domain.
    pub variables: Vec<SendVariable>,
}

impl SendDomain {
    /// Returns the class name as a string.
    pub fn class_name(&self) -> Option<&'static str> {
        self.class.map(|c| c.as_str())
    }

    /// Find a variable by name (case-insensitive).
    pub fn find_variable(&self, name: &str) -> Option<&SendVariable> {
        self.variables
            .iter()
            .find(|v| v.name.eq_ignore_ascii_case(name))
    }

    /// Infer the sequence variable for this domain.
    pub fn infer_seq_column(&self) -> Option<&str> {
        let expected = format!("{}SEQ", self.name);
        self.variables
            .iter()
            .find(|v| v.name.eq_ignore_ascii_case(&expected))
            .map(|v| v.name.as_str())
            .or_else(|| {
                self.variables
                    .iter()
                    .map(|v| v.name.as_str())
                    .filter(|n| n.len() > 3 && n.to_uppercase().ends_with("SEQ"))
                    .min_by_key(|n| n.to_uppercase())
            })
    }
}
