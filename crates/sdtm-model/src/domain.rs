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
        if let Some(variable) = self
            .variables
            .iter()
            .find(|var| var.name.eq_ignore_ascii_case(&expected))
        {
            return Some(variable.name.as_str());
        }
        let mut candidates: Vec<&str> = self
            .variables
            .iter()
            .map(|var| var.name.as_str())
            .filter(|name| {
                ends_with_case_insensitive(name, "SEQ") && !name.eq_ignore_ascii_case("SEQ")
            })
            .collect();
        candidates.sort_by_key(|name| name.to_ascii_uppercase());
        if let Some(name) = candidates.first() {
            return Some(*name);
        }
        let mut grp_candidates: Vec<&str> = self
            .variables
            .iter()
            .map(|var| var.name.as_str())
            .filter(|name| {
                ends_with_case_insensitive(name, "GRPID") && !name.eq_ignore_ascii_case("GRPID")
            })
            .collect();
        grp_candidates.sort_by_key(|name| name.to_ascii_uppercase());
        grp_candidates.first().copied()
    }
}

/// SDTM variable roles per SDTMIG v3.4 Chapter 2 (Section 2.1).
/// Roles define the type of information conveyed by a variable.
///
/// The order of variants defines the standard column ordering:
/// 1. Identifier - identify study, subject, domain, sequence
/// 2. Topic - focus of the observation
/// 3. Qualifiers (in order): Grouping, Result, Synonym, Record, Variable
/// 4. Rule - Trial Design Model conditions (start, end, branch, loop)
/// 5. Timing - timing of the observation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SdtmRole {
    /// Identifier variables (STUDYID, USUBJID, DOMAIN, --SEQ)
    Identifier,
    /// Topic variables - focus of observation (e.g., lab test name)
    Topic,
    /// Grouping Qualifier - group observations (--CAT, --SCAT)
    GroupingQualifier,
    /// Result Qualifier - describe results (--ORRES, --STRESC, --STRESN)
    ResultQualifier,
    /// Synonym Qualifier - alternative names (--MODIFY, --DECOD)
    SynonymQualifier,
    /// Record Qualifier - attributes of the record as a whole
    RecordQualifier,
    /// Variable Qualifier - modify specific variables (--ORRESU, --DOSU)
    VariableQualifier,
    /// Rule variables - Trial Design conditions (start, end, branch, loop)
    Rule,
    /// Timing variables - timing of observation (--STDTC, --ENDTC, --DY)
    Timing,
}

impl SdtmRole {
    /// Parse a role string from SDTMIG metadata into an SdtmRole.
    /// Returns None for empty or unrecognized role strings.
    fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("IDENTIFIER") {
            Some(SdtmRole::Identifier)
        } else if trimmed.eq_ignore_ascii_case("TOPIC") {
            Some(SdtmRole::Topic)
        } else if trimmed.eq_ignore_ascii_case("GROUPING QUALIFIER") {
            Some(SdtmRole::GroupingQualifier)
        } else if trimmed.eq_ignore_ascii_case("RESULT QUALIFIER") {
            Some(SdtmRole::ResultQualifier)
        } else if trimmed.eq_ignore_ascii_case("SYNONYM QUALIFIER") {
            Some(SdtmRole::SynonymQualifier)
        } else if trimmed.eq_ignore_ascii_case("RECORD QUALIFIER") {
            Some(SdtmRole::RecordQualifier)
        } else if trimmed.eq_ignore_ascii_case("VARIABLE QUALIFIER") {
            Some(SdtmRole::VariableQualifier)
        } else if trimmed.eq_ignore_ascii_case("RULE") {
            Some(SdtmRole::Rule)
        } else if trimmed.eq_ignore_ascii_case("TIMING") {
            Some(SdtmRole::Timing)
        } else {
            None
        }
    }

    /// Returns the sort order for this role (lower = earlier in output).
    /// Per SDTMIG v3.4 Chapter 2: Identifiers, Topic, Qualifiers, Rule, Timing.
    fn sort_order(&self) -> u8 {
        match self {
            SdtmRole::Identifier => 1,
            SdtmRole::Topic => 2,
            SdtmRole::GroupingQualifier => 3,
            SdtmRole::ResultQualifier => 4,
            SdtmRole::SynonymQualifier => 5,
            SdtmRole::RecordQualifier => 6,
            SdtmRole::VariableQualifier => 7,
            SdtmRole::Rule => 8,
            SdtmRole::Timing => 9,
        }
    }
}

/// Get the sort key for a variable based on SDTM role and order.
/// Uses the variable's order field if present, otherwise uses role order * 1000.
/// This ensures variables are sorted by role first, then by their defined order within each role.
fn variable_sort_key(var: &Variable) -> (u8, u32) {
    let role = var
        .role
        .as_ref()
        .and_then(|r| SdtmRole::parse(r))
        .map(|r| r.sort_order())
        .unwrap_or(99); // Unknown roles sort last

    let order = var.order.unwrap_or(999);
    (role, order)
}

fn ends_with_case_insensitive(value: &str, suffix: &str) -> bool {
    if value.len() < suffix.len() {
        return false;
    }
    value[value.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
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
