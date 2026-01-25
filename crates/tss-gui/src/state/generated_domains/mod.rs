//! Entry types for generated domains.
//!
//! This module separates entry types by domain category per SDTM-IG v3.4:
//!
//! - **Special-Purpose Domains**: CO (Comments)
//! - **Relationship Domains**: RELREC, RELSPEC, RELSUB
//!
//! Each domain has its own submodule with entry types.

pub mod co;
pub mod relrec;
pub mod relspec;
pub mod relsub;

pub use co::CommentEntry;
pub use relrec::{RelrecEntry, RelrecRelType};
pub use relspec::RelspecEntry;
pub use relsub::RelsubEntry;

// =============================================================================
// UNIFIED ENTRY ENUM
// =============================================================================

/// A generated domain entry.
///
/// This enum allows storing entries for different generated domains
/// in a single collection when needed.
#[derive(Debug, Clone, PartialEq)]
pub enum GeneratedDomainEntry {
    /// CO (Comments) - Special-Purpose domain.
    Comment(CommentEntry),
    /// RELREC (Related Records) - Relationship domain.
    RelatedRecord(RelrecEntry),
    /// RELSPEC (Related Specimens) - Relationship domain.
    RelatedSpecimen(RelspecEntry),
    /// RELSUB (Related Subjects) - Relationship domain.
    RelatedSubject(RelsubEntry),
}

impl From<CommentEntry> for GeneratedDomainEntry {
    fn from(entry: CommentEntry) -> Self {
        Self::Comment(entry)
    }
}

impl From<RelrecEntry> for GeneratedDomainEntry {
    fn from(entry: RelrecEntry) -> Self {
        Self::RelatedRecord(entry)
    }
}

impl From<RelspecEntry> for GeneratedDomainEntry {
    fn from(entry: RelspecEntry) -> Self {
        Self::RelatedSpecimen(entry)
    }
}

impl From<RelsubEntry> for GeneratedDomainEntry {
    fn from(entry: RelsubEntry) -> Self {
        Self::RelatedSubject(entry)
    }
}

// =============================================================================
// GENERATED DOMAIN TYPE
// =============================================================================

/// Type of generated domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneratedDomainType {
    /// CO (Comments) - Special-Purpose domain.
    Comments,
    /// RELREC (Related Records) - Relationship domain.
    RelatedRecords,
    /// RELSPEC (Related Specimens) - Relationship domain.
    RelatedSpecimens,
    /// RELSUB (Related Subjects) - Relationship domain.
    RelatedSubjects,
}

impl GeneratedDomainType {
    /// Get the CDISC domain code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Comments => "CO",
            Self::RelatedRecords => "RELREC",
            Self::RelatedSpecimens => "RELSPEC",
            Self::RelatedSubjects => "RELSUB",
        }
    }

    /// Get the domain class per SDTM-IG.
    pub fn domain_class(&self) -> &'static str {
        match self {
            Self::Comments => "Special-Purpose",
            Self::RelatedRecords | Self::RelatedSpecimens | Self::RelatedSubjects => "Relationship",
        }
    }

    /// Get the human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Comments => "Comments",
            Self::RelatedRecords => "Related Records",
            Self::RelatedSpecimens => "Related Specimens",
            Self::RelatedSubjects => "Related Subjects",
        }
    }

    /// Get the SDTM-IG section reference.
    pub fn ig_section(&self) -> &'static str {
        match self {
            Self::Comments => "5.1",
            Self::RelatedRecords => "8.2",
            Self::RelatedSpecimens => "8.8",
            Self::RelatedSubjects => "8.7",
        }
    }

    /// All generated domain types.
    pub const ALL: [GeneratedDomainType; 4] = [
        Self::Comments,
        Self::RelatedRecords,
        Self::RelatedSpecimens,
        Self::RelatedSubjects,
    ];
}
