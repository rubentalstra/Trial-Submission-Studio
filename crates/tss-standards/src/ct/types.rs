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
//! - **Extensible=Yes**: Value not in codelist = **No issue** (custom values allowed per CDISC)
//!
//! # Important: Submission Value vs Synonyms
//!
//! - Only **CDISC Submission Value** is valid for regulatory submission!
//! - **Synonyms** are for **mapping help only** - they should NOT be accepted in final datasets.
//! - During normalization, synonyms should be converted to their proper submission values.

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
/// use tss_standards::Term;
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
    /// This is the ONLY valid value for regulatory submission.
    pub submission_value: String,

    /// Alternative spellings/aliases that can be mapped to this term.
    /// These are for mapping assistance only - NOT valid for submission.
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
/// use tss_standards::{Codelist, Term};
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
/// assert!(codelist.is_valid_submission_value("M"));
/// assert!(!codelist.is_valid_submission_value("MALE")); // Synonym, not submission value!
/// assert_eq!(codelist.find_submission_value("male"), Some("M"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Codelist {
    /// NCI code for this codelist (e.g., "C66731" for Sex).
    pub code: String,

    /// Human-readable name (e.g., "Sex").
    pub name: String,

    /// Whether sponsors can add values not in this codelist.
    /// - `false`: Invalid values are errors
    /// - `true`: Custom values are allowed (no issue)
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

    /// Check if a value is a valid CDISC Submission Value (case-insensitive).
    ///
    /// **Important:** This only checks submission values, NOT synonyms.
    /// Only CDISC Submission Values are valid for regulatory submission.
    pub fn is_valid_submission_value(&self, value: &str) -> bool {
        let key = value.to_uppercase();
        self.terms.contains_key(&key)
    }

    /// Check if a value matches any term (submission value OR synonym).
    ///
    /// Use this for mapping/normalization purposes, NOT for validation.
    pub fn matches_any(&self, value: &str) -> bool {
        let key = value.to_uppercase();
        self.terms.contains_key(&key) || self.synonyms.contains_key(&key)
    }

    /// Find the correct CDISC Submission Value for an input value.
    ///
    /// Returns the submission value if the input is:
    /// - Already a valid submission value (returns itself)
    /// - A synonym (returns the corresponding submission value)
    ///
    /// Use this during normalization to convert sponsor terms to CDISC terms.
    pub fn find_submission_value(&self, value: &str) -> Option<&str> {
        let key = value.to_uppercase();

        // Check if it's already a submission value
        if let Some(term) = self.terms.get(&key) {
            return Some(&term.submission_value);
        }

        // Check if it's a synonym
        if let Some(canonical_key) = self.synonyms.get(&key)
            && let Some(term) = self.terms.get(canonical_key)
        {
            return Some(&term.submission_value);
        }

        None
    }

    /// Normalize a value to its canonical submission value.
    ///
    /// Returns the original value if not found (for extensible codelists).
    #[deprecated(note = "Use find_submission_value() for explicit handling")]
    pub fn normalize(&self, value: &str) -> String {
        self.find_submission_value(value)
            .map(ToString::to_string)
            .unwrap_or_else(|| value.to_string())
    }

    /// Legacy method for backwards compatibility.
    #[deprecated(note = "Use is_valid_submission_value() or matches_any()")]
    pub fn is_valid(&self, value: &str) -> bool {
        self.matches_any(value)
    }
}

/// A CT catalog representing a specific CT release.
///
/// For example: "SDTM CT 2024-03-29"
///
/// # Example
///
/// ```
/// use tss_standards::TerminologyCatalog;
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
/// use tss_standards::{TerminologyRegistry, TerminologyCatalog};
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

    /// Validate a value against a codelist.
    ///
    /// Returns `None` if valid, or an error message if invalid.
    ///
    /// **Important:** Only CDISC Submission Value is valid for submission!
    /// Synonyms are for mapping help only.
    pub fn validate_submission_value(
        &self,
        codelist_code: &str,
        value: &str,
    ) -> Option<CtValidationIssue> {
        let resolved = self.resolve(codelist_code, None)?;

        // Check ONLY submission_value - synonyms are for mapping, not submission!
        // If valid OR the codelist is extensible (custom values allowed), no issue
        if resolved.codelist.is_valid_submission_value(value) || resolved.codelist.extensible {
            None
        } else {
            Some(CtValidationIssue {
                codelist_code: codelist_code.to_string(),
                codelist_name: resolved.codelist.name.clone(),
                invalid_value: value.to_string(),
                valid_values: resolved
                    .codelist
                    .submission_values()
                    .into_iter()
                    .map(String::from)
                    .collect(),
            })
        }
    }

    /// Find the correct submission value for any input (submission value or synonym).
    ///
    /// Used during normalization to convert sponsor terms to CDISC terms.
    pub fn find_submission_value(&self, codelist_code: &str, input: &str) -> Option<String> {
        let resolved = self.resolve(codelist_code, None)?;
        resolved
            .codelist
            .find_submission_value(input)
            .map(String::from)
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

    /// Check if a value is a valid submission value.
    pub fn is_valid_submission_value(&self, value: &str) -> bool {
        self.codelist.is_valid_submission_value(value)
    }

    /// Find the submission value for an input.
    pub fn find_submission_value(&self, value: &str) -> Option<&str> {
        self.codelist.find_submission_value(value)
    }
}

/// CT validation issue returned when a value fails validation.
#[derive(Debug, Clone)]
pub struct CtValidationIssue {
    /// The codelist code that was checked.
    pub codelist_code: String,
    /// The codelist name.
    pub codelist_name: String,
    /// The invalid value that was found.
    pub invalid_value: String,
    /// The valid submission values for this codelist.
    pub valid_values: Vec<String>,
}

impl std::fmt::Display for CtValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Value '{}' is not a valid CDISC Submission Value for codelist {} ({})",
            self.invalid_value, self.codelist_name, self.codelist_code
        )
    }
}
