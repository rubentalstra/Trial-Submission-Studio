//! Utility types for case-insensitive lookups.
//!
//! Provides data structures for efficient case-insensitive string matching,
//! commonly needed when comparing SDTM variable names and values.

use std::collections::HashMap;

/// A set that performs case-insensitive lookups while preserving original case.
///
/// Useful for SDTM variable name matching where "USUBJID", "usubjid", and
/// "Usubjid" should all match, but the original casing should be preserved
/// in output.
///
/// # Example
///
/// ```rust
/// use sdtm_model::CaseInsensitiveSet;
///
/// let set = CaseInsensitiveSet::new(["USUBJID", "STUDYID"]);
/// assert_eq!(set.get("usubjid"), Some("USUBJID"));
/// assert!(set.contains("studyid"));
/// ```
#[derive(Debug, Clone)]
pub struct CaseInsensitiveSet {
    map: HashMap<String, String>,
}

impl CaseInsensitiveSet {
    /// Create a new set from an iterator of names.
    ///
    /// The first occurrence of each name (case-insensitively) is preserved.
    pub fn new<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut map = HashMap::new();
        for name in names {
            let name = name.as_ref();
            let key = name.to_ascii_uppercase();
            map.entry(key).or_insert_with(|| name.to_string());
        }
        Self { map }
    }

    /// Get the original-cased name for a case-insensitive lookup.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.map
            .get(&name.to_ascii_uppercase())
            .map(|value| value.as_str())
    }

    /// Check if a name exists (case-insensitive).
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(&name.to_ascii_uppercase())
    }
}
