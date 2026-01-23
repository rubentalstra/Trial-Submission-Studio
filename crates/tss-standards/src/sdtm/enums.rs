//! SDTM-specific enumerations.
//!
//! This module contains enumerations specific to SDTM:
//! - [`SdtmDatasetClass`] - Dataset observation classes (Interventions, Events, Findings, etc.)
//! - [`VariableRole`] - Variable roles (Identifier, Topic, Qualifier, etc.)
//!
//! Core designation is shared across standards and is in the `traits` module.
//!
//! # SDTMIG Reference
//!
//! - Dataset classes: SDTMIG v3.4 Chapter 2 (Fundamentals of the SDTM)
//! - Variable roles: SDTMIG v3.4 Section 2.1 (General Observation Classes)

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
/// use tss_standards::SdtmDatasetClass;
///
/// let class: SdtmDatasetClass = "Findings".parse().unwrap();
/// assert!(class.is_general_observation());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SdtmDatasetClass {
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

impl SdtmDatasetClass {
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

impl fmt::Display for SdtmDatasetClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SdtmDatasetClass {
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
    fn test_role_sort_order() {
        assert!(VariableRole::Identifier.sort_order() < VariableRole::Topic.sort_order());
        assert!(VariableRole::Topic.sort_order() < VariableRole::Timing.sort_order());
    }

    #[test]
    fn test_dataset_class_from_str() {
        assert_eq!(
            "Findings".parse::<SdtmDatasetClass>().unwrap(),
            SdtmDatasetClass::Findings
        );
        assert_eq!(
            "Special-Purpose".parse::<SdtmDatasetClass>().unwrap(),
            SdtmDatasetClass::SpecialPurpose
        );
        assert_eq!(
            "TRIAL DESIGN".parse::<SdtmDatasetClass>().unwrap(),
            SdtmDatasetClass::TrialDesign
        );
    }

    #[test]
    fn test_dataset_class_is_general_observation() {
        assert!(SdtmDatasetClass::Interventions.is_general_observation());
        assert!(SdtmDatasetClass::Events.is_general_observation());
        assert!(SdtmDatasetClass::Findings.is_general_observation());
        assert!(SdtmDatasetClass::FindingsAbout.is_general_observation());
        assert!(!SdtmDatasetClass::SpecialPurpose.is_general_observation());
        assert!(!SdtmDatasetClass::TrialDesign.is_general_observation());
    }

    #[test]
    fn test_dataset_class_general_observation_class() {
        assert_eq!(
            SdtmDatasetClass::FindingsAbout.general_observation_class(),
            Some(SdtmDatasetClass::Findings)
        );
        assert_eq!(
            SdtmDatasetClass::SpecialPurpose.general_observation_class(),
            None
        );
    }
}
