//! CO (Comments) domain builder messages and state.
//!
//! CO is a **Special-Purpose** domain per SDTM-IG v3.4 Section 5.3.
//! It captures free-text comments that can be linked to records in other domains.

use crate::state::CommentEntry;

/// Messages for the CO (Comments) builder.
#[derive(Debug, Clone)]
pub enum CoMessage {
    /// Update subject ID field (USUBJID).
    UsubjidChanged(String),

    /// Update comment text (COVAL).
    CommentChanged(String),

    /// Update related domain (RDOMAIN).
    RdomainChanged(Option<String>),

    /// Update identifying variable name (IDVAR).
    IdvarChanged(Option<String>),

    /// Update identifying variable value (IDVARVAL).
    IdvarvalChanged(Option<String>),

    /// Update comment reference (COREF).
    CorefChanged(Option<String>),

    /// Update date/time (CODTC).
    CodtcChanged(Option<String>),

    /// Update evaluator (COEVAL).
    CoevalChanged(Option<String>),

    /// Add current entry to list.
    AddEntry,

    /// Remove entry at index.
    RemoveEntry(usize),

    /// Edit entry at index (loads into form).
    EditEntry(usize),
}

/// State for building CO (Comments) entries.
#[derive(Debug, Clone, Default)]
pub struct CoBuilderState {
    /// Current entries.
    pub entries: Vec<CommentEntry>,

    /// Current form fields.
    pub usubjid: String,
    pub comment: String,
    pub rdomain: String,
    pub idvar: String,
    pub idvarval: String,
    pub coref: String,
    pub codtc: String,
    pub coeval: String,

    /// Index of entry being edited (None = adding new).
    pub editing_index: Option<usize>,
}

impl CoBuilderState {
    /// Build entry from current form fields.
    pub fn build_entry(&self) -> Option<CommentEntry> {
        if self.usubjid.trim().is_empty() || self.comment.trim().is_empty() {
            return None;
        }

        let mut entry = CommentEntry::standalone(&self.usubjid, &self.comment);

        if !self.rdomain.trim().is_empty() {
            entry.rdomain = Some(self.rdomain.clone());
        }
        if !self.idvar.trim().is_empty() {
            entry.idvar = Some(self.idvar.clone());
        }
        if !self.idvarval.trim().is_empty() {
            entry.idvarval = Some(self.idvarval.clone());
        }
        if !self.coref.trim().is_empty() {
            entry.coref = Some(self.coref.clone());
        }
        if !self.codtc.trim().is_empty() {
            entry.codtc = Some(self.codtc.clone());
        }
        if !self.coeval.trim().is_empty() {
            entry.coeval = Some(self.coeval.clone());
        }

        Some(entry)
    }

    /// Clear form fields.
    pub fn clear_form(&mut self) {
        self.usubjid.clear();
        self.comment.clear();
        self.rdomain.clear();
        self.idvar.clear();
        self.idvarval.clear();
        self.coref.clear();
        self.codtc.clear();
        self.coeval.clear();
        self.editing_index = None;
    }

    /// Load entry into form for editing.
    pub fn load_entry(&mut self, index: usize) {
        if let Some(entry) = self.entries.get(index) {
            self.usubjid = entry.usubjid.clone();
            self.comment = entry.comment.clone();
            self.rdomain = entry.rdomain.clone().unwrap_or_default();
            self.idvar = entry.idvar.clone().unwrap_or_default();
            self.idvarval = entry.idvarval.clone().unwrap_or_default();
            self.coref = entry.coref.clone().unwrap_or_default();
            self.codtc = entry.codtc.clone().unwrap_or_default();
            self.coeval = entry.coeval.clone().unwrap_or_default();
            self.editing_index = Some(index);
        }
    }
}
