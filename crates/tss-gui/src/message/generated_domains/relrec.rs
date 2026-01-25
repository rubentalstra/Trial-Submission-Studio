//! RELREC (Related Records) domain builder messages and state.
//!
//! RELREC is a **Relationship** domain per SDTM-IG v3.4 Section 8.5.
//! It links records across or within domains using a relationship identifier.

use crate::state::RelrecEntry;

/// Messages for the RELREC (Related Records) builder.
#[derive(Debug, Clone)]
pub enum RelrecMessage {
    /// Update relationship ID (RELID).
    RelidChanged(String),

    /// Update subject ID (USUBJID) - optional for dataset-level relationships.
    UsubjidChanged(Option<String>),

    /// Update related domain (RDOMAIN).
    RdomainChanged(String),

    /// Update identifying variable (IDVAR).
    IdvarChanged(String),

    /// Update identifying variable value (IDVARVAL).
    IdvarvalChanged(Option<String>),

    /// Update relationship type (RELTYPE) - "ONE" or "MANY".
    ReltypeChanged(Option<String>),

    /// Add current entry to list.
    AddEntry,

    /// Remove entry at index.
    RemoveEntry(usize),
}

/// State for building RELREC (Related Records) entries.
#[derive(Debug, Clone, Default)]
pub struct RelrecBuilderState {
    /// Current entries.
    pub entries: Vec<RelrecEntry>,

    /// Current form fields.
    pub relid: String,
    pub usubjid: String,
    pub rdomain: String,
    pub idvar: String,
    pub idvarval: String,
    pub reltype: String,
}

impl RelrecBuilderState {
    /// Build entry from current form fields.
    pub fn build_entry(&self) -> Option<RelrecEntry> {
        if self.relid.trim().is_empty()
            || self.rdomain.trim().is_empty()
            || self.idvar.trim().is_empty()
        {
            return None;
        }

        let reltype = match self.reltype.trim().to_uppercase().as_str() {
            "ONE" => Some(crate::state::RelrecRelType::One),
            "MANY" => Some(crate::state::RelrecRelType::Many),
            _ => None,
        };

        Some(RelrecEntry {
            relid: self.relid.clone(),
            usubjid: if self.usubjid.trim().is_empty() {
                None
            } else {
                Some(self.usubjid.clone())
            },
            rdomain: self.rdomain.clone(),
            idvar: self.idvar.clone(),
            idvarval: if self.idvarval.trim().is_empty() {
                None
            } else {
                Some(self.idvarval.clone())
            },
            reltype,
        })
    }

    /// Clear form fields.
    pub fn clear_form(&mut self) {
        self.relid.clear();
        self.usubjid.clear();
        self.rdomain.clear();
        self.idvar.clear();
        self.idvarval.clear();
        self.reltype.clear();
    }
}
