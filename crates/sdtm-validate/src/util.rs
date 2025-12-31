//! Utility types for validation.

use std::collections::HashMap;

/// A set that performs case-insensitive lookups but preserves original names.
///
/// Used for matching SDTM variable names which should be case-insensitive.
#[derive(Debug, Clone, Default)]
pub struct CaseInsensitiveSet {
    /// Maps uppercase name -> original name
    inner: HashMap<String, String>,
}

impl CaseInsensitiveSet {
    /// Create an empty set.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Create a set from an iterator of strings.
    pub fn from_iter<I, S>(iter: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            inner: iter
                .into_iter()
                .map(|s| (s.as_ref().to_uppercase(), s.as_ref().to_string()))
                .collect(),
        }
    }

    /// Insert a value into the set.
    pub fn insert(&mut self, value: impl AsRef<str>) {
        let s = value.as_ref();
        self.inner.insert(s.to_uppercase(), s.to_string());
    }

    /// Check if the set contains a value (case-insensitive).
    pub fn contains(&self, value: impl AsRef<str>) -> bool {
        self.inner.contains_key(&value.as_ref().to_uppercase())
    }

    /// Get the original column name for a variable (case-insensitive lookup).
    pub fn get(&self, value: impl AsRef<str>) -> Option<&str> {
        self.inner.get(&value.as_ref().to_uppercase()).map(|s| s.as_str())
    }

    /// Number of elements in the set.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if set is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_insensitive_set() {
        let set = CaseInsensitiveSet::from_iter(["USUBJID", "STUDYID", "Domain"]);

        assert!(set.contains("USUBJID"));
        assert!(set.contains("usubjid"));
        assert!(set.contains("Usubjid"));
        assert!(set.contains("domain"));
        assert!(set.contains("DOMAIN"));
        assert!(!set.contains("OTHER"));
    }
}
