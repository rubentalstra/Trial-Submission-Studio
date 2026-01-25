//! Messages for generated domain builders.
//!
//! This module separates messages by domain category per SDTM-IG v3.4:
//!
//! - **Special-Purpose Domains**: CO (Comments)
//! - **Relationship Domains**: RELREC, RELSPEC, RELSUB
//!
//! Each domain has its own submodule with messages and builder state.

pub mod co;
pub mod relrec;
pub mod relspec;
pub mod relsub;

pub use co::{CoBuilderState, CoMessage};
pub use relrec::{RelrecBuilderState, RelrecMessage};
pub use relspec::{RelspecBuilderState, RelspecMessage};
pub use relsub::{RelsubBuilderState, RelsubMessage};

use crate::state::GeneratedDomainType;

/// Root message for all generated domain builders.
#[derive(Debug, Clone)]
pub enum GeneratedDomainMessage {
    /// Select which domain type to create.
    SelectDomainType(GeneratedDomainType),

    /// Cancel and return to home.
    Cancel,

    /// Create the domain from current entries.
    CreateDomain,

    /// CO (Comments) - Special-Purpose domain.
    Co(CoMessage),

    /// RELREC (Related Records) - Relationship domain.
    Relrec(RelrecMessage),

    /// RELSPEC (Related Specimens) - Relationship domain.
    Relspec(RelspecMessage),

    /// RELSUB (Related Subjects) - Relationship domain.
    Relsub(RelsubMessage),
}

/// Combined builder state for all generated domain types.
#[derive(Debug, Clone, Default)]
pub struct GeneratedDomainBuilderState {
    /// Currently selected domain type (or None if choosing).
    pub selected_type: Option<GeneratedDomainType>,

    /// CO (Comments) builder state.
    pub co: CoBuilderState,

    /// RELREC (Related Records) builder state.
    pub relrec: RelrecBuilderState,

    /// RELSPEC (Related Specimens) builder state.
    pub relspec: RelspecBuilderState,

    /// RELSUB (Related Subjects) builder state.
    pub relsub: RelsubBuilderState,
}

impl GeneratedDomainBuilderState {
    /// Create new builder state for a specific domain type.
    pub fn for_domain(domain_type: GeneratedDomainType) -> Self {
        Self {
            selected_type: Some(domain_type),
            ..Default::default()
        }
    }

    /// Get all entries as GeneratedDomainEntry for the current domain type.
    pub fn get_entries(&self) -> Vec<crate::state::GeneratedDomainEntry> {
        match self.selected_type {
            Some(GeneratedDomainType::Comments) => self
                .co
                .entries
                .iter()
                .cloned()
                .map(crate::state::GeneratedDomainEntry::Comment)
                .collect(),
            Some(GeneratedDomainType::RelatedRecords) => self
                .relrec
                .entries
                .iter()
                .cloned()
                .map(crate::state::GeneratedDomainEntry::RelatedRecord)
                .collect(),
            Some(GeneratedDomainType::RelatedSpecimens) => self
                .relspec
                .entries
                .iter()
                .cloned()
                .map(crate::state::GeneratedDomainEntry::RelatedSpecimen)
                .collect(),
            Some(GeneratedDomainType::RelatedSubjects) => self
                .relsub
                .entries
                .iter()
                .cloned()
                .map(crate::state::GeneratedDomainEntry::RelatedSubject)
                .collect(),
            None => Vec::new(),
        }
    }
}
