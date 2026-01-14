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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VariableType {
    /// Character/text data type.
    Char,
    /// Numeric data type (8-byte floating point).
    Num,
}

impl VariableType {
    /// Returns the type name as it appears in standards files.
    pub fn as_str(&self) -> &'static str {
        match self {
            VariableType::Char => "Char",
            VariableType::Num => "Num",
        }
    }
}

impl std::fmt::Display for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for VariableType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_uppercase();
        match normalized.as_str() {
            "CHAR" | "CHARACTER" | "TEXT" => Ok(VariableType::Char),
            "NUM" | "NUMERIC" | "NUMBER" => Ok(VariableType::Num),
            _ => Err(format!("Unknown variable type: {s}")),
        }
    }
}

/// Core designation indicating whether a variable is required, expected, or permissible.
///
/// This is common across SDTM, ADaM, and SEND standards.
///
/// # CDISC Rules
///
/// - **Required (Req)**: Must be present in the dataset. Cannot be marked as "not collected" or "omitted".
/// - **Expected (Exp)**: Should be present when applicable. Can be marked as "not collected" if data was not gathered.
/// - **Permissible (Perm)**: Optional. Can be "omitted" entirely from the dataset.
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

    /// Returns true if this variable can be marked as "not collected".
    ///
    /// Only Expected variables can use this designation.
    pub fn can_be_not_collected(&self) -> bool {
        matches!(self, CoreDesignation::Expected)
    }

    /// Returns true if this variable can be omitted from the dataset.
    ///
    /// Only Permissible variables can be omitted.
    pub fn can_be_omitted(&self) -> bool {
        matches!(self, CoreDesignation::Permissible)
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

/// Common trait for all CDISC domain types (SDTM, ADaM, SEND).
///
/// This trait provides a unified interface for domain operations across all standards.
pub trait CdiscDomain {
    /// The variable type for this domain.
    type Variable: CdiscVariable;

    /// Returns the domain/dataset name (e.g., "AE", "ADSL", "BW").
    fn name(&self) -> &str;

    /// Returns the human-readable label.
    fn label(&self) -> Option<&str>;

    /// Returns the variables in this domain.
    fn variables(&self) -> &[Self::Variable];

    /// Find a variable by name (case-insensitive).
    fn find_variable(&self, name: &str) -> Option<&Self::Variable> {
        self.variables()
            .iter()
            .find(|v| v.name().eq_ignore_ascii_case(name))
    }

    /// Returns required variables (core = Req).
    fn required_variables(&self) -> Vec<&Self::Variable> {
        self.variables()
            .iter()
            .filter(|v| v.core() == Some(CoreDesignation::Required))
            .collect()
    }

    /// Returns expected variables (core = Exp).
    fn expected_variables(&self) -> Vec<&Self::Variable> {
        self.variables()
            .iter()
            .filter(|v| v.core() == Some(CoreDesignation::Expected))
            .collect()
    }

    /// Returns permissible variables (core = Perm).
    fn permissible_variables(&self) -> Vec<&Self::Variable> {
        self.variables()
            .iter()
            .filter(|v| v.core() == Some(CoreDesignation::Permissible))
            .collect()
    }
}

/// Common trait for all CDISC variable types.
///
/// This trait provides a unified interface for variable metadata across all standards.
pub trait CdiscVariable {
    /// Returns the variable name.
    fn name(&self) -> &str;

    /// Returns the human-readable label.
    fn label(&self) -> Option<&str>;

    /// Returns the data type (Char or Num).
    fn data_type(&self) -> VariableType;

    /// Returns the core designation.
    fn core(&self) -> Option<CoreDesignation>;

    /// Returns the codelist code for controlled terminology validation.
    fn codelist_code(&self) -> Option<&str>;

    /// Returns the described value domain (e.g., "ISO 8601 datetime").
    fn described_value_domain(&self) -> Option<&str>;

    /// Returns the variable ordering.
    fn order(&self) -> Option<u32>;
}
