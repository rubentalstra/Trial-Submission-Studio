//! Study-level runtime state.
//!
//! This module contains `StudyState` which holds all domain states
//! and tracks DM domain readiness for dependency enforcement.

use super::DomainState;
use sdtm_ingest::StudyMetadata;
use std::collections::HashMap;
use std::path::PathBuf;

/// Runtime state for a loaded study.
///
/// The `dm_preview_version` field is critical for DM dependency enforcement.
/// When set, it indicates that the DM domain has a valid preview with USUBJID,
/// which unlocks access to other domains.
pub struct StudyState {
    /// Study identifier (derived from folder name)
    pub study_id: String,
    /// Path to study folder
    pub study_folder: PathBuf,
    /// State for each discovered domain
    pub domains: HashMap<String, DomainState>,
    /// Source metadata (Items.csv, CodeLists.csv)
    pub metadata: Option<StudyMetadata>,
    /// DM preview version - set when DM generates valid preview.
    ///
    /// When this is `Some`, other domains are unlocked for editing.
    /// When this is `None`, only DM can be accessed.
    pub dm_preview_version: Option<u64>,
}

impl StudyState {
    /// Create a new study state.
    pub fn new(study_folder: PathBuf, study_id: String) -> Self {
        Self {
            study_id,
            study_folder,
            domains: HashMap::new(),
            metadata: None,
            dm_preview_version: None,
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
    // DM Dependency Methods
    // ========================================================================

    /// Check if DM domain exists in this study.
    pub fn has_dm_domain(&self) -> bool {
        self.domains.contains_key("DM")
    }

    /// Check if DM domain has a valid preview (USUBJID generated).
    ///
    /// This is the primary check for unlocking other domains.
    pub fn is_dm_ready(&self) -> bool {
        self.dm_preview_version.is_some()
    }

    /// Update DM preview version when DM preview is generated.
    ///
    /// This should be called after successfully building DM's preview DataFrame.
    pub fn set_dm_ready(&mut self, version: u64) {
        self.dm_preview_version = Some(version);
    }

    // ========================================================================
    // Domain Access
    // ========================================================================

    /// Get a domain by code (no DM check - use AppState.domain() for that).
    pub fn get_domain(&self, code: &str) -> Option<&DomainState> {
        self.domains.get(code)
    }

    /// Get a mutable domain by code (no DM check).
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
        dm.derived.preview.as_ref().map(|v| &v.data)
    }
}

impl std::fmt::Debug for StudyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StudyState")
            .field("study_id", &self.study_id)
            .field("study_folder", &self.study_folder)
            .field("domain_count", &self.domains.len())
            .field("has_metadata", &self.metadata.is_some())
            .field("dm_preview_version", &self.dm_preview_version)
            .finish()
    }
}
