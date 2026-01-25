//! RELSUB (Related Subjects) domain entry type.
//!
//! RELSUB is a **Relationship** domain per SDTM-IG v3.4 Section 8.7.
//! It tracks subject-to-subject relationships (e.g., mother/child, twins).
//!
//! **IMPORTANT**: Per SDTM-IG, RELSUB relationships MUST be bidirectional.
//! When A→B is defined, B→A must also exist with the reciprocal relationship.

/// Entry for RELSUB (Related Subjects) domain.
///
/// Per SDTM-IG v3.4 Section 8.7:
/// - Represents relationships between study subjects
/// - **MUST be bidirectional**: if A to B exists, B to A must also exist
/// - The generation service handles auto-creating reciprocal relationships
#[derive(Debug, Clone, PartialEq)]
pub struct RelsubEntry {
    /// Subject identifier (USUBJID).
    pub usubjid: String,

    /// Related subject identifier (RSUBJID).
    /// Must be a USUBJID present in the DM domain.
    pub rsubjid: String,

    /// Subject relationship (SREL).
    /// From RELSUB codelist (e.g., "MOTHER, BIOLOGICAL", "TWIN, DIZYGOTIC").
    pub srel: String,
}

impl RelsubEntry {
    /// Create a new subject relationship entry.
    ///
    /// Note: The generation service will automatically create the reciprocal
    /// relationship (e.g., if you create MOTHER to CHILD, CHILD to MOTHER is auto-added).
    pub fn new(
        usubjid: impl Into<String>,
        rsubjid: impl Into<String>,
        srel: impl Into<String>,
    ) -> Self {
        Self {
            usubjid: usubjid.into(),
            rsubjid: rsubjid.into(),
            srel: srel.into(),
        }
    }
}
