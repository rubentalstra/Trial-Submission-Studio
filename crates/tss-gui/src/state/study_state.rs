//! Study-level runtime state.
//!
//! This module contains `StudyState` which holds all domain states.

use super::DomainState;
use std::collections::HashMap;
use std::path::PathBuf;
use tss_ingest::StudyMetadata;

/// Runtime state for a loaded study.
#[derive(Clone)]
pub struct StudyState {
    /// Study identifier (derived from folder name)
    pub study_id: String,
    /// Path to study folder
    pub study_folder: PathBuf,
    /// State for each discovered domain
    pub domains: HashMap<String, DomainState>,
    /// Source metadata (Items.csv, CodeLists.csv)
    pub metadata: Option<StudyMetadata>,
}

impl StudyState {
    /// Create a new study state.
    pub fn new(study_folder: PathBuf, study_id: String) -> Self {
        Self {
            study_id,
            study_folder,
            domains: HashMap::new(),
            metadata: None,
        }
    }

    /// Create from a folder path, deriving study_id from folder name.
    pub fn from_folder(study_folder: PathBuf) -> Self {
        let study_id = study_folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Study")
            .to_string();

        Self::new(study_folder, study_id)
    }

    // ========================================================================
    // Domain Access
    // ========================================================================

    /// Get a domain by code.
    pub fn get_domain(&self, code: &str) -> Option<&DomainState> {
        self.domains.get(code)
    }

    /// Get a mutable domain by code.
    pub fn get_domain_mut(&mut self, code: &str) -> Option<&mut DomainState> {
        self.domains.get_mut(code)
    }

    /// Get all domain codes sorted alphabetically.
    pub fn domain_codes(&self) -> Vec<&str> {
        let mut codes: Vec<&str> = self.domains.keys().map(String::as_str).collect();
        codes.sort();
        codes
    }

    /// Get domain codes with DM always first (if present).
    pub fn domain_codes_dm_first(&self) -> Vec<&str> {
        let mut codes = self.domain_codes();
        if let Some(dm_pos) = codes.iter().position(|&c| c == "DM") {
            codes.remove(dm_pos);
            codes.insert(0, "DM");
        }
        codes
    }

    /// Check if a domain exists.
    pub fn has_domain(&self, code: &str) -> bool {
        self.domains.contains_key(code)
    }

    /// Add a domain to the study.
    pub fn add_domain(&mut self, code: String, domain: DomainState) {
        self.domains.insert(code, domain);
    }

    // ========================================================================
    // Metadata Access
    // ========================================================================

    /// Get the label for a source column from Items.csv metadata.
    pub fn column_label(&self, column_id: &str) -> Option<&str> {
        self.metadata
            .as_ref()?
            .items
            .get(&column_id.to_uppercase())
            .map(|item| item.label.as_str())
    }

    /// Get DM's preview DataFrame if available.
    ///
    /// Used for passing DM data to other domains (for RFSTDTC reference).
    pub fn dm_preview_data(&self) -> Option<&polars::prelude::DataFrame> {
        let dm = self.domains.get("DM")?;
        dm.derived.preview.as_ref()
    }
}

impl std::fmt::Debug for StudyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StudyState")
            .field("study_id", &self.study_id)
            .field("study_folder", &self.study_folder)
            .field("domain_count", &self.domains.len())
            .field("has_metadata", &self.metadata.is_some())
            .finish()
    }
}
