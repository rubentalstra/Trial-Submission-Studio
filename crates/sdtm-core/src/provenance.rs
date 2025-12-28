//! Provenance tracking for derived SDTM values.
//!
//! This module provides infrastructure to track the origin and derivation
//! method for all values that are computed or transformed during SDTM processing.
//!
//! SDTMIG requires clear documentation of variable origins (Collected, Derived,
//! Assigned, Protocol) for Define-XML. This module captures that metadata at
//! processing time rather than inferring it later.
//!
//! # Tracked Derivations
//!
//! - **Study Day (--DY)**: Derived from --DTC and RFSTDTC per SDTMIG 4.1.4
//! - **Sequence (--SEQ)**: Assigned per SDTMIG 4.1.5
//! - **USUBJID**: Concatenation of STUDYID + SUBJID per SDTMIG 4.1.2
//! - **CT Normalization**: Values normalized to submission values from CT
//! - **Value Normalization**: Synonym mappings (e.g., SEX "FEMALE" → "F")

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

/// Types of variable origin per Define-XML 2.1 spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginType {
    /// Data collected directly from subjects/sites
    Collected,
    /// Value computed from other data
    Derived,
    /// Value assigned by sponsor (not collected or derived)
    Assigned,
    /// Value from protocol/study design
    Protocol,
    /// Value from predecessor study
    Predecessor,
}

impl OriginType {
    /// Returns the Define-XML string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Collected => "Collected",
            Self::Derived => "Derived",
            Self::Assigned => "Assigned",
            Self::Protocol => "Protocol",
            Self::Predecessor => "Predecessor",
        }
    }
}

/// Source of the derivation/assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginSource {
    /// Sponsor-defined derivation
    Sponsor,
    /// Investigator-provided data
    Investigator,
    /// Third-party vendor data
    Vendor,
    /// Subject-reported data
    Subject,
}

impl OriginSource {
    /// Returns the Define-XML string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sponsor => "Sponsor",
            Self::Investigator => "Investigator",
            Self::Vendor => "Vendor",
            Self::Subject => "Subject",
        }
    }
}

/// Describes how a value was derived or transformed.
#[derive(Debug, Clone)]
pub enum DerivationMethod {
    /// Study day calculation: --DY = --DTC - RFSTDTC + (adjustment)
    /// Per SDTMIG 4.1.4.4
    StudyDay {
        dtc_variable: String,
        reference_variable: String,
    },
    /// Sequence assignment: sequential integers within USUBJID
    /// Per SDTMIG 4.1.5
    Sequence,
    /// USUBJID concatenation: STUDYID + SUBJID
    /// Per SDTMIG 4.1.2
    UsubjidConcatenation,
    /// CT normalization: value matched to submission value
    CtNormalization {
        codelist: String,
        original_value: String,
        submission_value: String,
    },
    /// Value synonym mapping (e.g., SEX "FEMALE" → "F")
    ValueNormalization {
        original_value: String,
        normalized_value: String,
    },
    /// Domain-specific derivation with custom description
    Custom { description: String },
}

impl DerivationMethod {
    /// Returns a human-readable description of the derivation.
    pub fn description(&self) -> String {
        match self {
            Self::StudyDay {
                dtc_variable,
                reference_variable,
            } => format!(
                "Study day derived from {dtc_variable} relative to {reference_variable} \
                 per SDTMIG 4.1.4.4"
            ),
            Self::Sequence => {
                "Sequence number assigned within USUBJID per SDTMIG 4.1.5".to_string()
            }
            Self::UsubjidConcatenation => {
                "Concatenation of STUDYID and SUBJID per SDTMIG 4.1.2".to_string()
            }
            Self::CtNormalization {
                codelist,
                original_value,
                submission_value,
            } => format!(
                "Normalized '{original_value}' to CT submission value '{submission_value}' \
                 from codelist {codelist}"
            ),
            Self::ValueNormalization {
                original_value,
                normalized_value,
            } => format!("Value synonym mapping: '{original_value}' → '{normalized_value}'"),
            Self::Custom { description } => description.clone(),
        }
    }
}

/// A single provenance record for a derived value.
#[derive(Debug, Clone)]
pub struct ProvenanceRecord {
    /// Domain code (e.g., "DM", "AE")
    pub domain_code: String,
    /// Variable name (e.g., "AESTDY", "DMSEQ")
    pub variable_name: String,
    /// Origin type for Define-XML
    pub origin_type: OriginType,
    /// Origin source for Define-XML
    pub origin_source: OriginSource,
    /// How the value was derived
    pub method: DerivationMethod,
    /// Number of values affected (for reporting)
    pub affected_count: usize,
}

/// Thread-safe provenance tracker for a processing run.
#[derive(Debug, Clone, Default)]
pub struct ProvenanceTracker {
    records: Arc<RwLock<Vec<ProvenanceRecord>>>,
}

impl ProvenanceTracker {
    /// Create a new empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a derivation.
    pub fn record(&self, record: ProvenanceRecord) {
        if let Ok(mut records) = self.records.write() {
            records.push(record);
        }
    }

    /// Record a study day derivation.
    pub fn record_study_day(
        &self,
        domain_code: &str,
        dy_variable: &str,
        dtc_variable: &str,
        reference_variable: &str,
        count: usize,
    ) {
        self.record(ProvenanceRecord {
            domain_code: domain_code.to_string(),
            variable_name: dy_variable.to_string(),
            origin_type: OriginType::Derived,
            origin_source: OriginSource::Sponsor,
            method: DerivationMethod::StudyDay {
                dtc_variable: dtc_variable.to_string(),
                reference_variable: reference_variable.to_string(),
            },
            affected_count: count,
        });
    }

    /// Record a sequence assignment.
    pub fn record_sequence(&self, domain_code: &str, seq_variable: &str, count: usize) {
        self.record(ProvenanceRecord {
            domain_code: domain_code.to_string(),
            variable_name: seq_variable.to_string(),
            origin_type: OriginType::Derived,
            origin_source: OriginSource::Sponsor,
            method: DerivationMethod::Sequence,
            affected_count: count,
        });
    }

    /// Get all recorded provenance records.
    pub fn records(&self) -> Vec<ProvenanceRecord> {
        self.records.read().map(|r| r.clone()).unwrap_or_default()
    }

    /// Get records for a specific domain.
    pub fn records_for_domain(&self, domain_code: &str) -> Vec<ProvenanceRecord> {
        self.records()
            .into_iter()
            .filter(|r| r.domain_code.eq_ignore_ascii_case(domain_code))
            .collect()
    }

    /// Get records for a specific variable.
    pub fn records_for_variable(&self, domain_code: &str, variable: &str) -> Vec<ProvenanceRecord> {
        self.records()
            .into_iter()
            .filter(|r| {
                r.domain_code.eq_ignore_ascii_case(domain_code)
                    && r.variable_name.eq_ignore_ascii_case(variable)
            })
            .collect()
    }

    /// Generate a summary of derivations by domain and variable.
    pub fn summary(&self) -> BTreeMap<String, BTreeMap<String, Vec<String>>> {
        let mut result: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

        for record in self.records() {
            let domain_map = result.entry(record.domain_code.clone()).or_default();
            let var_methods = domain_map.entry(record.variable_name.clone()).or_default();
            var_methods.push(record.method.description());
        }

        result
    }

    /// Check if a variable has any provenance records.
    pub fn has_provenance(&self, domain_code: &str, variable: &str) -> bool {
        !self.records_for_variable(domain_code, variable).is_empty()
    }

    /// Get count of all records.
    pub fn len(&self) -> usize {
        self.records.read().map(|r| r.len()).unwrap_or(0)
    }

    /// Check if tracker is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_study_day() {
        let tracker = ProvenanceTracker::new();
        tracker.record_study_day("AE", "AESTDY", "AESTDTC", "RFSTDTC", 10);

        let records = tracker.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].domain_code, "AE");
        assert_eq!(records[0].variable_name, "AESTDY");
        assert_eq!(records[0].origin_type, OriginType::Derived);
        assert!(records[0].method.description().contains("SDTMIG 4.1.4.4"));
    }

    #[test]
    fn test_record_sequence() {
        let tracker = ProvenanceTracker::new();
        tracker.record_sequence("LB", "LBSEQ", 100);

        let records = tracker.records_for_domain("LB");
        assert_eq!(records.len(), 1);
        assert!(records[0].method.description().contains("SDTMIG 4.1.5"));
    }

    #[test]
    fn test_summary() {
        let tracker = ProvenanceTracker::new();
        tracker.record_study_day("AE", "AESTDY", "AESTDTC", "RFSTDTC", 10);
        tracker.record_sequence("AE", "AESEQ", 10);

        let summary = tracker.summary();
        assert!(summary.contains_key("AE"));
        assert_eq!(summary["AE"].len(), 2);
    }
}
