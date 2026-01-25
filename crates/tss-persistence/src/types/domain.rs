//! Domain state snapshots for persistence.

use std::collections::{BTreeMap, BTreeSet};

use rkyv::{Archive, Deserialize, Serialize};

use super::{GeneratedDomainEntrySnapshot, GeneratedDomainTypeSnapshot, SuppColumnSnapshot};

// =============================================================================
// DOMAIN SNAPSHOT ENUM
// =============================================================================

/// Snapshot of a domain's state for persistence.
///
/// This is an enum to handle both source (CSV-mapped) and generated domains.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub enum DomainSnapshot {
    /// Source domain (mapped from CSV file).
    Source(SourceDomainSnapshot),
    /// Generated domain (CO, RELREC, RELSPEC, RELSUB).
    Generated(GeneratedDomainSnapshot),
}

impl DomainSnapshot {
    /// Create a new source domain snapshot.
    pub fn new(domain_code: impl Into<String>) -> Self {
        Self::Source(SourceDomainSnapshot::new(domain_code))
    }

    /// Create a source snapshot with a label.
    pub fn with_label(domain_code: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Source(SourceDomainSnapshot::with_label(domain_code, label))
    }

    /// Create a new generated domain snapshot.
    pub fn new_generated(domain_type: GeneratedDomainTypeSnapshot) -> Self {
        Self::Generated(GeneratedDomainSnapshot::new(domain_type))
    }

    /// Get the domain code.
    pub fn domain_code(&self) -> &str {
        match self {
            Self::Source(s) => &s.domain_code,
            Self::Generated(g) => g.domain_type.code(),
        }
    }

    /// Check if this is a source domain.
    pub fn is_source(&self) -> bool {
        matches!(self, Self::Source(_))
    }

    /// Check if this is a generated domain.
    pub fn is_generated(&self) -> bool {
        matches!(self, Self::Generated(_))
    }

    /// Get as source snapshot.
    pub fn as_source(&self) -> Option<&SourceDomainSnapshot> {
        match self {
            Self::Source(s) => Some(s),
            Self::Generated(_) => None,
        }
    }

    /// Get as source snapshot mutably.
    pub fn as_source_mut(&mut self) -> Option<&mut SourceDomainSnapshot> {
        match self {
            Self::Source(s) => Some(s),
            Self::Generated(_) => None,
        }
    }

    /// Get as generated snapshot.
    pub fn as_generated(&self) -> Option<&GeneratedDomainSnapshot> {
        match self {
            Self::Source(_) => None,
            Self::Generated(g) => Some(g),
        }
    }
}

// =============================================================================
// SOURCE DOMAIN SNAPSHOT
// =============================================================================

/// Snapshot of a source domain's state (mapped from CSV).
///
/// # Design Note
///
/// This snapshot intentionally does NOT include:
/// - **Normalization pipeline**: 100% derived via `infer_normalization_rules(domain)` from SDTM metadata
/// - **Validation report**: Pure cache computed from `domain + DataFrame + not_collected`
///
/// Both are regenerated on load from the persisted mapping state.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct SourceDomainSnapshot {
    /// Domain code (e.g., "DM", "AE").
    pub domain_code: String,

    /// Human-readable label (e.g., "Demographics").
    pub label: Option<String>,

    /// Mapping state snapshot.
    pub mapping: MappingSnapshot,

    /// SUPP configuration for unmapped columns.
    pub supp_config: BTreeMap<String, SuppColumnSnapshot>,
}

impl SourceDomainSnapshot {
    /// Create a new source domain snapshot.
    pub fn new(domain_code: impl Into<String>) -> Self {
        Self {
            domain_code: domain_code.into(),
            label: None,
            mapping: MappingSnapshot::default(),
            supp_config: BTreeMap::new(),
        }
    }

    /// Create a snapshot with a label.
    pub fn with_label(domain_code: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            domain_code: domain_code.into(),
            label: Some(label.into()),
            mapping: MappingSnapshot::default(),
            supp_config: BTreeMap::new(),
        }
    }
}

// =============================================================================
// GENERATED DOMAIN SNAPSHOT
// =============================================================================

/// Snapshot of a generated domain's state (CO, RELREC, RELSPEC, RELSUB).
///
/// Generated domains store their entries directly. The DataFrame is regenerated
/// from entries on load using the generation service.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct GeneratedDomainSnapshot {
    /// Type of generated domain.
    pub domain_type: GeneratedDomainTypeSnapshot,

    /// Entries used to generate the domain data.
    pub entries: Vec<GeneratedDomainEntrySnapshot>,
}

impl GeneratedDomainSnapshot {
    /// Create a new generated domain snapshot.
    pub fn new(domain_type: GeneratedDomainTypeSnapshot) -> Self {
        Self {
            domain_type,
            entries: Vec::new(),
        }
    }

    /// Create with entries.
    pub fn with_entries(
        domain_type: GeneratedDomainTypeSnapshot,
        entries: Vec<GeneratedDomainEntrySnapshot>,
    ) -> Self {
        Self {
            domain_type,
            entries,
        }
    }
}

impl GeneratedDomainTypeSnapshot {
    /// Get the CDISC domain code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Comments => "CO",
            Self::RelatedRecords => "RELREC",
            Self::RelatedSpecimens => "RELSPEC",
            Self::RelatedSubjects => "RELSUB",
        }
    }
}

/// Snapshot of mapping state.
///
/// Note: We don't persist suggestions - they're regenerated from source data.
/// We only persist user decisions (accepted, not_collected, omitted, auto_generated).
#[derive(Debug, Clone, Default, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct MappingSnapshot {
    /// Study ID at time of save.
    pub study_id: String,

    /// User-accepted mappings: variable_name -> MappingEntry.
    pub accepted: BTreeMap<String, MappingEntry>,

    /// Variables marked as "not collected" with Define-XML reason.
    pub not_collected: BTreeMap<String, String>,

    /// Variables marked to be omitted from output.
    pub omitted: BTreeSet<String>,

    /// Variables that are auto-generated by the transform system.
    pub auto_generated: BTreeSet<String>,
}

impl MappingSnapshot {
    /// Create a new mapping snapshot.
    pub fn new(study_id: impl Into<String>) -> Self {
        Self {
            study_id: study_id.into(),
            accepted: BTreeMap::new(),
            not_collected: BTreeMap::new(),
            omitted: BTreeSet::new(),
            auto_generated: BTreeSet::new(),
        }
    }

    /// Check if the mapping has any user decisions.
    pub fn has_decisions(&self) -> bool {
        !self.accepted.is_empty()
            || !self.not_collected.is_empty()
            || !self.omitted.is_empty()
            || !self.auto_generated.is_empty()
    }
}

/// A single mapping entry.
///
/// Note: Confidence is intentionally NOT stored. Confidence is only meaningful
/// during a mapping session (showing how confident the suggestion algorithm is).
/// Once a user accepts a mapping, they've validated it - the confidence score
/// becomes meaningless. It's transient GUI state, not persisted data.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct MappingEntry {
    /// Source column name.
    pub source_column: String,
}

impl MappingEntry {
    /// Create a new mapping entry.
    pub fn new(source_column: impl Into<String>) -> Self {
        Self {
            source_column: source_column.into(),
        }
    }
}
