//! SEND-specific enumerations.
//!
//! This module contains enumerations specific to SEND such as dataset classes
//! and study types.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// SEND dataset class per SENDIG v3.1.1.
///
/// Similar to SDTM but adapted for nonclinical studies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SendDatasetClass {
    /// Interventions: EX (Exposure)
    Interventions,
    /// Events: DS (Disposition), CL (Clinical Signs)
    Events,
    /// Findings: BW, BG, CL, CV, DD, EG, FW, LB, MA, MI, OM, PC, PP, TF, VS
    Findings,
    /// Special-Purpose: CO, DM, SE, TX (pooled/hierarchical subjects)
    SpecialPurpose,
    /// Trial Design: TA, TE, TS, TX
    TrialDesign,
    /// Relationship: RELREC
    Relationship,
}

impl SendDatasetClass {
    /// Returns the canonical class name as it appears in SENDIG.
    pub fn as_str(&self) -> &'static str {
        match self {
            SendDatasetClass::Interventions => "Interventions",
            SendDatasetClass::Events => "Events",
            SendDatasetClass::Findings => "Findings",
            SendDatasetClass::SpecialPurpose => "Special-Purpose",
            SendDatasetClass::TrialDesign => "Trial Design",
            SendDatasetClass::Relationship => "Relationship",
        }
    }
}

impl fmt::Display for SendDatasetClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SendDatasetClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_uppercase().replace(['-', '_'], " ");
        match normalized.as_str() {
            "INTERVENTIONS" => Ok(SendDatasetClass::Interventions),
            "EVENTS" => Ok(SendDatasetClass::Events),
            "FINDINGS" => Ok(SendDatasetClass::Findings),
            "SPECIAL PURPOSE" => Ok(SendDatasetClass::SpecialPurpose),
            "TRIAL DESIGN" => Ok(SendDatasetClass::TrialDesign),
            "RELATIONSHIP" => Ok(SendDatasetClass::Relationship),
            _ => Err(format!("Unknown SEND dataset class: {s}")),
        }
    }
}

/// SEND study type.
///
/// Different types of nonclinical studies have different domain requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SendStudyType {
    /// Single-dose toxicology study.
    SingleDoseToxicology,
    /// Repeat-dose (general) toxicology study.
    RepeatDoseToxicology,
    /// Carcinogenicity study.
    Carcinogenicity,
    /// Safety pharmacology (respiratory/cardiovascular).
    SafetyPharmacology,
    /// Developmental and reproductive toxicology (DART).
    ReproductiveToxicology,
    /// Genetic toxicology.
    GeneticToxicology,
    /// Studies under the Animal Rule.
    AnimalRule,
    /// Other/unspecified study type.
    Other,
}

impl SendStudyType {
    /// Returns the study type name.
    pub fn as_str(&self) -> &'static str {
        match self {
            SendStudyType::SingleDoseToxicology => "Single-Dose Toxicology",
            SendStudyType::RepeatDoseToxicology => "Repeat-Dose Toxicology",
            SendStudyType::Carcinogenicity => "Carcinogenicity",
            SendStudyType::SafetyPharmacology => "Safety Pharmacology",
            SendStudyType::ReproductiveToxicology => "Reproductive Toxicology",
            SendStudyType::GeneticToxicology => "Genetic Toxicology",
            SendStudyType::AnimalRule => "Animal Rule",
            SendStudyType::Other => "Other",
        }
    }

    /// Returns a description of the study type.
    pub fn description(&self) -> &'static str {
        match self {
            SendStudyType::SingleDoseToxicology => "Single-dose general toxicology studies",
            SendStudyType::RepeatDoseToxicology => "Repeat-dose general toxicology studies",
            SendStudyType::Carcinogenicity => "Long-term carcinogenicity studies",
            SendStudyType::SafetyPharmacology => {
                "Respiratory and cardiovascular safety pharmacology"
            }
            SendStudyType::ReproductiveToxicology => {
                "Embryo-fetal development and juvenile animal studies"
            }
            SendStudyType::GeneticToxicology => "In vivo genetic toxicology studies",
            SendStudyType::AnimalRule => "Studies conducted under the Animal Rule",
            SendStudyType::Other => "Other nonclinical study type",
        }
    }
}

impl fmt::Display for SendStudyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SendStudyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_uppercase();
        match normalized.as_str() {
            "SINGLE-DOSE TOXICOLOGY" | "SINGLE DOSE" => Ok(SendStudyType::SingleDoseToxicology),
            "REPEAT-DOSE TOXICOLOGY" | "REPEAT DOSE" | "GENERAL TOXICOLOGY" => {
                Ok(SendStudyType::RepeatDoseToxicology)
            }
            "CARCINOGENICITY" => Ok(SendStudyType::Carcinogenicity),
            "SAFETY PHARMACOLOGY" => Ok(SendStudyType::SafetyPharmacology),
            "REPRODUCTIVE TOXICOLOGY" | "DART" => Ok(SendStudyType::ReproductiveToxicology),
            "GENETIC TOXICOLOGY" | "GENETOX" => Ok(SendStudyType::GeneticToxicology),
            "ANIMAL RULE" => Ok(SendStudyType::AnimalRule),
            _ => Ok(SendStudyType::Other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_class_from_str() {
        assert_eq!(
            "Findings".parse::<SendDatasetClass>().unwrap(),
            SendDatasetClass::Findings
        );
        assert_eq!(
            "Special-Purpose".parse::<SendDatasetClass>().unwrap(),
            SendDatasetClass::SpecialPurpose
        );
    }

    #[test]
    fn test_study_type_from_str() {
        assert_eq!(
            "Carcinogenicity".parse::<SendStudyType>().unwrap(),
            SendStudyType::Carcinogenicity
        );
    }
}
