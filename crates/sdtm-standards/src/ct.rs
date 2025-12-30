//! Controlled Terminology (CT) loading.
//!
//! Loads CDISC Controlled Terminology from CSV files in the
//! `standards/ct/` directory. Supports multiple CT versions.

use std::path::Path;

use serde::Deserialize;

use sdtm_model::ct::{Codelist, Term, TerminologyCatalog, TerminologyRegistry};

use crate::error::{Result, StandardsError};
use crate::paths::ct_path;

// =============================================================================
// CT Version Enum
// =============================================================================

/// Controlled Terminology version.
///
/// CDISC publishes CT updates quarterly. This enum represents
/// the available versions in the standards directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CtVersion {
    /// CT version 2024-03-29 (current production default).
    #[default]
    V2024_03_29,
    /// CT version 2025-09-26 (latest).
    V2025_09_26,
}

impl CtVersion {
    /// Get the directory name for this version.
    pub const fn dir_name(&self) -> &'static str {
        match self {
            Self::V2024_03_29 => "2024-03-29",
            Self::V2025_09_26 => "2025-09-26",
        }
    }

    /// Get all available CT versions.
    pub const fn all() -> &'static [CtVersion] {
        &[Self::V2024_03_29, Self::V2025_09_26]
    }

    /// Get the latest CT version.
    pub const fn latest() -> Self {
        Self::V2025_09_26
    }
}

impl std::fmt::Display for CtVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dir_name())
    }
}

// =============================================================================
// Public Loading Functions
// =============================================================================

/// Load a CT registry for a specific version.
///
/// # Arguments
///
/// * `version` - The CT version to load
///
/// # Example
///
/// ```rust,ignore
/// use sdtm_standards::ct::{self, CtVersion};
///
/// let registry = ct::load(CtVersion::default())?;
/// let ny = registry.resolve("NY", None);
/// ```
pub fn load(version: CtVersion) -> Result<TerminologyRegistry> {
    let dir = ct_path(version.dir_name());
    if !dir.exists() {
        return Err(StandardsError::DirectoryNotFound { path: dir });
    }
    load_from(&dir)
}

/// Load a CT registry from a custom directory.
///
/// Scans the directory for `*_CT_*.csv` files and loads all of them.
pub fn load_from(dir: &Path) -> Result<TerminologyRegistry> {
    if !dir.exists() {
        return Err(StandardsError::DirectoryNotFound {
            path: dir.to_path_buf(),
        });
    }

    let mut registry = TerminologyRegistry::new();

    let entries = std::fs::read_dir(dir).map_err(|_| StandardsError::DirectoryNotFound {
        path: dir.to_path_buf(),
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if !name.contains("_CT_") || !name.ends_with(".csv") {
            continue;
        }

        let catalog = load_catalog(&path)?;
        registry.add_catalog(catalog);
    }

    Ok(registry)
}

/// Load a single CT catalog from a CSV file.
///
/// # CSV Structure
///
/// Per CDISC CT format:
/// - Codelist definition rows: `Codelist Code` is blank, `Code` is the NCI code
/// - Term rows: `Codelist Code` is the parent codelist code
pub fn load_catalog(path: &Path) -> Result<TerminologyCatalog> {
    if !path.exists() {
        return Err(StandardsError::FileNotFound {
            path: path.to_path_buf(),
        });
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(|e| StandardsError::CsvRead {
            path: path.to_path_buf(),
            source: e,
        })?;

    let (label, version, publishing_set) = parse_ct_metadata(path);
    let mut catalog = TerminologyCatalog::new(label, version, publishing_set);
    catalog.source = path.file_name().and_then(|v| v.to_str()).map(String::from);

    // Collect all rows
    let mut rows: Vec<CtCsvRow> = Vec::new();
    for result in reader.deserialize::<CtCsvRow>() {
        let row = result.map_err(|e| StandardsError::CsvRead {
            path: path.to_path_buf(),
            source: e,
        })?;
        rows.push(row);
    }

    // First pass: collect codelist definition rows
    for row in &rows {
        let code = row.code.trim();
        let codelist_code = row.codelist_code.trim();

        // Codelist definition row: Codelist Code is blank
        if codelist_code.is_empty() && !code.is_empty() {
            let name = row.codelist_name.trim().to_string();
            let extensible = row.extensible.trim().eq_ignore_ascii_case("yes");

            let codelist = Codelist::new(code.to_string(), name, extensible);
            catalog.add_codelist(codelist);
        }
    }

    // Second pass: collect term rows
    for row in &rows {
        let term_code = row.code.trim();
        let codelist_code = row.codelist_code.trim();
        let submission_value = row.submission_value.trim();

        // Term row: Codelist Code is NOT blank
        if !codelist_code.is_empty() && !submission_value.is_empty() {
            let synonyms = parse_synonyms(&row.synonyms);
            let definition = non_empty(&row.definition);
            let preferred_term = non_empty(&row.preferred_term);

            let term = Term {
                code: term_code.to_string(),
                submission_value: submission_value.to_string(),
                synonyms,
                definition,
                preferred_term,
            };

            // Add term to its parent codelist
            if let Some(codelist) = catalog.codelists.get_mut(&codelist_code.to_uppercase()) {
                codelist.add_term(term);
            }
        }
    }

    Ok(catalog)
}

/// Load only SDTM CT for a specific version.
///
/// Returns a single catalog containing only SDTM terminology.
pub fn load_sdtm_only(version: CtVersion) -> Result<TerminologyCatalog> {
    let dir = ct_path(version.dir_name());
    if !dir.exists() {
        return Err(StandardsError::DirectoryNotFound { path: dir });
    }

    // Find the SDTM CT file
    let entries = std::fs::read_dir(&dir)
        .map_err(|_| StandardsError::DirectoryNotFound { path: dir.clone() })?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if name.starts_with("SDTM_CT_") && name.ends_with(".csv") {
            return load_catalog(&path);
        }
    }

    Err(StandardsError::FileNotFound {
        path: dir.join("SDTM_CT_*.csv"),
    })
}

// =============================================================================
// CSV Row Type
// =============================================================================

/// Row from CT CSV files.
#[derive(Debug, Deserialize)]
struct CtCsvRow {
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Codelist Code")]
    codelist_code: String,
    #[serde(rename = "Codelist Extensible (Yes/No)")]
    extensible: String,
    #[serde(rename = "Codelist Name")]
    codelist_name: String,
    #[serde(rename = "CDISC Submission Value")]
    submission_value: String,
    #[serde(rename = "CDISC Synonym(s)")]
    synonyms: String,
    #[serde(rename = "CDISC Definition")]
    definition: String,
    #[serde(rename = "NCI Preferred Term")]
    preferred_term: String,
}

// =============================================================================
// Helpers
// =============================================================================

/// Parse CT metadata from filename (e.g., "SDTM_CT_2024-03-29.csv").
fn parse_ct_metadata(path: &Path) -> (String, Option<String>, Option<String>) {
    let stem = path.file_stem().and_then(|v| v.to_str()).unwrap_or("");

    // Pattern: PREFIX_CT_YYYY-MM-DD
    if let Some((prefix, date)) = stem.split_once("_CT_") {
        let publishing_set = match prefix.to_uppercase().as_str() {
            "SDTM" => "SDTM",
            "SEND" => "SEND",
            "ADAM" => "ADaM",
            "DEFINE-XML" | "DEFINEXML" => "DEFINE-XML",
            "PROTOCOL" => "Protocol",
            "DDF" => "DDF",
            "MRCT" => "MRCT",
            _ => prefix,
        };

        let label = format!("{} CT", publishing_set);
        let version = if date.is_empty() {
            None
        } else {
            Some(date.to_string())
        };
        let publishing_set = Some(publishing_set.to_string());

        return (label, version, publishing_set);
    }

    (stem.to_string(), None, None)
}

/// Parse semicolon-separated synonyms.
fn parse_synonyms(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    trimmed
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Return Some(value) if non-empty, None otherwise.
fn non_empty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_ct_default() {
        let registry = load(CtVersion::default()).expect("load CT");
        assert!(
            !registry.catalogs.is_empty(),
            "Registry should not be empty"
        );
    }

    #[test]
    fn test_load_ct_latest() {
        let registry = load(CtVersion::latest()).expect("load CT");
        assert!(
            !registry.catalogs.is_empty(),
            "Registry should not be empty"
        );
    }

    #[test]
    fn test_load_sdtm_ct_only() {
        let catalog = load_sdtm_only(CtVersion::default()).expect("load SDTM CT");
        assert!(!catalog.codelists.is_empty(), "Should have codelists");

        // Check for common codelists - C66742 is the NY (Yes/No) codelist
        assert!(
            catalog.codelists.contains_key("C66742"),
            "Should have NY codelist (C66742)"
        );
    }

    #[test]
    fn test_ct_version_dir_names() {
        assert_eq!(CtVersion::V2024_03_29.dir_name(), "2024-03-29");
        assert_eq!(CtVersion::V2025_09_26.dir_name(), "2025-09-26");
    }

    #[test]
    fn test_ct_version_all() {
        let all = CtVersion::all();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&CtVersion::V2024_03_29));
        assert!(all.contains(&CtVersion::V2025_09_26));
    }

    #[test]
    fn test_resolve_codelist() {
        let registry = load(CtVersion::default()).expect("load CT");

        // Test resolving NY codelist by NCI code C66742
        let resolved = registry.resolve("C66742", None);
        assert!(resolved.is_some(), "Should resolve NY codelist (C66742)");

        let codelist = resolved.unwrap().codelist;
        assert!(!codelist.terms.is_empty(), "NY should have terms");

        // Check submission values
        let values = codelist.submission_values();
        assert!(values.contains(&"Y"), "NY should contain Y");
        assert!(values.contains(&"N"), "NY should contain N");
    }
}
