//! Study state - collection of domains for a clinical study.
//!
//! A study represents a loaded folder containing CSV files for SDTM domains.

use std::collections::BTreeMap;
use std::path::PathBuf;

use tss_ingest::StudyMetadata;

use super::domain::Domain;

// =============================================================================
// STUDY STATE
// =============================================================================

/// A loaded clinical study with its domains.
///
/// # Design Notes
///
/// - **BTreeMap for domains** - Maintains alphabetical order by domain code.
/// - **Study ID from folder name** - Simple convention, can be overridden.
/// - **No caching** - All derived data computed on demand.
///
/// # Example
///
/// ```ignore
/// let study = Study::from_folder(PathBuf::from("/path/to/study"));
///
/// // Add domains as they're loaded
/// study.add_domain("DM", domain);
/// study.add_domain("AE", domain);
///
/// // Access domains
/// if let Some(dm) = study.domain("DM") {
///     println!("DM has {} rows", dm.row_count());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Study {
    /// Study identifier (derived from folder name).
    pub study_id: String,

    /// Path to the study folder.
    pub study_folder: PathBuf,

    /// Study metadata (Items.csv, CodeLists.csv) if available.
    pub metadata: Option<StudyMetadata>,

    /// Domains indexed by code (e.g., "DM", "AE", "LB").
    domains: BTreeMap<String, Domain>,
}

impl Study {
    /// Create a new study from a folder path.
    ///
    /// The study ID is derived from the folder name.
    pub fn from_folder(folder: PathBuf) -> Self {
        let study_id = folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self {
            study_id,
            study_folder: folder,
            metadata: None,
            domains: BTreeMap::new(),
        }
    }

    /// Create a study with a custom ID.
    pub fn new(study_id: impl Into<String>, folder: PathBuf) -> Self {
        Self {
            study_id: study_id.into(),
            study_folder: folder,
            metadata: None,
            domains: BTreeMap::new(),
        }
    }

    // =========================================================================
    // DOMAIN ACCESS
    // =========================================================================

    /// Get a domain by code.
    #[inline]
    pub fn domain(&self, code: &str) -> Option<&Domain> {
        self.domains.get(code)
    }

    /// Get a mutable reference to a domain.
    ///
    /// Note: In Iced, prefer updating domain through messages in `update()`.
    /// This is provided for use within the `update()` function only.
    #[inline]
    pub fn domain_mut(&mut self, code: &str) -> Option<&mut Domain> {
        self.domains.get_mut(code)
    }

    /// Check if a domain exists.
    #[inline]
    pub fn has_domain(&self, code: &str) -> bool {
        self.domains.contains_key(code)
    }

    /// Add a domain to the study.
    pub fn add_domain(&mut self, code: impl Into<String>, domain: Domain) {
        self.domains.insert(code.into(), domain);
    }

    /// Remove a domain from the study.
    pub fn remove_domain(&mut self, code: &str) -> Option<Domain> {
        self.domains.remove(code)
    }

    /// Get all domain codes.
    pub fn domain_codes(&self) -> Vec<&str> {
        self.domains.keys().map(String::as_str).collect()
    }

    /// Get domain codes with DM first, then alphabetical.
    ///
    /// DM (Demographics) is the primary domain and should appear first.
    pub fn domain_codes_dm_first(&self) -> Vec<&str> {
        let mut codes: Vec<_> = self.domain_codes();
        codes.sort_by(|a, b| {
            // DM comes first
            match (*a, *b) {
                ("DM", _) => std::cmp::Ordering::Less,
                (_, "DM") => std::cmp::Ordering::Greater,
                _ => a.cmp(b),
            }
        });
        codes
    }

    /// Get number of domains.
    #[inline]
    pub fn domain_count(&self) -> usize {
        self.domains.len()
    }

    /// Iterate over all domains.
    pub fn domains(&self) -> impl Iterator<Item = (&str, &Domain)> {
        self.domains.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Iterate over all domains mutably.
    pub fn domains_mut(&mut self) -> impl Iterator<Item = (&str, &mut Domain)> {
        self.domains.iter_mut().map(|(k, v)| (k.as_str(), v))
    }

    // =========================================================================
    // COMPUTED PROPERTIES
    // =========================================================================

    /// Count total rows across all domains.
    pub fn total_rows(&self) -> usize {
        self.domains.values().map(|d| d.row_count()).sum()
    }

    /// Check if all domains have complete mappings.
    pub fn is_all_complete(&self) -> bool {
        self.domains.values().all(|d| d.is_mapping_complete())
    }

    /// Get mapping progress as (complete, total).
    pub fn mapping_progress(&self) -> (usize, usize) {
        let complete = self
            .domains
            .values()
            .filter(|d| d.is_mapping_complete())
            .count();
        (complete, self.domains.len())
    }
}
