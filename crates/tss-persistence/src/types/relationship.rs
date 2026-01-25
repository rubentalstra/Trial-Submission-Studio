//! Relationship and special-purpose domain entry snapshots for persistence.
//!
//! These types mirror the GUI state entry types but with rkyv serialization.

use rkyv::{Archive, Deserialize, Serialize};

// =============================================================================
// CO (COMMENTS) SNAPSHOT
// =============================================================================

/// Snapshot of a comment entry for CO domain.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct CommentEntrySnapshot {
    /// Subject identifier (USUBJID).
    pub usubjid: String,

    /// Comment text (COVAL).
    pub comment: String,

    /// Related domain (RDOMAIN).
    pub rdomain: Option<String>,

    /// Identifying variable name (IDVAR).
    pub idvar: Option<String>,

    /// Identifying variable value (IDVARVAL).
    pub idvarval: Option<String>,

    /// Comment reference (COREF).
    pub coref: Option<String>,

    /// Date/time of comment (CODTC).
    pub codtc: Option<String>,

    /// Evaluator (COEVAL).
    pub coeval: Option<String>,
}

// =============================================================================
// RELREC (RELATED RECORDS) SNAPSHOT
// =============================================================================

/// Snapshot of a related record entry for RELREC domain.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct RelrecEntrySnapshot {
    /// Relationship identifier (RELID).
    pub relid: String,

    /// Subject identifier (USUBJID).
    pub usubjid: Option<String>,

    /// Related domain (RDOMAIN).
    pub rdomain: String,

    /// Identifying variable name (IDVAR).
    pub idvar: String,

    /// Identifying variable value (IDVARVAL).
    pub idvarval: Option<String>,

    /// Relationship type (RELTYPE).
    pub reltype: Option<RelrecRelTypeSnapshot>,
}

/// RELTYPE values for RELREC domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub enum RelrecRelTypeSnapshot {
    /// Single record in the relationship.
    One,
    /// Multiple records in the relationship.
    Many,
}

// =============================================================================
// RELSPEC (RELATED SPECIMENS) SNAPSHOT
// =============================================================================

/// Snapshot of a related specimen entry for RELSPEC domain.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct RelspecEntrySnapshot {
    /// Subject identifier (USUBJID).
    pub usubjid: String,

    /// Specimen identifier (REFID).
    pub refid: String,

    /// Specimen type (SPEC).
    pub spec: Option<String>,

    /// Parent specimen identifier (PARENT).
    pub parent: Option<String>,
}

// =============================================================================
// RELSUB (RELATED SUBJECTS) SNAPSHOT
// =============================================================================

/// Snapshot of a related subject entry for RELSUB domain.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct RelsubEntrySnapshot {
    /// Subject identifier (USUBJID).
    pub usubjid: String,

    /// Related subject identifier (RSUBJID).
    pub rsubjid: String,

    /// Subject relationship (SREL).
    pub srel: String,
}

// =============================================================================
// UNIFIED ENTRY ENUM
// =============================================================================

/// Snapshot of a generated domain entry.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub enum GeneratedDomainEntrySnapshot {
    /// CO (Comments) entry.
    Comment(CommentEntrySnapshot),
    /// RELREC (Related Records) entry.
    RelatedRecord(RelrecEntrySnapshot),
    /// RELSPEC (Related Specimens) entry.
    RelatedSpecimen(RelspecEntrySnapshot),
    /// RELSUB (Related Subjects) entry.
    RelatedSubject(RelsubEntrySnapshot),
}

// =============================================================================
// GENERATED DOMAIN TYPE
// =============================================================================

/// Type of generated domain for persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub enum GeneratedDomainTypeSnapshot {
    /// CO (Comments) - Special-Purpose domain.
    Comments,
    /// RELREC (Related Records) - Relationship domain.
    RelatedRecords,
    /// RELSPEC (Related Specimens) - Relationship domain.
    RelatedSpecimens,
    /// RELSUB (Related Subjects) - Relationship domain.
    RelatedSubjects,
}
