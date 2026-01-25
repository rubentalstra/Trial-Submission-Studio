//! Controlled Terminology (CT) loading.
//!
//! Loads CDISC Controlled Terminology from embedded CSV data.
//! All CT files are embedded at compile time for offline operation.

use std::io::Cursor;

use serde::Deserialize;

use super::types::{Codelist, Term, TerminologyCatalog, TerminologyRegistry};
use crate::embedded;
use crate::error::{Result, StandardsError};

// =============================================================================
// CT Version Enum
// =============================================================================

/// Controlled Terminology version.
///
/// CDISC publishes CT updates quarterly. This enum represents
/// the available versions embedded in the binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CtVersion {
    /// CT version 2024-03-29 (current production default).
    #[default]
    V2024_03_29,
    /// CT version 2025-03-28.
    V2025_03_28,
    /// CT version 2025-09-26 (latest).
    V2025_09_26,
}

impl CtVersion {
    /// Get the directory name for this version.
    pub const fn dir_name(&self) -> &'static str {
        match self {
            Self::V2024_03_29 => "2024-03-29",
            Self::V2025_03_28 => "2025-03-28",
            Self::V2025_09_26 => "2025-09-26",
        }
    }

    /// Get all available CT versions.
    pub const fn all() -> &'static [CtVersion] {
        &[Self::V2024_03_29, Self::V2025_03_28, Self::V2025_09_26]
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

/// Load a CT registry for a specific version from embedded data.
///
/// # Arguments
///
/// * `version` - The CT version to load
/// * `primary_set` - The publishing set to mark as primary (e.g., "SDTM", "SEND").
///   Pass `None` for no primary designation.
///
/// # Example
///
/// ```rust,ignore
/// use tss_standards::ct::{self, CtVersion};
///
/// // Load with SDTM CT as primary (typical for SDTM studies)
/// let registry = ct::load(CtVersion::default(), Some("SDTM"))?;
///
/// // For SEND studies, mark SEND CT as primary
/// let registry = ct::load(CtVersion::default(), Some("SEND"))?;
/// ```
/// TODO: i think we should refactor and rewrite the whole logic and every file that ahs to do with dataset and use ENUM's so we can use these ENUM's troughout the whole codebase! i think this idead is way better right? no custom strings or using SOME() //// Foundational: CDISC Foundational Standards are the basis of a complete suite of data standards, enhancing the quality, efficiency and cost effectiveness of clinical research processes from beginning to end. Foundational Standards focus on the core principles for defining data standards and include models, domains and specifications for data representation.
/// <https://www.cdisc.org/standards/foundational>
pub fn load(version: CtVersion, primary_set: Option<&str>) -> Result<TerminologyRegistry> {
    let mut registry = TerminologyRegistry::new();

    for (filename, content) in embedded::ct_files_for_version(version) {
        let mut catalog = load_catalog_from_str(content, filename)?;

        // Mark catalog as primary if it matches the requested publishing set
        if let Some(primary) = primary_set
            && let Some(ref pub_set) = catalog.publishing_set
            && pub_set.eq_ignore_ascii_case(primary)
        {
            catalog.set_primary(true);
        }

        registry.add_catalog(catalog);
    }

    Ok(registry)
}

/// Load a single CT catalog from CSV string content.
///
/// # CSV Structure
///
/// Per CDISC CT format:
/// - Codelist definition rows: `Codelist Code` is blank, `Code` is the NCI code
/// - Term rows: `Codelist Code` is the parent codelist code
pub fn load_catalog_from_str(content: &str, filename: &str) -> Result<TerminologyCatalog> {
    let cursor = Cursor::new(content.as_bytes());
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(cursor);

    let (label, version, publishing_set) = parse_ct_metadata_from_filename(filename);
    let mut catalog = TerminologyCatalog::new(label, version, publishing_set);
    catalog.source = Some(filename.to_string());

    // Collect all rows
    let mut rows: Vec<CtCsvRow> = Vec::new();
    for result in reader.deserialize::<CtCsvRow>() {
        let row = result.map_err(|e| StandardsError::CsvParse {
            file: filename.to_string(),
            message: e.to_string(),
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
    let mut orphan_term_count = 0u32;
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
            } else {
                // Log warning for orphan terms (referencing non-existent codelists)
                orphan_term_count += 1;
                if orphan_term_count <= 5 {
                    tracing::warn!(
                        file = %filename,
                        term_code = %term_code,
                        codelist_code = %codelist_code,
                        submission_value = %submission_value,
                        "Orphan term references non-existent codelist"
                    );
                }
            }
        }
    }

    // Log summary if there were orphan terms
    if orphan_term_count > 0 {
        tracing::warn!(
            file = %filename,
            orphan_count = orphan_term_count,
            "CT file contains orphan terms referencing non-existent codelists"
        );
    }

    // Third pass: check for empty codelists
    let empty_codelists: Vec<&str> = catalog
        .codelists
        .values()
        .filter(|cl| cl.terms.is_empty())
        .map(|cl| cl.code.as_str())
        .collect();

    if !empty_codelists.is_empty() {
        tracing::warn!(
            file = %filename,
            empty_count = empty_codelists.len(),
            sample_codes = ?empty_codelists.iter().take(5).collect::<Vec<_>>(),
            "CT file contains codelists with no terms"
        );
    }

    Ok(catalog)
}

/// Load only SDTM CT for a specific version from embedded data.
///
/// Returns a single catalog containing only SDTM terminology.
pub fn load_sdtm_only(version: CtVersion) -> Result<TerminologyCatalog> {
    let (filename, content) = embedded::sdtm_ct_for_version(version);
    load_catalog_from_str(content, filename)
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

/// Parse CT metadata from filename string (e.g., "SDTM_CT_2024-03-29.csv").
fn parse_ct_metadata_from_filename(filename: &str) -> (String, Option<String>, Option<String>) {
    // Strip .csv extension if present
    let stem = filename.strip_suffix(".csv").unwrap_or(filename);

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
            "CDASH" => "CDASH",
            "GLOSSARY" => "Glossary",
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
        let registry = load(CtVersion::default(), Some("SDTM")).expect("load CT");
        assert!(
            !registry.catalogs.is_empty(),
            "Registry should not be empty"
        );
    }

    #[test]
    fn test_load_ct_latest() {
        let registry = load(CtVersion::latest(), Some("SDTM")).expect("load CT");
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
        assert_eq!(CtVersion::V2025_03_28.dir_name(), "2025-03-28");
        assert_eq!(CtVersion::V2025_09_26.dir_name(), "2025-09-26");
    }

    #[test]
    fn test_ct_version_all() {
        let all = CtVersion::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&CtVersion::V2024_03_29));
        assert!(all.contains(&CtVersion::V2025_03_28));
        assert!(all.contains(&CtVersion::V2025_09_26));
    }

    #[test]
    fn test_resolve_codelist() {
        let registry = load(CtVersion::default(), Some("SDTM")).expect("load CT");

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

    #[test]
    fn test_submission_value_validation() {
        let registry = load(CtVersion::default(), Some("SDTM")).expect("load CT");

        // C66742 is NY (Yes/No) - non-extensible
        // Valid submission values: Y, N
        assert!(registry.validate_submission_value("C66742", "Y").is_none());
        assert!(registry.validate_submission_value("C66742", "N").is_none());

        // "YES" is a synonym, NOT a valid submission value
        let issue = registry.validate_submission_value("C66742", "YES");
        assert!(
            issue.is_some(),
            "YES should fail - it's a synonym, not a submission value"
        );
    }

    #[test]
    fn test_primary_catalog_marking() {
        // Load with SDTM as primary
        let registry = load(CtVersion::default(), Some("SDTM")).expect("load CT");

        // SDTM CT should be marked as primary
        let sdtm_catalog = registry.catalogs.get("SDTM CT");
        assert!(sdtm_catalog.is_some(), "Should have SDTM CT catalog");
        assert!(
            sdtm_catalog.unwrap().primary,
            "SDTM CT should be marked as primary"
        );

        // SEND CT should NOT be primary (if it exists)
        if let Some(send_catalog) = registry.catalogs.get("SEND CT") {
            assert!(
                !send_catalog.primary,
                "SEND CT should NOT be primary by default"
            );
        }
    }

    #[test]
    fn test_load_with_send_primary() {
        // Load with SEND as primary
        let registry = load(CtVersion::default(), Some("SEND")).expect("load CT");

        // SEND CT should be marked as primary
        if let Some(send_catalog) = registry.catalogs.get("SEND CT") {
            assert!(
                send_catalog.primary,
                "SEND CT should be marked as primary when requested"
            );
        }

        // SDTM CT should NOT be primary
        if let Some(sdtm_catalog) = registry.catalogs.get("SDTM CT") {
            assert!(
                !sdtm_catalog.primary,
                "SDTM CT should NOT be primary when SEND is requested"
            );
        }
    }

    #[test]
    fn test_resolved_from_primary() {
        let registry = load(CtVersion::default(), Some("SDTM")).expect("load CT");

        // Resolve a codelist - should come from SDTM CT (primary)
        let resolved = registry.resolve("C66742", None);
        assert!(resolved.is_some());
        assert!(
            resolved.unwrap().from_primary(),
            "Resolution should come from primary catalog"
        );
    }
}
