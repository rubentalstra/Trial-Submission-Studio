//! Controlled Terminology (CT) model per SDTM_CT_relationships.md
//!
//! This module provides a clean CT model that properly separates:
//! - **Codelist rows** (parent): `Code` = codelist NCI code, `Codelist Code` = blank
//! - **Term rows** (children): `Codelist Code` = parent, `CDISC Submission Value` = valid value
//!
//! ## CT File Structure
//!
//! Each CT CSV contains two types of rows:
//!
//! 1. **Codelist definition row** (the parent)
//!    - `Code` = the codelist's NCI code (e.g., `C66731`)
//!    - `Codelist Code` = _(blank/null)_
//!    - `Codelist Extensible (Yes/No)` = `Yes` or `No`
//!    - `CDISC Submission Value` = the codelist short name (NOT a permissible dataset value)
//!
//! 2. **Codelist term rows** (the children)
//!    - `Code` = the term's NCI concept code (e.g., `C20197`)
//!    - `Codelist Code` = parent codelist code (e.g., `C66731`)
//!    - `CDISC Submission Value` = the **permissible value in datasets**
//!    - `CDISC Synonym(s)` = alternative spellings/aliases to normalize
//!
//! ## Example: `DM.SEX` (Codelist C66731)
//!
//! ```text
//! Codelist row:  Code=C66731, Codelist Code="", Extensible=No, Name=Sex
//! Term rows:     Codelist Code=C66731, submission values: F, M, INTERSEX, U
//! ```
//!
//! ## Validation Rules
//!
//! - **Extensible=No**: Value not in allowed set = **Error**
//! - **Extensible=Yes**: Value not in allowed set = **Warning** (sponsors may extend)

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A single term within a codelist.
///
/// Per SDTM_CT_relationships.md, term rows have:
/// - `Code` = term's NCI concept code
/// - `Codelist Code` = parent codelist code
/// - `CDISC Submission Value` = the permissible dataset value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Term {
    /// NCI concept code for this term (e.g., "C20197" for Male).
    pub code: String,

    /// The permissible value in datasets (e.g., "M" for Male).
    /// This is what should appear in actual SDTM data.
    pub submission_value: String,

    /// Alternative spellings/aliases that should normalize to submission_value.
    /// Parsed from `CDISC Synonym(s)` column (semicolon-separated).
    pub synonyms: Vec<String>,

    /// Definition from `CDISC Definition` column.
    pub definition: Option<String>,

    /// NCI preferred term from `NCI Preferred Term` column.
    pub preferred_term: Option<String>,
}

/// A codelist containing multiple terms.
///
/// Per SDTM_CT_relationships.md, codelist rows have:
/// - `Code` = codelist's NCI code (e.g., "C66731")
/// - `Codelist Code` = blank (identifies this as a codelist row)
/// - `Codelist Extensible (Yes/No)` = extensibility flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Codelist {
    /// NCI code for this codelist (e.g., "C66731" for Sex).
    pub code: String,

    /// Human-readable name (e.g., "Sex", "No Yes Response").
    pub name: String,

    /// Whether sponsors can add values not in this codelist.
    /// - `false` (Non-extensible): Invalid values are **errors**
    /// - `true` (Extensible): Invalid values are **warnings**
    pub extensible: bool,

    /// Terms belonging to this codelist.
    /// Key: uppercase submission value for case-insensitive lookup.
    pub terms: BTreeMap<String, Term>,

    /// Synonym lookup: maps uppercase alias -> uppercase submission value.
    /// Built from all terms' synonyms for fast normalization.
    synonyms: BTreeMap<String, String>,
}

impl Codelist {
    /// Create a new codelist.
    pub fn new(code: String, name: String, extensible: bool) -> Self {
        Self {
            code,
            name,
            extensible,
            terms: BTreeMap::new(),
            synonyms: BTreeMap::new(),
        }
    }

    /// Add a term to this codelist.
    pub fn add_term(&mut self, term: Term) {
        let key = term.submission_value.to_uppercase();

        // Build synonym lookup
        for synonym in &term.synonyms {
            let syn_key = synonym.to_uppercase();
            if syn_key != key {
                self.synonyms.insert(syn_key, key.clone());
            }
        }

        self.terms.insert(key, term);
    }

    /// Get all valid submission values (for validation).
    pub fn submission_values(&self) -> Vec<&str> {
        self.terms
            .values()
            .map(|t| t.submission_value.as_str())
            .collect()
    }

    /// Check if a value is valid for this codelist (case-insensitive).
    pub fn is_valid(&self, value: &str) -> bool {
        let key = value.to_uppercase();
        self.terms.contains_key(&key) || self.synonyms.contains_key(&key)
    }

    /// Normalize a value to its canonical submission value.
    /// Returns the original value if not found (for extensible codelists).
    pub fn normalize(&self, value: &str) -> String {
        let key = value.to_uppercase();

        // Check if it's already a valid submission value
        if let Some(term) = self.terms.get(&key) {
            return term.submission_value.clone();
        }

        // Check if it's a synonym
        if let Some(canonical_key) = self.synonyms.get(&key)
            && let Some(term) = self.terms.get(canonical_key)
        {
            return term.submission_value.clone();
        }

        // Return original for extensible codelists
        value.to_string()
    }
}

/// A CT catalog (e.g., "SDTM CT 2024-03-29").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminologyCatalog {
    /// Display label (e.g., "SDTM CT").
    pub label: String,

    /// Release version/date (e.g., "2024-03-29").
    pub version: Option<String>,

    /// Publishing set (e.g., "SDTM", "SEND", "ADaM").
    pub publishing_set: Option<String>,

    /// Source file name.
    pub source: Option<String>,

    /// Codelists by NCI code (uppercase).
    pub codelists: BTreeMap<String, Codelist>,
}

impl TerminologyCatalog {
    /// Create a new catalog.
    pub fn new(label: String, version: Option<String>, publishing_set: Option<String>) -> Self {
        Self {
            label,
            version,
            publishing_set,
            source: None,
            codelists: BTreeMap::new(),
        }
    }

    /// Get a codelist by NCI code.
    pub fn get(&self, code: &str) -> Option<&Codelist> {
        self.codelists.get(&code.to_uppercase())
    }

    /// Add a codelist to this catalog.
    pub fn add_codelist(&mut self, codelist: Codelist) {
        self.codelists
            .insert(codelist.code.to_uppercase(), codelist);
    }
}

/// Registry of all loaded CT catalogs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerminologyRegistry {
    /// Catalogs by label (uppercase).
    pub catalogs: BTreeMap<String, TerminologyCatalog>,
}

impl TerminologyRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a catalog to the registry.
    pub fn add_catalog(&mut self, catalog: TerminologyCatalog) {
        self.catalogs.insert(catalog.label.to_uppercase(), catalog);
    }

    /// Resolve a codelist by NCI code.
    ///
    /// Searches catalogs in priority order: SDTM CT, SEND CT, others.
    /// Use `preferred` to specify catalog preference (e.g., for SEND studies).
    pub fn resolve(
        &self,
        code: &str,
        preferred: Option<&[String]>,
    ) -> Option<ResolvedCodelist<'_>> {
        let catalogs = self.catalogs_in_order(preferred);
        let key = code.to_uppercase();

        for catalog in catalogs {
            if let Some(codelist) = catalog.codelists.get(&key) {
                return Some(ResolvedCodelist { codelist, catalog });
            }
        }
        None
    }

    /// Get catalogs in priority order.
    fn catalogs_in_order(&self, preferred: Option<&[String]>) -> Vec<&TerminologyCatalog> {
        if let Some(preferred) = preferred {
            return preferred
                .iter()
                .filter_map(|label| self.catalogs.get(&label.to_uppercase()))
                .collect();
        }

        // Default order: SDTM CT first, then SEND CT, then others alphabetically
        let mut catalogs: Vec<&TerminologyCatalog> = self.catalogs.values().collect();
        catalogs.sort_by_key(|c| {
            let label = c.label.to_uppercase();
            match label.as_str() {
                "SDTM CT" => (0, label),
                "SEND CT" => (1, label),
                _ => (2, label),
            }
        });
        catalogs
    }
}

/// A resolved codelist with its source catalog.
pub struct ResolvedCodelist<'a> {
    /// Reference to the resolved codelist.
    pub codelist: &'a Codelist,
    /// Reference to the catalog containing this codelist.
    pub catalog: &'a TerminologyCatalog,
}

impl<'a> ResolvedCodelist<'a> {
    /// Get the catalog label (for reporting).
    pub fn source(&self) -> &str {
        &self.catalog.label
    }

    /// Check if a value is valid.
    pub fn is_valid(&self, value: &str) -> bool {
        self.codelist.is_valid(value)
    }

    /// Normalize a value.
    pub fn normalize(&self, value: &str) -> String {
        self.codelist.normalize(value)
    }
}
