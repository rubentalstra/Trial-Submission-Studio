//! Study state - collection of domains for a clinical study.
//!
//! A study represents a loaded folder containing CSV files for SDTM domains.

use std::collections::BTreeMap;
use std::path::PathBuf;

use tss_ingest::StudyMetadata;

use super::domain_state::DomainState;

// =============================================================================
// STUDY ID EXTRACTION
// =============================================================================

/// Extract the study ID from a folder name, stripping any timestamp suffix.
///
/// Folder names often include a timestamp suffix in the format `_YYYYMMDD_HHMMSS`
/// (e.g., `DEMO_GDISC_20240903_072908`). This function extracts just the study ID
/// portion (e.g., `DEMO_GDISC`).
///
/// # Examples
///
/// ```ignore
/// assert_eq!(extract_study_id("DEMO_GDISC_20240903_072908"), "DEMO_GDISC");
/// assert_eq!(extract_study_id("STUDY_ABC"), "STUDY_ABC"); // No timestamp
/// assert_eq!(extract_study_id("TEST_20231225_143022"), "TEST"); // With timestamp
/// ```
fn extract_study_id(folder_name: &str) -> String {
    // Pattern: name ends with _YYYYMMDD_HHMMSS (8 digits + 6 digits)
    // Total suffix length: 1 + 8 + 1 + 6 = 16 characters
    const TIMESTAMP_SUFFIX_LEN: usize = 16; // "_YYYYMMDD_HHMMSS"

    if folder_name.len() > TIMESTAMP_SUFFIX_LEN {
        let potential_suffix = &folder_name[folder_name.len() - TIMESTAMP_SUFFIX_LEN..];

        // Check if it matches the pattern _XXXXXXXX_XXXXXX where X are digits
        if potential_suffix.starts_with('_')
            && potential_suffix.chars().nth(9) == Some('_')
            && potential_suffix[1..9].chars().all(|c| c.is_ascii_digit())
            && potential_suffix[10..].chars().all(|c| c.is_ascii_digit())
        {
            return folder_name[..folder_name.len() - TIMESTAMP_SUFFIX_LEN].to_string();
        }
    }

    // No timestamp suffix found, return as-is
    folder_name.to_string()
}

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

    /// DomainStates indexed by code (e.g., "DM", "AE", "LB").
    domains: BTreeMap<String, DomainState>,
}

impl Study {
    /// Create a new study from a folder path.
    ///
    /// The study ID is derived from the folder name, with any timestamp suffix
    /// (format `_YYYYMMDD_HHMMSS`) stripped. For example, folder name
    /// `DEMO_GDISC_20240903_072908` becomes study ID `DEMO_GDISC`.
    pub fn from_folder(folder: PathBuf) -> Self {
        let folder_name = folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        let study_id = extract_study_id(folder_name);

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
    pub fn domain(&self, code: &str) -> Option<&DomainState> {
        self.domains.get(code)
    }

    /// Get a mutable reference to a domain.
    ///
    /// Note: In Iced, prefer updating domain through messages in `update()`.
    /// This is provided for use within the `update()` function only.
    #[inline]
    pub fn domain_mut(&mut self, code: &str) -> Option<&mut DomainState> {
        self.domains.get_mut(code)
    }

    /// Check if a domain exists.
    #[inline]
    pub fn has_domain(&self, code: &str) -> bool {
        self.domains.contains_key(code)
    }

    /// Add a domain to the study.
    pub fn add_domain(&mut self, code: impl Into<String>, domain: DomainState) {
        self.domains.insert(code.into(), domain);
    }

    /// Remove a domain from the study.
    pub fn remove_domain(&mut self, code: &str) -> Option<DomainState> {
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
    pub fn domains(&self) -> impl Iterator<Item = (&str, &DomainState)> {
        self.domains.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Iterate over all domains mutably.
    pub fn domains_mut(&mut self) -> impl Iterator<Item = (&str, &mut DomainState)> {
        self.domains.iter_mut().map(|(k, v)| (k.as_str(), v))
    }

    // =========================================================================
    // COMPUTED PROPERTIES
    // =========================================================================

    /// Count total rows across all domains.
    pub fn total_rows(&self) -> usize {
        self.domains.values().map(DomainState::row_count).sum()
    }

    /// Check if all domains have complete mappings.
    pub fn is_all_complete(&self) -> bool {
        self.domains.values().all(DomainState::is_mapping_complete)
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

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_study_id_with_timestamp() {
        // Standard case: folder name with timestamp suffix
        assert_eq!(extract_study_id("DEMO_GDISC_20240903_072908"), "DEMO_GDISC");
    }

    #[test]
    fn test_extract_study_id_without_timestamp() {
        // No timestamp suffix - should return as-is
        assert_eq!(extract_study_id("STUDY_ABC"), "STUDY_ABC");
        assert_eq!(extract_study_id("DEMO_GDISC"), "DEMO_GDISC");
    }

    #[test]
    fn test_extract_study_id_short_name() {
        // Name shorter than timestamp suffix - should return as-is
        assert_eq!(extract_study_id("ABC"), "ABC");
        assert_eq!(extract_study_id("X"), "X");
    }

    #[test]
    fn test_extract_study_id_various_timestamps() {
        // Various valid timestamps
        assert_eq!(extract_study_id("TEST_20231225_143022"), "TEST");
        assert_eq!(extract_study_id("STUDY_A_20200101_000000"), "STUDY_A");
        assert_eq!(
            extract_study_id("MY_TRIAL_99_20991231_235959"),
            "MY_TRIAL_99"
        );
    }

    #[test]
    fn test_extract_study_id_invalid_timestamp_patterns() {
        // These look like timestamps but aren't valid - should return as-is
        assert_eq!(
            extract_study_id("STUDY_2024090_0729088"), // Wrong digit counts
            "STUDY_2024090_0729088"
        );
        assert_eq!(
            extract_study_id("STUDY_ABCDEFGH_IJKLMN"), // Letters instead of digits
            "STUDY_ABCDEFGH_IJKLMN"
        );
    }

    #[test]
    fn test_study_from_folder_extracts_id() {
        let study = Study::from_folder(PathBuf::from("/path/to/DEMO_GDISC_20240903_072908"));
        assert_eq!(study.study_id, "DEMO_GDISC");
    }

    #[test]
    fn test_study_from_folder_no_timestamp() {
        let study = Study::from_folder(PathBuf::from("/path/to/SIMPLE_STUDY"));
        assert_eq!(study.study_id, "SIMPLE_STUDY");
    }
}
