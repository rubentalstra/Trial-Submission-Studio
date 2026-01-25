//! RELSPEC (Related Specimens) domain builder messages and state.
//!
//! RELSPEC is a **Relationship** domain per SDTM-IG v3.4 Section 8.6.
//! It tracks specimen parent/child relationships and hierarchy levels.

use crate::state::RelspecEntry;

/// Messages for the RELSPEC (Related Specimens) builder.
#[derive(Debug, Clone)]
pub enum RelspecMessage {
    /// Update subject ID (USUBJID).
    UsubjidChanged(String),

    /// Update specimen reference ID (REFID).
    RefidChanged(String),

    /// Update specimen type (SPEC).
    SpecChanged(Option<String>),

    /// Update parent specimen reference (PARENT).
    ParentChanged(Option<String>),

    /// Add current entry to list.
    AddEntry,

    /// Remove entry at index.
    RemoveEntry(usize),
}

/// State for building RELSPEC (Related Specimens) entries.
#[derive(Debug, Clone, Default)]
pub struct RelspecBuilderState {
    /// Current entries.
    pub entries: Vec<RelspecEntry>,

    /// Current form fields.
    pub usubjid: String,
    pub refid: String,
    pub spec: String,
    pub parent: String,
}

impl RelspecBuilderState {
    /// Build entry from current form fields.
    pub fn build_entry(&self) -> Option<RelspecEntry> {
        if self.usubjid.trim().is_empty() || self.refid.trim().is_empty() {
            return None;
        }

        Some(RelspecEntry {
            usubjid: self.usubjid.clone(),
            refid: self.refid.clone(),
            spec: if self.spec.trim().is_empty() {
                None
            } else {
                Some(self.spec.clone())
            },
            parent: if self.parent.trim().is_empty() {
                None
            } else {
                Some(self.parent.clone())
            },
        })
    }

    /// Clear form fields.
    pub fn clear_form(&mut self) {
        self.usubjid.clear();
        self.refid.clear();
        self.spec.clear();
        self.parent.clear();
    }
}
