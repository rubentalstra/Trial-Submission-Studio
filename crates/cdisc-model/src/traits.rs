//! Core traits for CDISC data model abstraction.
//!
//! This module provides a common interface across SDTM, ADaM, and SEND standards.

use serde::{Deserialize, Serialize};

/// CDISC foundational standard identifier.
///
/// Represents the three main foundational standards for regulatory data:
/// - **SDTM**: Clinical trial tabulation (human studies)
/// - **ADaM**: Analysis datasets derived from SDTM
/// - **SEND**: Nonclinical/animal study tabulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Standard {
    /// Study Data Tabulation Model - Clinical trial data
    Sdtm,
    /// Analysis Data Model - Statistical analysis datasets
    Adam,
    /// Standard for Exchange of Nonclinical Data - Animal studies
    Send,
}

impl Standard {
    /// Returns the standard name as it appears in CDISC documentation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Standard::Sdtm => "SDTM",
            Standard::Adam => "ADaM",
            Standard::Send => "SEND",
        }
    }

    /// Returns the full name of the standard.
    pub fn full_name(&self) -> &'static str {
        match self {
            Standard::Sdtm => "Study Data Tabulation Model",
            Standard::Adam => "Analysis Data Model",
            Standard::Send => "Standard for Exchange of Nonclinical Data",
        }
    }

    /// Returns the directory name used in the standards folder.
    pub fn dir_name(&self) -> &'static str {
        match self {
            Standard::Sdtm => "sdtm",
            Standard::Adam => "adam",
            Standard::Send => "send",
        }
    }

    /// Returns a brief description of the standard's purpose.
    pub fn description(&self) -> &'static str {
        match self {
            Standard::Sdtm => "Clinical trial tabulation for regulatory submissions",
            Standard::Adam => "Analysis-ready datasets derived from SDTM",
            Standard::Send => "Nonclinical/animal study tabulation",
        }
    }

    /// Returns the regulatory agencies that require this standard.
    pub fn regulatory_agencies(&self) -> &'static [&'static str] {
        match self {
            Standard::Sdtm => &["FDA", "PMDA"],
            Standard::Adam => &["FDA", "PMDA"],
            Standard::Send => &["FDA"],
        }
    }
}

impl std::fmt::Display for Standard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Common variable data type across all CDISC standards.
///
/// All CDISC standards use the same fundamental data types:
/// - `Char`: Character/text data
/// - `Num`: Numeric data (8-byte IEEE float in SAS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// Character/text data type.
    Char,
    /// Numeric data type (8-byte floating point).
    Num,
}

impl DataType {
    /// Returns the type name as it appears in standards files.
    pub fn as_str(&self) -> &'static str {
        match self {
            DataType::Char => "Char",
            DataType::Num => "Num",
        }
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Core designation indicating whether a variable is required, expected, or permissible.
///
/// This is common across SDTM, ADaM, and SEND standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreDesignation {
    /// Required (Req): Must be present in the dataset. Null values not allowed.
    Required,
    /// Expected (Exp): Expected when applicable. Null allowed if not applicable.
    Expected,
    /// Permissible (Perm): Optional. May be included if data is collected.
    Permissible,
}

impl CoreDesignation {
    /// Returns the short code as it appears in standards files.
    pub fn as_code(&self) -> &'static str {
        match self {
            CoreDesignation::Required => "Req",
            CoreDesignation::Expected => "Exp",
            CoreDesignation::Permissible => "Perm",
        }
    }

    /// Returns the full name.
    pub fn as_str(&self) -> &'static str {
        match self {
            CoreDesignation::Required => "Required",
            CoreDesignation::Expected => "Expected",
            CoreDesignation::Permissible => "Permissible",
        }
    }

    /// Returns true if this variable is required (must be present, non-null).
    pub fn is_required(&self) -> bool {
        matches!(self, CoreDesignation::Required)
    }
}

impl std::fmt::Display for CoreDesignation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for CoreDesignation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_uppercase();
        match normalized.as_str() {
            "REQ" | "REQUIRED" => Ok(CoreDesignation::Required),
            "EXP" | "EXPECTED" => Ok(CoreDesignation::Expected),
            "PERM" | "PERMISSIBLE" => Ok(CoreDesignation::Permissible),
            _ => Err(format!("Unknown core designation: {s}")),
        }
    }
}
