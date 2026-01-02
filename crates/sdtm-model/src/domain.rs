//! SDTM domain and variable definitions per SDTMIG v3.4.
//!
//! This module provides the core types for representing SDTM domains and variables.
//!
//! # SDTMIG Reference
//!
//! - Chapter 2: Fundamentals of the SDTM
//! - Section 2.1: General Observation Classes
//! - Section 4.1: Variable Naming Conventions

use crate::enums::{CoreDesignation, VariableRole};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Dataset class per SDTMIG v3.4 Chapter 2.
///
/// SDTM organizes domains into observation classes:
/// - **General Observation Classes**: Interventions, Events, Findings
/// - **Special-Purpose**: Demographics, Comments, Subject Elements
/// - **Trial Design**: Study design metadata
/// - **Relationship**: Cross-domain links
///
/// # Example
///
/// ```
/// use sdtm_model::DatasetClass;
///
/// let class: DatasetClass = "Findings".parse().unwrap();
/// assert!(class.is_general_observation());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DatasetClass {
    /// Interventions: AG, CM, EC, EX, ML, PR, SU
    Interventions,
    /// Events: AE, BE, CE, DS, DV, HO, MH
    Events,
    /// Findings: BS, CP, CV, DA, DD, EG, FT, GF, IE, IS, LB, MB, MI, MK, MS, NV, OE, PC, PE, PP, QS, RE, RP, RS, SC, SS, TR, TU, UR, VS
    Findings,
    /// Findings About (subclass of Findings): FA, SR
    FindingsAbout,
    /// Special-Purpose: CO, DM, SE, SM, SV
    SpecialPurpose,
    /// Trial Design: TA, TD, TE, TI, TM, TS, TV
    TrialDesign,
    /// Study Reference: OI
    StudyReference,
    /// Relationship: RELREC, RELSPEC, RELSUB, SUPPQUAL
    Relationship,
}

impl DatasetClass {
    /// Returns true if this is a General Observation class.
    ///
    /// Per SDTMIG v3.4 Section 2.1, the three general observation classes are
    /// Interventions, Events, and Findings (including Findings About).
    pub fn is_general_observation(&self) -> bool {
        matches!(
            self,
            Self::Interventions | Self::Events | Self::Findings | Self::FindingsAbout
        )
    }

    /// Returns the normalized general observation class.
    ///
    /// Maps `FindingsAbout` to `Findings` per SDTMIG v3.4.
    pub fn general_observation_class(&self) -> Option<Self> {
        match self {
            Self::Interventions => Some(Self::Interventions),
            Self::Events => Some(Self::Events),
            Self::Findings | Self::FindingsAbout => Some(Self::Findings),
            _ => None,
        }
    }

    /// Returns the canonical class name as it appears in SDTMIG.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Interventions => "Interventions",
            Self::Events => "Events",
            Self::Findings => "Findings",
            Self::FindingsAbout => "Findings About",
            Self::SpecialPurpose => "Special-Purpose",
            Self::TrialDesign => "Trial Design",
            Self::StudyReference => "Study Reference",
            Self::Relationship => "Relationship",
        }
    }
}

impl fmt::Display for DatasetClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for DatasetClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_uppercase().replace(['-', '_'], " ");
        match normalized.as_str() {
            "INTERVENTIONS" => Ok(Self::Interventions),
            "EVENTS" => Ok(Self::Events),
            "FINDINGS" => Ok(Self::Findings),
            "FINDINGS ABOUT" => Ok(Self::FindingsAbout),
            "SPECIAL PURPOSE" => Ok(Self::SpecialPurpose),
            "TRIAL DESIGN" => Ok(Self::TrialDesign),
            "STUDY REFERENCE" => Ok(Self::StudyReference),
            "RELATIONSHIP" => Ok(Self::Relationship),
            _ => Err(format!("Unknown dataset class: {s}")),
        }
    }
}

/// Variable data type per SDTMIG v3.4.
///
/// SDTM supports two fundamental data types:
/// - `Char`: Character/text data
/// - `Num`: Numeric data (8-byte IEEE float in SAS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    /// Character/text data type.
    Char,
    /// Numeric data type (8-byte floating point).
    Num,
}

/// SDTM variable definition per SDTMIG v3.4.
///
/// Represents a single variable (column) within an SDTM domain dataset.
/// Variables have associated metadata including role, core status, and
/// controlled terminology references.
///
/// # Example
///
/// ```
/// use sdtm_model::{Variable, VariableType, VariableRole, CoreDesignation};
///
/// let var = Variable {
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
pub struct Variable {
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

/// SDTM domain definition per SDTMIG v3.4.
///
/// A domain represents a collection of observations with a common topic
/// (e.g., Adverse Events, Demographics, Lab Results). Each domain has
/// a two-character code and contains multiple variables.
///
/// # Example
///
/// ```
/// use sdtm_model::{Domain, DatasetClass};
///
/// let domain = Domain {
///     name: "AE".to_string(),
///     label: Some("Adverse Events".to_string()),
///     class: Some(DatasetClass::Events),
///     structure: Some("One record per adverse event per subject".to_string()),
///     dataset_name: None,
///     variables: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    /// Two-character domain name (e.g., "AE", "DM", "LB").
    pub name: String,

    /// Human-readable domain label (e.g., "Adverse Events").
    pub label: Option<String>,

    /// Dataset class (e.g., Events, Findings, Interventions).
    #[serde(default)]
    pub class: Option<DatasetClass>,

    /// Dataset structure description (e.g., "One record per subject").
    pub structure: Option<String>,

    /// Output dataset name (may differ from code for split domains).
    pub dataset_name: Option<String>,

    /// Variables belonging to this domain.
    pub variables: Vec<Variable>,
}

impl Domain {
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
    pub fn general_observation_class(&self) -> Option<DatasetClass> {
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
    pub fn variables_by_role(&self) -> Vec<&Variable> {
        let mut ordered: Vec<&Variable> = self.variables.iter().collect();
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
}
