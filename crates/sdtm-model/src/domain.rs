//! SDTM domain and variable definitions.
//!
//! This module provides the core types for representing SDTM domains, variables,
//! and dataset classes per SDTMIG v3.4.
//!
//! # SDTMIG Reference
//!
//! - Chapter 2: Fundamentals of the SDTM
//! - Section 2.1: General Observation Classes
//! - Section 4.1: Variable Naming Conventions

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Dataset class per SDTMIG v3.4 Chapter 2 (Fundamentals of the SDTM).
/// These are the major observation class categories used to organize domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
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
    /// Study Reference: OI (and DI per SDTMIG-MD)
    StudyReference,
    /// Relationship: RELREC, RELSPEC, RELSUB, SUPPQUAL
    Relationship,
}

impl DatasetClass {
    /// Returns true if this class is a General Observation class (Interventions, Events, Findings).
    /// Per SDTMIG v3.4 Section 2.1, these are the three general observation classes.
    pub fn is_general_observation(&self) -> bool {
        matches!(
            self,
            DatasetClass::Interventions
                | DatasetClass::Events
                | DatasetClass::Findings
                | DatasetClass::FindingsAbout
        )
    }

    /// Returns the normalized general observation class for Findings About -> Findings mapping.
    /// Per SDTMIG v3.4, Findings About is a specialized version of the Findings class.
    pub fn general_observation_class(&self) -> Option<DatasetClass> {
        match self {
            DatasetClass::Interventions => Some(DatasetClass::Interventions),
            DatasetClass::Events => Some(DatasetClass::Events),
            DatasetClass::Findings | DatasetClass::FindingsAbout => Some(DatasetClass::Findings),
            _ => None,
        }
    }

    /// Returns the canonical class name as it appears in SDTMIG documentation.
    pub fn as_str(&self) -> &'static str {
        match self {
            DatasetClass::Interventions => "Interventions",
            DatasetClass::Events => "Events",
            DatasetClass::Findings => "Findings",
            DatasetClass::FindingsAbout => "Findings About",
            DatasetClass::SpecialPurpose => "Special-Purpose",
            DatasetClass::TrialDesign => "Trial Design",
            DatasetClass::StudyReference => "Study Reference",
            DatasetClass::Relationship => "Relationship",
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

    /// Parse a class name string into a DatasetClass.
    /// Handles various formats found in standards files (case-insensitive, with/without hyphens).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_uppercase().replace('-', " ");
        match normalized.as_str() {
            "INTERVENTIONS" => Ok(DatasetClass::Interventions),
            "EVENTS" => Ok(DatasetClass::Events),
            "FINDINGS" => Ok(DatasetClass::Findings),
            "FINDINGS ABOUT" => Ok(DatasetClass::FindingsAbout),
            "SPECIAL PURPOSE" | "SPECIAL-PURPOSE" => Ok(DatasetClass::SpecialPurpose),
            "TRIAL DESIGN" => Ok(DatasetClass::TrialDesign),
            "STUDY REFERENCE" => Ok(DatasetClass::StudyReference),
            "RELATIONSHIP" => Ok(DatasetClass::Relationship),
            _ => Err(format!("Unknown dataset class: {}", s)),
        }
    }
}

/// Variable data type per SDTMIG v3.4.
///
/// SDTM supports two fundamental data types:
/// - `Char` - Character/text data
/// - `Num` - Numeric data (stored as 8-byte IEEE floating point in SAS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
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
    pub role: Option<String>,
    /// Core designation: Req (Required), Exp (Expected), or Perm (Permissible).
    pub core: Option<String>,
    /// NCI codelist code(s) for controlled terminology validation.
    pub codelist_code: Option<String>,
    /// Variable ordering within the domain.
    #[serde(default)]
    pub order: Option<u32>,
}

/// SDTM domain definition per SDTMIG v3.4.
///
/// A domain represents a collection of observations with a common topic
/// (e.g., Adverse Events, Demographics, Lab Results). Each domain has
/// a two-character code and contains multiple variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    /// Two-character domain code (e.g., "AE", "DM", "LB").
    pub code: String,
    /// Domain description.
    pub description: Option<String>,
    /// Raw class name from standards (e.g., "Findings", "Special-Purpose").
    pub class_name: Option<String>,
    /// Parsed dataset class enum.
    #[serde(default)]
    pub dataset_class: Option<DatasetClass>,
    /// Human-readable domain label.
    pub label: Option<String>,
    /// Dataset structure description (e.g., "One record per subject").
    pub structure: Option<String>,
    /// Output dataset name (may differ from code for split domains).
    pub dataset_name: Option<String>,
    /// Variables belonging to this domain.
    pub variables: Vec<Variable>,
}

impl Domain {
    /// Returns true if this domain belongs to a General Observation class.
    pub fn is_general_observation(&self) -> bool {
        self.dataset_class
            .map(|c| c.is_general_observation())
            .unwrap_or(false)
    }

    /// Returns the general observation class for this domain (Findings About -> Findings).
    pub fn general_observation_class(&self) -> Option<DatasetClass> {
        self.dataset_class
            .and_then(|c| c.general_observation_class())
    }

    /// Return the variable name that matches a canonical SDTM name (case-insensitive).
    pub fn column_name(&self, canonical: &str) -> Option<&str> {
        self.variables
            .iter()
            .find(|variable| variable.name.eq_ignore_ascii_case(canonical))
            .map(|variable| variable.name.as_str())
    }

    /// Order variables by SDTM role per SDTMIG v3.4 Chapter 2 (Section 2.1).
    /// Within each role category, variables are ordered by their defined order.
    pub fn variables_by_role(&self) -> Vec<&Variable> {
        let mut ordered: Vec<&Variable> = self.variables.iter().collect();
        ordered.sort_by_key(|variable| variable_sort_key(variable));
        ordered
    }

    /// Infer the sequence variable for this domain using SDTM naming rules.
    /// Per SDTMIG v3.4 Section 4.1.7, domain-prefixed variables use DOMAIN as the prefix.
    pub fn infer_seq_column(&self) -> Option<&str> {
        let expected = format!("{}SEQ", self.code);
        if let Some(name) = self.column_name(&expected) {
            return Some(name);
        }
        find_suffix_column(&self.variables, "SEQ", "SEQ")
            .or_else(|| find_suffix_column(&self.variables, "GRPID", "GRPID"))
    }
}

/// Per SDTMIG v3.4 Chapter 2 (Section 2.1): Identifiers, Topic, Qualifiers, Rule, Timing.
const ROLE_SORT_ORDER: [(&str, u8); 9] = [
    ("IDENTIFIER", 1),
    ("TOPIC", 2),
    ("GROUPING QUALIFIER", 3),
    ("RESULT QUALIFIER", 4),
    ("SYNONYM QUALIFIER", 5),
    ("RECORD QUALIFIER", 6),
    ("VARIABLE QUALIFIER", 7),
    ("RULE", 8),
    ("TIMING", 9),
];

/// Get the sort key for a variable based on SDTM role and order.
/// Uses the variable's order field if present, otherwise uses role order * 1000.
/// This ensures variables are sorted by role first, then by their defined order within each role.
fn variable_sort_key(var: &Variable) -> (u8, u32) {
    let role = role_sort_order(var.role.as_deref());
    let order = var.order.unwrap_or(999);
    (role, order)
}

fn role_sort_order(role: Option<&str>) -> u8 {
    let Some(role) = role else {
        return 99;
    };
    let trimmed = role.trim();
    for (name, order) in ROLE_SORT_ORDER {
        if trimmed.eq_ignore_ascii_case(name) {
            return order;
        }
    }
    99
}

fn find_suffix_column<'a>(
    variables: &'a [Variable],
    suffix: &str,
    exact_exclude: &str,
) -> Option<&'a str> {
    variables
        .iter()
        .map(|var| var.name.as_str())
        .filter(|name| {
            ends_with_case_insensitive(name, suffix) && !name.eq_ignore_ascii_case(exact_exclude)
        })
        .min_by_key(|name| name.to_ascii_uppercase())
}

fn ends_with_case_insensitive(value: &str, suffix: &str) -> bool {
    if value.len() < suffix.len() {
        return false;
    }
    value[value.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

/// Metadata describing an SDTM dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    /// The dataset name (e.g., "AE", "DM", "SUPPAE").
    pub dataset_name: String,
    /// The raw class name from standards.
    pub class_name: Option<String>,
    /// The parsed dataset class enum.
    pub dataset_class: Option<DatasetClass>,
    /// Human-readable label for the dataset.
    pub label: Option<String>,
    /// Dataset structure type (e.g., "One record per subject").
    pub structure: Option<String>,
}
