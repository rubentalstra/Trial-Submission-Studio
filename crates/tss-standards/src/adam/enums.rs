//! ADaM-specific enumerations.
//!
//! This module contains enumerations specific to ADaM such as dataset types
//! and variable sources.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// ADaM dataset type per ADaMIG v1.3.
///
/// ADaM defines several standard dataset structures:
/// - **ADSL**: Subject-Level Analysis Dataset (one record per subject)
/// - **BDS**: Basic Data Structure (multiple records per subject/parameter)
/// - **OCCDS**: Occurrence Data Structure (for occurrence-type data like AEs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AdamDatasetType {
    /// Subject-Level Analysis Dataset.
    /// Contains one record per subject with all subject-level analysis variables.
    Adsl,
    /// Basic Data Structure.
    /// One or more records per subject, per analysis parameter, per analysis timepoint.
    Bds,
    /// Occurrence Data Structure.
    /// For occurrence-type data such as adverse events, concomitant medications.
    Occds,
    /// Time-to-Event (subclass of BDS).
    /// Specialized structure for survival/time-to-event analyses.
    Tte,
    /// Other/custom dataset structure.
    Other,
}

impl AdamDatasetType {
    /// Returns the dataset type name as it appears in ADaMIG.
    pub fn as_str(&self) -> &'static str {
        match self {
            AdamDatasetType::Adsl => "ADSL",
            AdamDatasetType::Bds => "BDS",
            AdamDatasetType::Occds => "OCCDS",
            AdamDatasetType::Tte => "TTE",
            AdamDatasetType::Other => "Other",
        }
    }

    /// Returns a description of the dataset type.
    pub fn description(&self) -> &'static str {
        match self {
            AdamDatasetType::Adsl => "Subject-Level Analysis Dataset",
            AdamDatasetType::Bds => "Basic Data Structure",
            AdamDatasetType::Occds => "Occurrence Data Structure",
            AdamDatasetType::Tte => "Time-to-Event Analysis",
            AdamDatasetType::Other => "Other Dataset Structure",
        }
    }
}

impl fmt::Display for AdamDatasetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AdamDatasetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_uppercase();
        match normalized.as_str() {
            "ADSL" | "SUBJECT LEVEL ANALYSIS DATASET" => Ok(AdamDatasetType::Adsl),
            "BDS" | "BASIC DATA STRUCTURE" => Ok(AdamDatasetType::Bds),
            "OCCDS" | "OCCURRENCE DATA STRUCTURE" => Ok(AdamDatasetType::Occds),
            "TTE" | "TIME-TO-EVENT" | "TIME TO EVENT" => Ok(AdamDatasetType::Tte),
            _ => Ok(AdamDatasetType::Other),
        }
    }
}

/// ADaM variable source/derivation type.
///
/// Indicates how an ADaM variable value is obtained.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdamVariableSource {
    /// Directly copied from SDTM variable.
    /// Contains the SDTM variable name.
    Sdtm(String),
    /// Derived from other variables using a derivation rule.
    /// Contains the derivation description.
    Derived(String),
    /// Assigned by the sponsor (e.g., analysis flags).
    Assigned,
    /// Predecessor variable (from prior analysis step).
    Predecessor(String),
}

impl AdamVariableSource {
    /// Returns a brief description of the source type.
    pub fn as_str(&self) -> &'static str {
        match self {
            AdamVariableSource::Sdtm(_) => "SDTM",
            AdamVariableSource::Derived(_) => "Derived",
            AdamVariableSource::Assigned => "Assigned",
            AdamVariableSource::Predecessor(_) => "Predecessor",
        }
    }

    /// Returns true if this variable is derived.
    pub fn is_derived(&self) -> bool {
        matches!(self, AdamVariableSource::Derived(_))
    }

    /// Returns true if this variable comes from SDTM.
    pub fn is_from_sdtm(&self) -> bool {
        matches!(self, AdamVariableSource::Sdtm(_))
    }
}

impl fmt::Display for AdamVariableSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdamVariableSource::Sdtm(var) => write!(f, "SDTM.{}", var),
            AdamVariableSource::Derived(desc) => write!(f, "Derived: {}", desc),
            AdamVariableSource::Assigned => write!(f, "Assigned"),
            AdamVariableSource::Predecessor(var) => write!(f, "Predecessor: {}", var),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_type_from_str() {
        assert_eq!(
            "ADSL".parse::<AdamDatasetType>().unwrap(),
            AdamDatasetType::Adsl
        );
        assert_eq!(
            "Basic Data Structure".parse::<AdamDatasetType>().unwrap(),
            AdamDatasetType::Bds
        );
    }
}
