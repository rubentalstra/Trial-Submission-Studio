//! Clean CT loader per SDTM_CT_relationships.md
//!
//! This module loads CT files into the new clean model (`sdtm_model::ct`).
//! The default registry is cached using `OnceLock` to avoid repeated file I/O.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Result;

use sdtm_model::ct::{Codelist, Term, TerminologyCatalog, TerminologyRegistry};

use crate::csv_utils::{default_standards_root, get_field, get_optional, read_csv_rows};

const DEFAULT_CT_VERSION: &str = "2024-03-29";

/// Cached default CT registry.
static DEFAULT_CT_REGISTRY: OnceLock<TerminologyRegistry> = OnceLock::new();

/// Load the default CT registry (SDTM CT 2024-03-29).
///
/// The registry is cached on first load and subsequent calls return a clone.
/// Use [`load_ct_registry`] directly if you need to load from custom paths.
pub fn load_default_ct_registry() -> Result<TerminologyRegistry> {
    // Return cached registry if available
    if let Some(registry) = DEFAULT_CT_REGISTRY.get() {
        return Ok(registry.clone());
    }

    // Load and cache the registry
    let registry = load_default_ct_registry_uncached()?;

    // Try to cache it (ignore if another thread beat us)
    let _ = DEFAULT_CT_REGISTRY.set(registry.clone());

    Ok(registry)
}

/// Load the default CT registry without caching.
fn load_default_ct_registry_uncached() -> Result<TerminologyRegistry> {
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
