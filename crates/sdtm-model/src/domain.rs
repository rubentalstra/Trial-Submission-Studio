use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Dataset class per SDTMIG v3.4 Chapter 2 (Fundamentals of the SDTM).
/// These are the major observation class categories used to organize domains.
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

    /// Returns true if this class is a Trial Design class.
    pub fn is_trial_design(&self) -> bool {
        matches!(self, DatasetClass::TrialDesign)
    }

    /// Returns true if this class is a Special-Purpose class.
    pub fn is_special_purpose(&self) -> bool {
        matches!(self, DatasetClass::SpecialPurpose)
    }

    /// Returns true if this class is a Relationship class.
    pub fn is_relationship(&self) -> bool {
        matches!(self, DatasetClass::Relationship)
    }

    /// Returns true if this class is a Study Reference class.
    pub fn is_study_reference(&self) -> bool {
        matches!(self, DatasetClass::StudyReference)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    Char,
    Num,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub label: Option<String>,
    pub data_type: VariableType,
    pub length: Option<u32>,
    pub role: Option<String>,
    pub core: Option<String>,
    pub codelist_code: Option<String>,
    #[serde(default)]
    pub order: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub code: String,
    pub description: Option<String>,
    /// The raw class name from standards (e.g., "Findings", "Special-Purpose")
    pub class_name: Option<String>,
    /// The parsed dataset class enum
    #[serde(default)]
    pub dataset_class: Option<DatasetClass>,
    pub label: Option<String>,
    pub structure: Option<String>,
    pub dataset_name: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub dataset_name: String,
    /// The raw class name from standards
    pub class_name: Option<String>,
    /// The parsed dataset class enum
    pub dataset_class: Option<DatasetClass>,
    pub label: Option<String>,
    pub structure: Option<String>,
}
