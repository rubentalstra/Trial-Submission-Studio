//! Controlled Terminology (CT) model per CDISC CT standards.
//!
//! This module provides types for working with CDISC Controlled Terminology,
//! which defines the permissible values for SDTM variables.
//!
//! # CT File Structure
//!
//! Each CT CSV contains two types of rows:
//!
//! 1. **Codelist definition row** (parent)
//!    - `Code` = the codelist's NCI code (e.g., `C66731`)
//!    - `Codelist Code` = blank (identifies this as a codelist row)
//!    - `Codelist Extensible` = `Yes` or `No`
//!
//! 2. **Term rows** (children)
//!    - `Code` = the term's NCI concept code (e.g., `C20197`)
//!    - `Codelist Code` = parent codelist code (e.g., `C66731`)
//!    - `CDISC Submission Value` = the permissible value in datasets
//!
//! # Example: `DM.SEX` (Codelist C66731)
//!
//! ```text
//! Codelist row:  Code=C66731, Extensible=No, Name=Sex
//! Term rows:     submission values: F, M, INTERSEX, U
//! ```
//!
//! # Validation Rules
//!
//! - **Extensible=No**: Value not in codelist = **Error**
//! - **Extensible=Yes**: Value not in codelist = **Warning**

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single term within a codelist.
///
/// Term rows in CT CSVs have:
/// - `Code`: Term's NCI concept code
/// - `Codelist Code`: Parent codelist code
/// - `CDISC Submission Value`: The permissible dataset value
///
/// # Example
///
/// ```
/// use tss_model::Term;
///
/// let term = Term {
///     code: "C20197".to_string(),
///     submission_value: "M".to_string(),
///     synonyms: vec!["MALE".to_string()],
///     definition: Some("Male gender".to_string()),
///     preferred_term: Some("Male".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Term {
    /// NCI concept code for this term (e.g., "C20197" for Male).
    pub code: String,

    /// The permissible value in datasets (e.g., "M" for Male).
    pub submission_value: String,

    /// Alternative spellings/aliases that normalize to this term.
    pub synonyms: Vec<String>,

    /// Definition from CDISC.
    pub definition: Option<String>,

    /// NCI preferred term.
    pub preferred_term: Option<String>,
}

/// A codelist containing permissible terms.
///
/// Codelist rows in CT CSVs have:
/// - `Code`: The codelist's NCI code (e.g., "C66731")
/// - `Codelist Code`: Blank (identifies this as a codelist row)
/// - `Codelist Extensible`: Whether sponsors can extend
///
/// # Example
///
/// ```
/// use tss_model::{Codelist, Term};
///
/// let mut codelist = Codelist::new(
///     "C66731".to_string(),
///     "Sex".to_string(),
///     false, // non-extensible
/// );
///
/// codelist.add_term(Term {
///     code: "C20197".to_string(),
///     submission_value: "M".to_string(),
///     synonyms: vec!["MALE".to_string()],
///     definition: None,
///     preferred_term: None,
/// });
///
/// assert!(codelist.is_valid("M"));
/// assert!(codelist.is_valid("MALE")); // synonym
/// assert_eq!(codelist.normalize("male"), "M");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Codelist {
    /// NCI code for this codelist (e.g., "C66731" for Sex).
    pub code: String,

    /// Human-readable name (e.g., "Sex").
    pub name: String,

    /// Whether sponsors can add values not in this codelist.
    /// - `false`: Invalid values are errors
    /// - `true`: Invalid values are warnings
    pub extensible: bool,

    /// Terms indexed by uppercase submission value.
    pub terms: BTreeMap<String, Term>,

    /// Synonym lookup: uppercase alias -> uppercase submission value.
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
        for synonym in &term.synonyms {
            let syn_key = synonym.to_uppercase();
            if syn_key != key {
                self.synonyms.insert(syn_key, key.clone());
            }
        }
        self.terms.insert(key, term);
    }

    /// Get all valid submission values.
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
    ///
    /// Returns the original value if not found (for extensible codelists).
    pub fn normalize(&self, value: &str) -> String {
        let key = value.to_uppercase();
        if let Some(term) = self.terms.get(&key) {
            return term.submission_value.clone();
        }
        if let Some(canonical_key) = self.synonyms.get(&key)
            && let Some(term) = self.terms.get(canonical_key)
        {
            return term.submission_value.clone();
        }
        value.to_string()
    }
}

/// A CT catalog representing a specific CT release.
///
/// For example: "SDTM CT 2024-03-29"
///
/// # Example
///
/// ```
/// use tss_model::TerminologyCatalog;
///
/// let catalog = TerminologyCatalog::new(
///     "SDTM CT".to_string(),
///     Some("2024-03-29".to_string()),
///     Some("SDTM".to_string()),
/// );
/// ```
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
///
/// Supports resolving codelists across multiple catalogs with
/// configurable priority order.
///
/// # Example
///
/// ```
/// use tss_model::{TerminologyRegistry, TerminologyCatalog};
///
/// let mut registry = TerminologyRegistry::new();
/// registry.add_catalog(TerminologyCatalog::new(
///     "SDTM CT".to_string(),
///     Some("2024-03-29".to_string()),
///     Some("SDTM".to_string()),
/// ));
///
/// // Resolve a codelist by NCI code
/// if let Some(resolved) = registry.resolve("C66731", None) {
///     println!("Found codelist: {}", resolved.codelist.name);
/// }
/// ```
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
    /// Searches catalogs in priority order:
    /// 1. Preferred catalogs (if specified)
    /// 2. SDTM CT
    /// 3. SEND CT
    /// 4. Others alphabetically
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

    fn catalogs_in_order(&self, preferred: Option<&[String]>) -> Vec<&TerminologyCatalog> {
        if let Some(preferred) = preferred {
            return preferred
                .iter()
                .filter_map(|label| self.catalogs.get(&label.to_uppercase()))
                .collect();
        }
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
///
/// Provides access to both the codelist and information about
/// which catalog it came from.
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
