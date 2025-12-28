//! Rule mapping module for proper P21 rule identification.
//!
//! This module provides a clear mapping between validation checks and their
//! corresponding Pinnacle 21 (P21) rules from Rules.csv. Validations that
//! don't have P21 equivalents use internal rule IDs prefixed with "TRANS_".
//!
//! # Architecture
//!
//! - **P21 Rules**: Official rules from `standards/p21/Rules.csv`. These have
//!   specific meanings defined by Pinnacle 21 and should ONLY be used when
//!   our validation matches the P21 rule's definition exactly.
//!
//! - **Internal Rules**: Validations specific to this transpiler that don't
//!   have P21 equivalents. These use the "TRANS_" prefix to clearly indicate
//!   they are not official P21 rules.
//!
//! # Important
//!
//! Never invent fake P21 rule IDs! If a validation doesn't map to a real P21
//! rule, use an internal rule ID instead.

use std::collections::BTreeMap;

use sdtm_standards::loaders::P21Rule;

// ============================================================================
// P21 Rule IDs - Only use these when validation matches P21 definition exactly
// ============================================================================

/// SD0002: Null value in variable marked as Required
/// P21 Definition: Required variables (where Core attribute is 'Req') cannot be null.
pub const P21_REQUIRED_VALUE_MISSING: &str = "SD0002";

/// SD0003: Invalid ISO 8601 value for variable
/// P21 Definition: Value of Dates/Time variables (*DTC) must conform to ISO 8601.
pub const P21_ISO8601_INVALID: &str = "SD0003";

/// SD0005: Duplicate value for --SEQ variable
/// P21 Definition: --SEQ must be unique per USUBJID within a domain.
pub const P21_SEQ_DUPLICATE: &str = "SD0005";

/// SD0056: SDTM Required variable not found
/// P21 Definition: Variables described as Required must be included.
pub const P21_REQUIRED_VAR_MISSING: &str = "SD0056";

/// SD0057: SDTM Expected variable not found
/// P21 Definition: Variables described as Expected should be included.
pub const P21_EXPECTED_VAR_MISSING: &str = "SD0057";

/// SD0086: SUPPQUAL duplicate records
/// P21 Definition: QNAM must be unique per parent record (STUDYID, USUBJID, IDVAR, IDVARVAL).
pub const P21_SUPP_QNAM_DUPLICATE: &str = "SD0086";

/// SD1022: Invalid value for QNAM variable
/// P21 Definition: QNAM should be <=8 chars, cannot start with number, alphanumeric + underscore only.
pub const P21_QNAM_INVALID: &str = "SD1022";

/// SD1230: Variable datatype is not the expected SDTM datatype
/// P21 Definition: Value level datatype must match define.xml.
pub const P21_DATATYPE_MISMATCH: &str = "SD1230";

/// SD1231: Variable value is longer than defined max length
/// P21 Definition: Value level length must not exceed define.xml specification.
pub const P21_LENGTH_EXCEEDED: &str = "SD1231";

/// SD1474: Invalid value for Variable Name
/// P21 Definition: Variable name must follow SAS XPORT v5 format (<=8 chars, uppercase, etc.).
pub const P21_VARNAME_INVALID: &str = "SD1474";

/// CT2001: Variable value not found in non-extensible codelist
/// P21 Definition: Variable must use terms from CDISC CT. New terms cannot be added.
pub const P21_CT_NON_EXTENSIBLE: &str = "CT2001";

/// CT2002: Variable value not found in extensible codelist
/// P21 Definition: Variable should use terms from CDISC CT. New terms allowed if not duplicates.
pub const P21_CT_EXTENSIBLE: &str = "CT2002";

// ============================================================================
// Internal Rule IDs - For validations without P21 equivalents
// ============================================================================

/// Internal: Undocumented derivation detected (provenance tracking)
/// This is a transpiler-specific check for derived variables without provenance.
pub const TRANS_UNDOCUMENTED_DERIVATION: &str = "TRANS0001";

/// Internal: --SEQ collision across split datasets
/// P21's SD0005 only covers single domain uniqueness, not cross-split.
/// This extends SD0005 for split dataset scenarios.
pub const TRANS_SEQ_CROSS_SPLIT: &str = "TRANS0002";

/// Internal: QVAL is empty in SUPPQUAL record
/// No specific P21 rule requires non-empty QVAL.
pub const TRANS_SUPP_QVAL_EMPTY: &str = "TRANS0003";

/// Internal: Variable prefix doesn't match base domain for split datasets
/// No P21 rule specifically covers variable prefix validation for split datasets.
pub const TRANS_VARIABLE_PREFIX: &str = "TRANS0004";

/// Internal: RELREC references non-existent record
/// No specific P21 rule for RELREC referential integrity.
pub const TRANS_RELREC_INTEGRITY: &str = "TRANS0005";

/// Internal: RELSPEC structure validation
/// No specific P21 rule for RELSPEC structure.
pub const TRANS_RELSPEC_INTEGRITY: &str = "TRANS0006";

/// Internal: RELSUB references non-existent subject
/// No specific P21 rule for RELSUB referential integrity.
pub const TRANS_RELSUB_INTEGRITY: &str = "TRANS0007";

/// Internal: Date pair order validation (end date before start date)
/// This is SDTMIG guidance-based, not a specific P21 rule.
pub const TRANS_DATE_PAIR_ORDER: &str = "TRANS0008";

/// Internal: Study day calculation requires complete date
/// SDTMIG guidance-based validation for SDY derivation.
pub const TRANS_STUDY_DAY_INCOMPLETE: &str = "TRANS0009";

/// Internal: Relative timing variable validation
/// SDTMIG Chapter 4.4.7 guidance, not a specific P21 rule.
pub const TRANS_RELATIVE_TIMING: &str = "TRANS0010";

/// Internal: Duration variable usage validation
/// SDTMIG Chapter 4.4.3 guidance, not a specific P21 rule.
pub const TRANS_DURATION_USAGE: &str = "TRANS0011";

/// Internal: Findings class timing variable validation
/// SDTMIG guidance for Findings class domains.
pub const TRANS_FINDINGS_TIMING: &str = "TRANS0012";

/// Internal: General Observation identifier presence
/// SDTMIG guidance for GO class domains (STUDYID, DOMAIN, USUBJID, --SEQ).
pub const TRANS_GO_IDENTIFIERS: &str = "TRANS0013";

/// Internal: --TEST value exceeds 40 characters
/// Per SDTMIG 4.5.3.1, --TEST is limited to 40 characters (except IE/TI/TS).
pub const TRANS_TEST_LENGTH: &str = "TRANS0014";

/// Internal: --TESTCD value exceeds 8 characters
/// Per SDTMIG, --TESTCD is limited to 8 characters.
pub const TRANS_TESTCD_LENGTH: &str = "TRANS0015";

/// Internal: QNAM value exceeds 8 characters
/// Per SDTMIG 8.4, QNAM is limited to 8 characters.
pub const TRANS_QNAM_LENGTH: &str = "TRANS0016";

/// Internal: QLABEL value exceeds 40 characters
/// Per SDTMIG 4.5.3.1, QLABEL is limited to 40 characters.
pub const TRANS_QLABEL_LENGTH: &str = "TRANS0017";

/// Internal: Text value exceeds 200 characters (SAS V5 limit)
/// Per SDTMIG 4.2.1, character variables have max length 200.
pub const TRANS_TEXT_LENGTH_200: &str = "TRANS0018";

/// Internal: CO (Comments) IDVAR/IDVARVAL referential integrity
/// Per SDTMIG 8.5, CO IDVAR/IDVARVAL must reference valid records.
pub const TRANS_CO_IDVAR_INTEGRITY: &str = "TRANS0019";

/// Internal: Timing variable in SUPPQUAL
/// Per SDTMIG 8.4, timing variables should be in parent domain, not SUPP.
pub const TRANS_SUPP_TIMING_VAR: &str = "TRANS0020";

// ============================================================================
// Rule Resolver
// ============================================================================

/// Resolves rule metadata from P21 rules registry.
///
/// Use this to get official P21 rule messages, descriptions, and severities
/// instead of hardcoding them.
#[derive(Debug, Clone)]
pub struct RuleResolver {
    p21_lookup: BTreeMap<String, P21Rule>,
    /// Internal rule descriptions
    internal_descriptions: BTreeMap<&'static str, InternalRuleInfo>,
}

/// Information about an internal (non-P21) rule.
#[derive(Debug, Clone)]
pub struct InternalRuleInfo {
    pub message: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub severity: &'static str,
    pub sdtmig_reference: Option<&'static str>,
}

impl RuleResolver {
    /// Create a new rule resolver from P21 rules.
    pub fn new(p21_rules: &[P21Rule]) -> Self {
        let mut p21_lookup = BTreeMap::new();
        for rule in p21_rules {
            p21_lookup.insert(rule.rule_id.to_uppercase(), rule.clone());
        }

        let internal_descriptions = Self::build_internal_descriptions();

        Self {
            p21_lookup,
            internal_descriptions,
        }
    }

    /// Get P21 rule by ID.
    pub fn get_p21_rule(&self, rule_id: &str) -> Option<&P21Rule> {
        self.p21_lookup.get(&rule_id.to_uppercase())
    }

    /// Get internal rule info by ID.
    pub fn get_internal_rule(&self, rule_id: &str) -> Option<&InternalRuleInfo> {
        self.internal_descriptions.get(rule_id)
    }

    /// Check if a rule ID is a P21 rule.
    pub fn is_p21_rule(&self, rule_id: &str) -> bool {
        self.p21_lookup.contains_key(&rule_id.to_uppercase())
    }

    /// Check if a rule ID is an internal (TRANS_*) rule.
    pub fn is_internal_rule(rule_id: &str) -> bool {
        rule_id.starts_with("TRANS")
    }

    /// Get the message for a rule (P21 or internal).
    pub fn get_message(&self, rule_id: &str) -> Option<String> {
        if let Some(p21) = self.get_p21_rule(rule_id) {
            if !p21.message.is_empty() {
                return Some(p21.message.clone());
            }
        }
        if let Some(internal) = self.get_internal_rule(rule_id) {
            return Some(internal.message.to_string());
        }
        None
    }

    /// Get the description for a rule (P21 or internal).
    pub fn get_description(&self, rule_id: &str) -> Option<String> {
        if let Some(p21) = self.get_p21_rule(rule_id) {
            if !p21.description.is_empty() {
                return Some(p21.description.clone());
            }
        }
        if let Some(internal) = self.get_internal_rule(rule_id) {
            return Some(internal.description.to_string());
        }
        None
    }

    /// Get the category for a rule (P21 or internal).
    pub fn get_category(&self, rule_id: &str) -> Option<String> {
        if let Some(p21) = self.get_p21_rule(rule_id) {
            return p21.category.clone();
        }
        if let Some(internal) = self.get_internal_rule(rule_id) {
            return Some(internal.category.to_string());
        }
        None
    }

    /// Get the severity for a rule (P21 or internal).
    pub fn get_severity(&self, rule_id: &str) -> Option<String> {
        if let Some(p21) = self.get_p21_rule(rule_id) {
            return p21.severity.clone();
        }
        if let Some(internal) = self.get_internal_rule(rule_id) {
            return Some(internal.severity.to_string());
        }
        None
    }

    fn build_internal_descriptions() -> BTreeMap<&'static str, InternalRuleInfo> {
        let mut map = BTreeMap::new();

        map.insert(
            TRANS_UNDOCUMENTED_DERIVATION,
            InternalRuleInfo {
                message: "Derived variable has values but no documented provenance",
                description: "Derived variables should have tracked provenance to document \
                              how values were calculated or derived.",
                category: "Provenance",
                severity: "Warning",
                sdtmig_reference: None,
            },
        );

        map.insert(
            TRANS_SEQ_CROSS_SPLIT,
            InternalRuleInfo {
                message: "--SEQ values collide across split datasets",
                description: "When a domain is split across multiple datasets (e.g., LBCH, LBHE), \
                              --SEQ values must remain unique per USUBJID across all splits.",
                category: "Consistency",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 4.1.5"),
            },
        );

        map.insert(
            TRANS_SUPP_QVAL_EMPTY,
            InternalRuleInfo {
                message: "QVAL is empty in SUPPQUAL record",
                description: "SUPPQUAL records should have non-empty QVAL values. \
                              An empty QVAL makes the supplemental qualifier meaningless.",
                category: "Completeness",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 8.4"),
            },
        );

        map.insert(
            TRANS_VARIABLE_PREFIX,
            InternalRuleInfo {
                message: "Variable prefix does not match base domain code",
                description: "For split datasets (e.g., LBCH), variable prefixes should use \
                              the base domain code (LB), not the dataset name (LBCH).",
                category: "Naming",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 4.1.7"),
            },
        );

        map.insert(
            TRANS_RELREC_INTEGRITY,
            InternalRuleInfo {
                message: "RELREC references non-existent record",
                description: "RELREC dataset references must point to valid records \
                              in the referenced domains.",
                category: "Referential Integrity",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 8"),
            },
        );

        map.insert(
            TRANS_RELSPEC_INTEGRITY,
            InternalRuleInfo {
                message: "RELSPEC structure validation failed",
                description: "RELSPEC dataset must contain required columns for \
                              specimen relationship tracking.",
                category: "Structure",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 8"),
            },
        );

        map.insert(
            TRANS_RELSUB_INTEGRITY,
            InternalRuleInfo {
                message: "RELSUB references non-existent subject",
                description: "RELSUB dataset USUBJID values must exist in the DM domain.",
                category: "Referential Integrity",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 8"),
            },
        );

        map.insert(
            TRANS_DATE_PAIR_ORDER,
            InternalRuleInfo {
                message: "End date precedes start date",
                description: "End date/time variables should not have values earlier \
                              than their corresponding start date/time variables.",
                category: "Consistency",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 4.1.4"),
            },
        );

        map.insert(
            TRANS_STUDY_DAY_INCOMPLETE,
            InternalRuleInfo {
                message: "Study day calculation requires complete date",
                description: "Study day (--DY) derivation requires a complete date \
                              (YYYY-MM-DD) to calculate correctly.",
                category: "Format",
                severity: "Warning",
                sdtmig_reference: Some("SDTMIG 4.1.4"),
            },
        );

        map.insert(
            TRANS_RELATIVE_TIMING,
            InternalRuleInfo {
                message: "Relative timing variable validation failed",
                description: "Relative timing variables (--STRF, --ENRF) must follow \
                              SDTMIG guidance for proper reference point usage.",
                category: "Consistency",
                severity: "Warning",
                sdtmig_reference: Some("SDTMIG 4.4.7"),
            },
        );

        map.insert(
            TRANS_DURATION_USAGE,
            InternalRuleInfo {
                message: "Duration variable usage validation failed",
                description: "--DUR should not be populated when both --STDTC and --ENDTC \
                              are complete, as duration can be derived.",
                category: "Consistency",
                severity: "Warning",
                sdtmig_reference: Some("SDTMIG 4.4.3"),
            },
        );

        map.insert(
            TRANS_FINDINGS_TIMING,
            InternalRuleInfo {
                message: "Findings class timing variable issue",
                description: "Findings class domains have specific timing variable requirements \
                              that differ from other observation classes.",
                category: "Consistency",
                severity: "Warning",
                sdtmig_reference: Some("SDTMIG 6"),
            },
        );

        map.insert(
            TRANS_GO_IDENTIFIERS,
            InternalRuleInfo {
                message: "General Observation identifier missing",
                description: "General Observation class domains (except DM) require \
                              STUDYID, DOMAIN, USUBJID, and --SEQ.",
                category: "Presence",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 4.1"),
            },
        );

        map.insert(
            TRANS_TEST_LENGTH,
            InternalRuleInfo {
                message: "--TEST value exceeds 40 characters",
                description: "Per SDTMIG v3.4 Section 4.5.3.1, the length of --TEST is \
                              limited to 40 characters to conform to SAS V5 transport \
                              file format. IE, TI, and TS domains allow up to 200 characters.",
                category: "Length",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 4.5.3.1"),
            },
        );

        map.insert(
            TRANS_TESTCD_LENGTH,
            InternalRuleInfo {
                message: "--TESTCD value exceeds 8 characters",
                description: "Per SDTMIG, the value of --TESTCD cannot be longer than \
                              8 characters, and cannot start with a number.",
                category: "Length",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 4.2.1"),
            },
        );

        map.insert(
            TRANS_QNAM_LENGTH,
            InternalRuleInfo {
                message: "QNAM value exceeds 8 characters",
                description: "Per SDTMIG v3.4 Section 8.4, QNAM serves the same purpose \
                              as --TESTCD within supplemental qualifier datasets, and is \
                              subject to the same constraints: max 8 characters.",
                category: "Length",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 8.4"),
            },
        );

        map.insert(
            TRANS_QLABEL_LENGTH,
            InternalRuleInfo {
                message: "QLABEL value exceeds 40 characters",
                description: "Per SDTMIG v3.4 Section 4.5.3.1, the QLABEL in SUPPQUAL \
                              datasets is limited to 40 characters, same as --TEST.",
                category: "Length",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 4.5.3.1"),
            },
        );

        map.insert(
            TRANS_TEXT_LENGTH_200,
            InternalRuleInfo {
                message: "Text value exceeds 200 characters",
                description: "Per SDTMIG v3.4 Section 4.2.1, the maximum SAS V5 transport \
                              file character variable length is 200 characters. Values \
                              exceeding this must be split into SUPP-- records.",
                category: "Length",
                severity: "Warning",
                sdtmig_reference: Some("SDTMIG 4.5.3.2"),
            },
        );

        map.insert(
            TRANS_CO_IDVAR_INTEGRITY,
            InternalRuleInfo {
                message: "CO IDVAR/IDVARVAL references non-existent record",
                description: "Per SDTMIG v3.4 Section 8.5, the CO (Comments) domain uses \
                              RDOMAIN, IDVAR, and IDVARVAL to link comments to specific \
                              records in other domains. These references must point to \
                              valid records.",
                category: "Referential Integrity",
                severity: "Error",
                sdtmig_reference: Some("SDTMIG 8.5"),
            },
        );

        map.insert(
            TRANS_SUPP_TIMING_VAR,
            InternalRuleInfo {
                message: "Timing variable found in SUPPQUAL",
                description: "Per SDTMIG v3.4 Section 8.4, timing variables (--DTC, --STDTC, \
                              --ENDTC, --DY, --DUR, etc.) should be included in the parent \
                              domain, not as supplemental qualifiers. Timing information \
                              in SUPP may indicate incorrect domain design.",
                category: "Structure",
                severity: "Warning",
                sdtmig_reference: Some("SDTMIG 8.4"),
            },
        );

        map
    }
}

impl Default for RuleResolver {
    fn default() -> Self {
        Self::new(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_rule_prefix() {
        assert!(RuleResolver::is_internal_rule(
            TRANS_UNDOCUMENTED_DERIVATION
        ));
        assert!(RuleResolver::is_internal_rule(TRANS_SEQ_CROSS_SPLIT));
        assert!(!RuleResolver::is_internal_rule(P21_REQUIRED_VALUE_MISSING));
        assert!(!RuleResolver::is_internal_rule(P21_SEQ_DUPLICATE));
    }

    #[test]
    fn test_resolver_internal_info() {
        let resolver = RuleResolver::default();
        let info = resolver.get_internal_rule(TRANS_SEQ_CROSS_SPLIT);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.category, "Consistency");
        assert_eq!(info.severity, "Error");
    }
}
