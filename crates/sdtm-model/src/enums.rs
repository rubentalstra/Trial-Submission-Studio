//! Type-safe enumerations for SDTM metadata.
//!
//! These enums provide compile-time type safety for SDTM concepts
//! that are represented as strings in standard files.
//!
//! # SDTMIG Reference
//!
//! - Variable roles: SDTMIG v3.4 Section 2.1 (General Observation Classes)
//! - Core designation: SDTMIG v3.4 Section 4.1.5

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Variable role per SDTMIG v3.4 Section 2.1.
///
/// SDTM defines 5 major roles plus 5 qualifier subclasses:
/// - **Identifier**: Keys that uniquely identify records (STUDYID, DOMAIN, USUBJID, --SEQ)
/// - **Topic**: Primary observation focus (--TERM, --TRT, --TESTCD)
/// - **Qualifiers**: Additional context (5 subclasses)
/// - **Timing**: Temporal context (--STDTC, --ENDTC, --DY, VISIT, EPOCH)
/// - **Rule**: Trial design flow control (used in TD domains)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VariableRole {
    /// Identifier: Keys that uniquely identify records.
    /// Examples: STUDYID, DOMAIN, USUBJID, --SEQ
    Identifier,

    /// Topic: Primary observation focus.
    /// Examples: --TERM, --TRT, --TESTCD, --DECOD
    Topic,

    /// Grouping Qualifier: Categorization variables.
    /// Examples: --CAT, --SCAT, --BODSYS
    GroupingQualifier,

    /// Result Qualifier: Observation results.
    /// Examples: --ORRES, --STRESC, --STRESN
    ResultQualifier,

    /// Synonym Qualifier: Alternative names/codes.
    /// Examples: --MODIFY, --DECOD, --TEST
    SynonymQualifier,

    /// Record Qualifier: Record-level attributes.
    /// Examples: --REASND, AESLIFE, AGE, SEX, RACE
    RecordQualifier,

    /// Variable Qualifier: Variable-level metadata.
    /// Examples: --ORRESU, --ORNRHI, --ORNRLO
    VariableQualifier,

    /// Timing: Temporal context variables.
    /// Examples: --STDTC, --ENDTC, --DY, VISIT, VISITNUM, EPOCH
    Timing,

    /// Rule: Trial design flow control (TD domains only).
    Rule,
}

impl VariableRole {
    /// Returns the canonical name as it appears in SDTMIG.
    pub fn as_str(&self) -> &'static str {
        match self {
            VariableRole::Identifier => "Identifier",
            VariableRole::Topic => "Topic",
            VariableRole::GroupingQualifier => "Grouping Qualifier",
            VariableRole::ResultQualifier => "Result Qualifier",
            VariableRole::SynonymQualifier => "Synonym Qualifier",
            VariableRole::RecordQualifier => "Record Qualifier",
            VariableRole::VariableQualifier => "Variable Qualifier",
            VariableRole::Timing => "Timing",
            VariableRole::Rule => "Rule",
        }
    }

    /// Returns the sort order for variable ordering per SDTMIG.
    /// Identifiers come first, then Topic, Qualifiers, Rule, and Timing last.
    pub fn sort_order(&self) -> u8 {
        match self {
            VariableRole::Identifier => 1,
            VariableRole::Topic => 2,
            VariableRole::GroupingQualifier => 3,
            VariableRole::ResultQualifier => 4,
            VariableRole::SynonymQualifier => 5,
            VariableRole::RecordQualifier => 6,
            VariableRole::VariableQualifier => 7,
            VariableRole::Rule => 8,
            VariableRole::Timing => 9,
        }
    }

    /// Returns true if this is a qualifier role (any of the 5 qualifier subclasses).
    pub fn is_qualifier(&self) -> bool {
        matches!(
            self,
            VariableRole::GroupingQualifier
                | VariableRole::ResultQualifier
                | VariableRole::SynonymQualifier
                | VariableRole::RecordQualifier
                | VariableRole::VariableQualifier
        )
    }
}

impl fmt::Display for VariableRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for VariableRole {
    type Err = String;

    /// Parse a role string into a `VariableRole`.
    /// Handles various formats found in standards files (case-insensitive).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Normalize: trim and uppercase
        let normalized = s.trim().to_uppercase();

        match normalized.as_str() {
            "IDENTIFIER" => Ok(VariableRole::Identifier),
            "TOPIC" => Ok(VariableRole::Topic),
            "GROUPING QUALIFIER" => Ok(VariableRole::GroupingQualifier),
            "RESULT QUALIFIER" => Ok(VariableRole::ResultQualifier),
            "SYNONYM QUALIFIER" => Ok(VariableRole::SynonymQualifier),
            "RECORD QUALIFIER" => Ok(VariableRole::RecordQualifier),
            "VARIABLE QUALIFIER" => Ok(VariableRole::VariableQualifier),
            "TIMING" => Ok(VariableRole::Timing),
            "RULE" => Ok(VariableRole::Rule),
            _ => Err(format!("Unknown variable role: {s}")),
        }
    }
}

/// Core designation per SDTMIG v3.4 Section 4.1.5.
///
/// Indicates whether a variable is required, expected, or permissible
/// for a given domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreDesignation {
    /// Required (Req): Must be present in the dataset.
    /// Null values are not allowed for required variables.
    Required,

    /// Expected (Exp): Expected when applicable.
    /// Should be present if data is collected; null allowed if not applicable.
    Expected,

    /// Permissible (Perm): Optional.
    /// May be included if data is collected.
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

impl fmt::Display for CoreDesignation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CoreDesignation {
    type Err = String;

    /// Parse a core designation string.
    /// Handles both short codes (Req, Exp, Perm) and full names.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_role_from_str() {
        assert_eq!(
            "Identifier".parse::<VariableRole>().unwrap(),
            VariableRole::Identifier
        );
        assert_eq!(
            "GROUPING QUALIFIER".parse::<VariableRole>().unwrap(),
            VariableRole::GroupingQualifier
        );
        assert_eq!(
            "timing".parse::<VariableRole>().unwrap(),
            VariableRole::Timing
        );
    }

    #[test]
    fn test_core_designation_from_str() {
        assert_eq!(
            "Req".parse::<CoreDesignation>().unwrap(),
            CoreDesignation::Required
        );
        assert_eq!(
            "EXPECTED".parse::<CoreDesignation>().unwrap(),
            CoreDesignation::Expected
        );
        assert_eq!(
            "perm".parse::<CoreDesignation>().unwrap(),
            CoreDesignation::Permissible
        );
    }

    #[test]
    fn test_role_sort_order() {
        assert!(VariableRole::Identifier.sort_order() < VariableRole::Topic.sort_order());
        assert!(VariableRole::Topic.sort_order() < VariableRole::Timing.sort_order());
    }
}
