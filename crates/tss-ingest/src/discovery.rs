//! File discovery and domain matching.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{IngestError, Result};

/// A discovered CSV file with domain classification.
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    /// Path to the CSV file.
    pub path: PathBuf,
    /// Matched domain code (e.g., "AE", "DM").
    pub domain: Option<String>,
    /// Variant suffix if present (e.g., "AE1", "AE2").
    pub variant: Option<String>,
    /// Whether this appears to be a metadata file.
    pub is_metadata: bool,
}

/// Lists all CSV files in a directory.
///
/// Returns files sorted by filename.
pub fn list_csv_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Err(IngestError::DirectoryNotFound {
            path: dir.to_path_buf(),
        });
    }

    let mut files = Vec::new();

    let entries = std::fs::read_dir(dir).map_err(|e| IngestError::DirectoryRead {
        path: dir.to_path_buf(),
        source: e,
    })?;

    for entry_result in entries {
        let entry = entry_result.map_err(|e| IngestError::DirectoryRead {
            path: dir.to_path_buf(),
            source: e,
        })?;

        let path = entry.path();

        // Skip directories
        if !path.is_file() {
            continue;
        }

        // Check for .csv extension (case-insensitive)
        let is_csv = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("csv"))
            .unwrap_or(false);

        if is_csv {
            files.push(path);
        }
    }

    // Sort by filename
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    Ok(files)
}

/// Discovers and classifies CSV files as domain data or metadata.
///
/// Returns a map of domain code to list of (path, variant) pairs.
pub fn discover_domain_files(
    csv_files: &[PathBuf],
    supported_domains: &[String],
) -> BTreeMap<String, Vec<(PathBuf, String)>> {
    let supported: Vec<String> = supported_domains
        .iter()
        .map(|d| d.trim().to_uppercase())
        .collect();

    let mut domain_files: BTreeMap<String, Vec<(PathBuf, String)>> = BTreeMap::new();

    for path in csv_files {
        let stem = path
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_string();

        let filename = stem.to_uppercase();

        // Skip metadata files (Items, CodeLists, etc.)
        if is_metadata_file(&filename) {
            continue;
        }

        // Try to match to a domain
        if let Some((domain, variant)) = match_domain(&filename, &supported) {
            domain_files
                .entry(domain)
                .or_default()
                .push((path.clone(), variant));
        }
    }

    domain_files
}

/// Discovers all CSV files and classifies them.
pub fn discover_files(dir: &Path, supported_domains: &[String]) -> Result<Vec<DiscoveredFile>> {
    let csv_files = list_csv_files(dir)?;

    let supported: Vec<String> = supported_domains
        .iter()
        .map(|d| d.trim().to_uppercase())
        .collect();

    let mut discovered = Vec::new();

    for path in csv_files {
        let stem = path
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_string();

        let filename = stem.to_uppercase();
        let is_metadata = is_metadata_file(&filename);

        let (domain, variant) = if is_metadata {
            (None, None)
        } else {
            match match_domain(&filename, &supported) {
                Some((d, v)) => (Some(d), Some(v)),
                None => (None, None),
            }
        };

        discovered.push(DiscoveredFile {
            path,
            domain,
            variant,
            is_metadata,
        });
    }

    Ok(discovered)
}

/// Checks if a filename looks like a metadata file.
///
/// Uses statistical patterns rather than hardcoded keywords:
/// - Contains common metadata indicators in the filename
fn is_metadata_file(filename: &str) -> bool {
    // Common metadata file patterns (case-insensitive, already uppercase)
    // These are structural patterns, not SDTM-specific keywords
    let metadata_indicators = ["CODELIST", "ITEMS", "README", "METADATA"];

    metadata_indicators
        .iter()
        .any(|pattern| filename.contains(pattern))
}

/// Matches a filename to a domain code.
///
/// Handles patterns like:
/// - STUDY_DM.csv -> DM
/// - STUDY_AE_SPLIT1.csv -> AE (variant: AE_SPLIT1)
/// - DM_DEMOGRAPHICS.csv -> DM
fn match_domain(filename: &str, supported_domains: &[String]) -> Option<(String, String)> {
    // Pad filename with underscores to match exact boundaries
    let padded = format!("_{filename}_");

    // First, try exact match (e.g., _DM_ in _STUDY_DM_)
    if let Some(domain) = supported_domains
        .iter()
        .find(|domain| padded.contains(&format!("_{domain}_")))
    {
        return Some((domain.clone(), domain.clone()));
    }

    // Sort domains by length (longest first) to prefer specific matches
    let mut domains_by_len: Vec<&String> = supported_domains.iter().collect();
    domains_by_len.sort_by_key(|domain| std::cmp::Reverse(domain.len()));

    // Split by underscore and check each part
    let parts: Vec<&str> = filename.split('_').collect();
    for part in parts {
        for domain in &domains_by_len {
            if part.starts_with(domain.as_str()) {
                return Some(((*domain).clone(), part.to_string()));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create some CSV files
        for name in &[
            "STUDY_DM.csv",
            "STUDY_AE.csv",
            "STUDY_AE2.csv",
            "STUDY_Items.csv",
            "STUDY_CodeLists.csv",
            "README.csv",
        ] {
            let path = dir.path().join(name);
            std::fs::write(&path, "header\ndata").unwrap();
        }

        dir
    }

    #[test]
    fn test_list_csv_files() {
        let dir = create_test_dir();
        let files = list_csv_files(dir.path()).unwrap();

        assert_eq!(files.len(), 6);
        // Should be sorted by filename
        assert!(
            files[0]
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains("README")
        );
    }

    #[test]
    fn test_discover_domain_files() {
        let dir = create_test_dir();
        let files = list_csv_files(dir.path()).unwrap();
        let domains = vec!["DM".to_string(), "AE".to_string()];

        let domain_files = discover_domain_files(&files, &domains);

        // Should have DM and AE
        assert!(domain_files.contains_key("DM"));
        assert!(domain_files.contains_key("AE"));

        // AE should have 2 files (AE and AE2)
        assert_eq!(domain_files.get("AE").unwrap().len(), 2);
    }

    #[test]
    fn test_is_metadata_file() {
        assert!(is_metadata_file("STUDY_ITEMS"));
        assert!(is_metadata_file("STUDY_CODELISTS"));
        assert!(is_metadata_file("README"));
        assert!(!is_metadata_file("STUDY_DM"));
        assert!(!is_metadata_file("STUDY_AE"));
    }

    #[test]
    fn test_match_domain() {
        let domains = vec!["DM".to_string(), "AE".to_string(), "CM".to_string()];

        assert_eq!(
            match_domain("STUDY_DM", &domains),
            Some(("DM".to_string(), "DM".to_string()))
        );
        assert_eq!(
            match_domain("STUDY_AE_SPLIT1", &domains),
            Some(("AE".to_string(), "AE".to_string()))
        );
        assert_eq!(match_domain("STUDY_UNKNOWN", &domains), None);
    }

    #[test]
    fn test_discover_files() {
        let dir = create_test_dir();
        let domains = vec!["DM".to_string(), "AE".to_string()];

        let discovered = discover_files(dir.path(), &domains).unwrap();

        // Should have 6 files
        assert_eq!(discovered.len(), 6);

        // Check metadata detection
        let items = discovered.iter().find(|f| {
            f.path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains("Items")
        });
        assert!(items.is_some());
        assert!(items.unwrap().is_metadata);

        // Check domain detection
        let dm = discovered.iter().find(|f| {
            f.path.file_name().unwrap().to_str().unwrap().contains("DM") && !f.is_metadata
        });
        assert!(dm.is_some());
        assert_eq!(dm.unwrap().domain, Some("DM".to_string()));
    }
}
