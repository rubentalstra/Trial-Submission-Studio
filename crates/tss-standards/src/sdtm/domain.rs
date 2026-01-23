//! SDTM domain and variable definitions per SDTMIG v3.4.
//!
//! This module provides the core types for representing SDTM domains and variables.
//!
//! # SDTMIG Reference
//!
//! - Chapter 2: Fundamentals of the SDTM
//! - Section 2.1: General Observation Classes
//! - Section 4.1: Variable Naming Conventions

use super::enums::{SdtmDatasetClass, VariableRole};
use crate::traits::{CdiscDomain, CdiscVariable, CoreDesignation, VariableType};
use serde::{Deserialize, Serialize};

/// SDTM variable definition per SDTMIG v3.4.
///
/// Represents a single variable (column) within an SDTM domain dataset.
/// Variables have associated metadata including role, core status, and
/// controlled terminology references.
///
/// # Example
///
/// ```
/// use tss_standards::{SdtmVariable, VariableType, VariableRole, CoreDesignation};
///
/// let var = SdtmVariable {
///     name: "USUBJID".to_string(),
///     label: Some("Unique Subject Identifier".to_string()),
///     data_type: VariableType::Char,
///     length: Some(200),
///     role: Some(VariableRole::Identifier),
///     core: Some(CoreDesignation::Required),
///     codelist_code: None,
///     described_value_domain: None,
///     order: Some(3),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdtmVariable {
    /// Variable name (e.g., "USUBJID", "AEDECOD").
    pub name: String,

    /// Human-readable label (max 40 characters for SAS).
    pub label: Option<String>,

    /// Data type (Char or Num).
    pub data_type: VariableType,

    /// Maximum length for character variables (in bytes).
    pub length: Option<u32>,

    /// SDTM role: Identifier, Topic, Qualifier, Timing, or Rule.
    pub role: Option<VariableRole>,

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

impl CdiscVariable for SdtmVariable {
    fn name(&self) -> &str {
        &self.name
    }

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn data_type(&self) -> VariableType {
        self.data_type
    }

    fn core(&self) -> Option<CoreDesignation> {
        self.core
    }

    fn codelist_code(&self) -> Option<&str> {
        self.codelist_code.as_deref()
    }

    fn described_value_domain(&self) -> Option<&str> {
        self.described_value_domain.as_deref()
    }

    fn order(&self) -> Option<u32> {
        self.order
    }
}

/// SDTM domain definition per SDTMIG v3.4.
///
/// A domain represents a collection of observations with a common topic
/// (e.g., Adverse Events, Demographics, Lab Results). Each domain has
/// a two-character code and contains multiple variables.
///
/// # Example
///
/// ```
/// use tss_standards::sdtm::{SdtmDomain, SdtmDatasetClass};
///
/// let domain = SdtmDomain {
///     name: "AE".to_string(),
///     label: Some("Adverse Events".to_string()),
///     class: Some(SdtmDatasetClass::Events),
///     structure: Some("One record per adverse event per subject".to_string()),
///     dataset_name: None,
///     variables: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdtmDomain {
    /// Two-character domain name (e.g., "AE", "DM", "LB").
    pub name: String,

    /// Human-readable domain label (e.g., "Adverse Events").
    pub label: Option<String>,

    /// Dataset class (e.g., Events, Findings, Interventions).
    #[serde(default)]
    pub class: Option<SdtmDatasetClass>,

    /// Dataset structure description (e.g., "One record per subject").
    pub structure: Option<String>,

    /// Output dataset name (may differ from code for split domains).
    pub dataset_name: Option<String>,

    /// Variables belonging to this domain.
    pub variables: Vec<SdtmVariable>,
}

impl CdiscDomain for SdtmDomain {
    type Variable = SdtmVariable;

    fn name(&self) -> &str {
        &self.name
    }

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn variables(&self) -> &[SdtmVariable] {
        &self.variables
    }
}

impl SdtmDomain {
    /// Returns the class name as a string.
    ///
    /// Returns the canonical class name from the dataset class enum,
    /// or None if no class is set.
    pub fn class_name(&self) -> Option<&'static str> {
        self.class.map(|c| c.as_str())
    }

    /// Returns true if this domain belongs to a General Observation class.
    pub fn is_general_observation(&self) -> bool {
        self.class
            .map(|c| c.is_general_observation())
            .unwrap_or(false)
    }

    /// Returns the general observation class for this domain.
    pub fn general_observation_class(&self) -> Option<SdtmDatasetClass> {
        self.class.and_then(|c| c.general_observation_class())
    }

    /// Return the variable name that matches a canonical SDTM name (case-insensitive).
    pub fn column_name(&self, canonical: &str) -> Option<&str> {
        self.variables
            .iter()
            .find(|v| v.name.eq_ignore_ascii_case(canonical))
            .map(|v| v.name.as_str())
    }

    /// Order variables by SDTM role per SDTMIG v3.4 Section 2.1.
    ///
    /// Within each role category, variables are ordered by their defined order.
    pub fn variables_by_role(&self) -> Vec<&SdtmVariable> {
        let mut ordered: Vec<&SdtmVariable> = self.variables.iter().collect();
        ordered.sort_by_key(|v| {
            let role_order = v.role.map(|r| r.sort_order()).unwrap_or(99);
            let order = v.order.unwrap_or(999);
            (role_order, order)
        });
        ordered
    }

    /// Infer the sequence variable for this domain using SDTM naming rules.
    ///
    /// Per SDTMIG v3.4 Section 4.1.7, looks for `{DOMAIN}SEQ` first,
    /// then falls back to any variable ending in "SEQ".
    pub fn infer_seq_column(&self) -> Option<&str> {
        let expected = format!("{}SEQ", self.name);
        if let Some(name) = self.column_name(&expected) {
            return Some(name);
        }
        self.variables
            .iter()
            .map(|v| v.name.as_str())
            .filter(|n| n.len() > 3 && n.to_uppercase().ends_with("SEQ"))
            .min_by_key(|n| n.to_uppercase())
    }

    /// Create a SUPP domain for this parent domain.
    ///
    /// Uses the SUPPQUAL template from the loaded standard and customizes
    /// the name and label for this specific parent domain.
    pub fn create_supp_domain(&self, suppqual_template: &SdtmDomain) -> SdtmDomain {
        let mut supp = suppqual_template.clone();
        supp.name = format!("SUPP{}", self.name);
        supp.label = Some(format!(
            "Supplemental Qualifiers for {}",
            self.label.as_deref().unwrap_or(&self.name)
        ));
        supp
    }
}
