//! RELSPEC (Related Specimens) domain entry type.
//!
//! RELSPEC is a **Relationship** domain per SDTM-IG v3.4 Section 8.8.
//! It tracks specimen parent/child relationships and hierarchy levels.

/// Entry for RELSPEC (Related Specimens) domain.
///
/// Per SDTM-IG v3.4 Section 8.8:
/// - Tracks specimen parent/child hierarchy
/// - LEVEL=1 for collected samples (PARENT is null)
/// - LEVEL is calculated automatically from hierarchy depth
#[derive(Debug, Clone, PartialEq)]
pub struct RelspecEntry {
    /// Subject identifier (USUBJID).
    pub usubjid: String,

    /// Specimen identifier (REFID).
    /// Unique within USUBJID.
    pub refid: String,

    /// Specimen type (SPEC).
    /// e.g., "SERUM", "PLASMA", "URINE", "TISSUE".
    pub spec: Option<String>,

    /// Parent specimen identifier (PARENT).
    /// REFID of the parent specimen. Null for collected samples (LEVEL=1).
    pub parent: Option<String>,
}

impl RelspecEntry {
    /// Create a collected specimen entry (LEVEL=1, no parent).
    pub fn collected(
        usubjid: impl Into<String>,
        refid: impl Into<String>,
        spec: Option<String>,
    ) -> Self {
        Self {
            usubjid: usubjid.into(),
            refid: refid.into(),
            spec,
            parent: None,
        }
    }

    /// Create a derived specimen entry (has parent).
    pub fn derived(
        usubjid: impl Into<String>,
        refid: impl Into<String>,
        parent: impl Into<String>,
        spec: Option<String>,
    ) -> Self {
        Self {
            usubjid: usubjid.into(),
            refid: refid.into(),
            spec,
            parent: Some(parent.into()),
        }
    }
}
