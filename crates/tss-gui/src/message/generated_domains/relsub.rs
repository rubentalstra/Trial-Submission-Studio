//! RELSUB (Related Subjects) domain builder messages and state.
//!
//! RELSUB is a **Relationship** domain per SDTM-IG v3.4 Section 8.7.
//! It tracks subject-to-subject relationships (e.g., mother/child, twins).
//!
//! **IMPORTANT**: Per SDTM-IG, RELSUB relationships MUST be bidirectional.
//! When A→B is defined, B→A must also exist with the reciprocal relationship.

use crate::state::RelsubEntry;

/// Messages for the RELSUB (Related Subjects) builder.
#[derive(Debug, Clone)]
pub enum RelsubMessage {
    /// Update subject ID (USUBJID).
    UsubjidChanged(String),

    /// Update related subject ID (RSUBJID).
    RsubjidChanged(String),

    /// Update subject relationship (SREL) from codelist.
    SrelChanged(String),

    /// Add current entry to list.
    AddEntry,

    /// Remove entry at index.
    RemoveEntry(usize),
}

/// State for building RELSUB (Related Subjects) entries.
#[derive(Debug, Clone, Default)]
pub struct RelsubBuilderState {
    /// Current entries.
    pub entries: Vec<RelsubEntry>,

    /// Current form fields.
    pub usubjid: String,
    pub rsubjid: String,
    pub srel: String,
}

impl RelsubBuilderState {
    /// Build entry from current form fields.
    pub fn build_entry(&self) -> Option<RelsubEntry> {
        if self.usubjid.trim().is_empty()
            || self.rsubjid.trim().is_empty()
            || self.srel.trim().is_empty()
        {
            return None;
        }

        Some(RelsubEntry::new(&self.usubjid, &self.rsubjid, &self.srel))
    }

    /// Clear form fields.
    pub fn clear_form(&mut self) {
        self.usubjid.clear();
        self.rsubjid.clear();
        self.srel.clear();
    }
}
