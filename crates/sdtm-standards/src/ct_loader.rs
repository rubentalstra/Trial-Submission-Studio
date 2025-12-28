//! Clean CT loader per SDTM_CT_relationships.md
//!
//! This module loads CT files into the new clean model (`sdtm_model::ct`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use csv::ReaderBuilder;

use sdtm_model::ct::{Codelist, Term, TerminologyCatalog, TerminologyRegistry};

const DEFAULT_CT_VERSION: &str = "2024-03-29";

/// Get the default standards root directory.
fn default_standards_root() -> PathBuf {
    if let Ok(root) = std::env::var("CDISC_STANDARDS_DIR") {
        return PathBuf::from(root);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards")
}

/// Load the default CT registry (SDTM CT 2024-03-29).
pub fn load_default_ct_registry() -> Result<TerminologyRegistry> {
    let root = default_standards_root();
    let mut ct_dirs = Vec::new();
    ct_dirs.push(root.join("ct").join(DEFAULT_CT_VERSION));

    // Also check docs/Controlled_Terminology
    let docs_ct = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Controlled_Terminology")
        .join(DEFAULT_CT_VERSION);
    ct_dirs.push(docs_ct);

    load_ct_registry(&ct_dirs)
}

/// Load a CT registry from one or more directories.
pub fn load_ct_registry(ct_dirs: &[PathBuf]) -> Result<TerminologyRegistry> {
    let mut registry = TerminologyRegistry::new();

    for dir in ct_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
            if !name.contains("_CT_") || !name.ends_with(".csv") {
                continue;
            }
            let catalog = load_ct_catalog(&path)?;
            registry.add_catalog(catalog);
        }
    }

    Ok(registry)
}

/// Load a single CT catalog from a CSV file.
///
/// Per SDTM_CT_relationships.md:
/// - Codelist rows: `Codelist Code` is blank, `Code` is the codelist NCI code
/// - Term rows: `Codelist Code` is the parent codelist code
pub fn load_ct_catalog(path: &Path) -> Result<TerminologyCatalog> {
    let rows = read_csv_rows(path)?;
    let (label, version, publishing_set) = parse_ct_metadata(path);

    let mut catalog = TerminologyCatalog::new(label, version, publishing_set);
    catalog.source = path.file_name().and_then(|v| v.to_str()).map(String::from);

    // First pass: collect codelist definition rows
    for row in &rows {
        let code = get_field(row, "Code");
        let codelist_code = get_field(row, "Codelist Code");

        // Codelist definition row: Codelist Code is blank
        if codelist_code.is_empty() && !code.is_empty() {
            let name = get_field(row, "Codelist Name");
            let extensible =
                get_field(row, "Codelist Extensible (Yes/No)").eq_ignore_ascii_case("yes");

            let codelist = Codelist::new(code.clone(), name, extensible);
            catalog.add_codelist(codelist);
        }
    }

    // Second pass: collect term rows
    for row in &rows {
        let term_code = get_field(row, "Code");
        let codelist_code = get_field(row, "Codelist Code");
        let submission_value = get_field(row, "CDISC Submission Value");

        // Term row: Codelist Code is NOT blank
        if !codelist_code.is_empty() && !submission_value.is_empty() {
            let synonyms = parse_synonyms(&get_field(row, "CDISC Synonym(s)"));
            let definition = get_optional(row, "CDISC Definition");
            let preferred_term = get_optional(row, "NCI Preferred Term");

            let term = Term {
                code: term_code,
                submission_value,
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

fn read_csv_rows(path: &Path) -> Result<Vec<BTreeMap<String, String>>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .with_context(|| format!("read csv: {}", path.display()))?;

    let headers = reader
        .headers()
        .with_context(|| format!("read headers: {}", path.display()))?
        .clone();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.with_context(|| format!("read record: {}", path.display()))?;
        let mut row = BTreeMap::new();
        for (idx, value) in record.iter().enumerate() {
            let key = headers
                .get(idx)
                .unwrap_or("")
                .trim_matches('\u{feff}')
                .to_string();
            row.insert(key, value.trim().to_string());
        }
        rows.push(row);
    }
    Ok(rows)
}

fn get_field(row: &BTreeMap<String, String>, key: &str) -> String {
    row.get(key).cloned().unwrap_or_default()
}

fn get_optional(row: &BTreeMap<String, String>, key: &str) -> Option<String> {
    row.get(key).filter(|v| !v.is_empty()).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_ct_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/Controlled_Terminology/2024-03-29")
    }

    #[test]
    fn test_load_sdtm_ct() {
        let path = test_ct_dir().join("SDTM_CT_2024-03-29.csv");
        if !path.exists() {
            return; // Skip if CT file not available
        }

        let catalog = load_ct_catalog(&path).unwrap();
        assert_eq!(catalog.label, "SDTM CT");
        assert_eq!(catalog.version, Some("2024-03-29".to_string()));
        assert_eq!(catalog.publishing_set, Some("SDTM".to_string()));

        // Check SEX codelist (C66731)
        let sex = catalog.get("C66731").expect("Sex codelist should exist");
        assert_eq!(sex.code, "C66731");
        assert_eq!(sex.name, "Sex");
        assert!(!sex.extensible);

        // Check terms
        assert!(sex.is_valid("F"));
        assert!(sex.is_valid("M"));
        assert!(sex.is_valid("U"));
        assert!(sex.is_valid("INTERSEX"));

        // Check synonym normalization
        assert_eq!(sex.normalize("Female"), "F");
        assert_eq!(sex.normalize("Male"), "M");
        assert_eq!(sex.normalize("UNK"), "U");
        assert_eq!(sex.normalize("Unknown"), "U");
    }

    #[test]
    fn test_extensible_codelist() {
        let path = test_ct_dir().join("SDTM_CT_2024-03-29.csv");
        if !path.exists() {
            return;
        }

        let catalog = load_ct_catalog(&path).unwrap();

        // Unit codelist (C71620) is extensible
        let unit = catalog.get("C71620").expect("Unit codelist should exist");
        assert!(unit.extensible);
    }
}
