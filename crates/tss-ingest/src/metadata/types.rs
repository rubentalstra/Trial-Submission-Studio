//! Core metadata types.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Definition of a source column from study metadata (Items.csv).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceColumn {
    /// Column identifier/name as it appears in source data.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Data type (e.g., "text", "integer", "float", "date").
    pub data_type: Option<String>,
    /// Whether the column is mandatory in the source.
    pub mandatory: bool,
    /// Name of the associated codelist/format.
    pub format_name: Option<String>,
    /// Maximum content length (for text fields).
    pub content_length: Option<usize>,
}

impl SourceColumn {
    /// Creates a new SourceColumn with required fields.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            data_type: None,
            mandatory: false,
            format_name: None,
            content_length: None,
        }
    }

    /// Sets the data type.
    pub fn with_data_type(mut self, data_type: impl Into<String>) -> Self {
        self.data_type = Some(data_type.into());
        self
    }

    /// Sets whether the column is mandatory.
    pub fn with_mandatory(mut self, mandatory: bool) -> Self {
        self.mandatory = mandatory;
        self
    }

    /// Sets the format/codelist name.
    pub fn with_format(mut self, format_name: impl Into<String>) -> Self {
        self.format_name = Some(format_name.into());
        self
    }

    /// Sets the content length.
    pub fn with_length(mut self, length: usize) -> Self {
        self.content_length = Some(length);
        self
    }
}

/// A study-specific codelist mapping coded values to decoded text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StudyCodelist {
    /// Name of the format/codelist.
    pub name: String,
    /// Primary lookup: exact match.
    values: BTreeMap<String, String>,
    /// Case-insensitive lookup (uppercase keys).
    #[serde(skip)]
    values_upper: BTreeMap<String, String>,
    /// Numeric-normalized lookup (e.g., "1.0" -> "1").
    #[serde(skip)]
    values_numeric: BTreeMap<String, String>,
}

impl StudyCodelist {
    /// Creates a new empty codelist.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            values: BTreeMap::new(),
            values_upper: BTreeMap::new(),
            values_numeric: BTreeMap::new(),
        }
    }

    /// Inserts a code-decode pair.
    pub fn insert(&mut self, code: &str, decode: &str) {
        let code_str = code.to_string();
        let decode_str = decode.to_string();

        // Primary lookup
        self.values.insert(code_str.clone(), decode_str.clone());

        // Case-insensitive lookup
        self.values_upper
            .insert(code_str.to_uppercase(), decode_str.clone());

        // Numeric-normalized lookup
        if let Some(normalized) = normalize_numeric_key(&code_str) {
            self.values_numeric.insert(normalized, decode_str);
        }
    }

    /// Looks up a decode value for a given code.
    ///
    /// Tries exact match first, then case-insensitive, then numeric-normalized.
    pub fn lookup(&self, raw: &str) -> Option<&str> {
        // Exact match
        if let Some(decode) = self.values.get(raw) {
            return Some(decode);
        }

        // Case-insensitive match
        if let Some(decode) = self.values_upper.get(&raw.to_uppercase()) {
            return Some(decode);
        }

        // Numeric-normalized match
        if let Some(normalized) = normalize_numeric_key(raw)
            && let Some(decode) = self.values_numeric.get(&normalized)
        {
            return Some(decode);
        }

        None
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if the codelist is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Iterates over all code-decode pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.values.iter()
    }

    /// Rebuilds the lookup indexes after deserialization.
    pub fn rebuild_indexes(&mut self) {
        self.values_upper.clear();
        self.values_numeric.clear();

        for (code, decode) in &self.values {
            self.values_upper
                .insert(code.to_uppercase(), decode.clone());
            if let Some(normalized) = normalize_numeric_key(code) {
                self.values_numeric.insert(normalized, decode.clone());
            }
        }
    }
}

/// Normalizes a string as a numeric key (removes trailing zeros, decimal points).
fn normalize_numeric_key(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try to parse as f64
    let num: f64 = trimmed.parse().ok()?;

    // Format without trailing zeros
    let formatted = format!("{num}");
    let normalized = formatted.trim_end_matches('0').trim_end_matches('.');

    if normalized.is_empty() {
        Some("0".to_string())
    } else {
        Some(normalized.to_string())
    }
}

/// Collection of study metadata loaded from Items.csv and CodeLists.csv.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StudyMetadata {
    /// Source column definitions keyed by uppercase column ID.
    pub items: BTreeMap<String, SourceColumn>,
    /// Codelists keyed by uppercase format name.
    pub codelists: BTreeMap<String, StudyCodelist>,
}

impl StudyMetadata {
    /// Creates an empty StudyMetadata.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a source column definition.
    pub fn add_item(&mut self, item: SourceColumn) {
        self.items.insert(item.id.to_uppercase(), item);
    }

    /// Adds a codelist.
    pub fn add_codelist(&mut self, codelist: StudyCodelist) {
        self.codelists
            .insert(codelist.name.to_uppercase(), codelist);
    }

    /// Gets a source column by ID (case-insensitive).
    pub fn get_item(&self, id: &str) -> Option<&SourceColumn> {
        self.items.get(&id.to_uppercase())
    }

    /// Gets a codelist by name (case-insensitive).
    pub fn get_codelist(&self, name: &str) -> Option<&StudyCodelist> {
        self.codelists.get(&name.to_uppercase())
    }

    /// Returns true if there are no items and no codelists.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.codelists.is_empty()
    }

    /// Rebuilds all codelist indexes after deserialization.
    pub fn rebuild_indexes(&mut self) {
        for codelist in self.codelists.values_mut() {
            codelist.rebuild_indexes();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_column_builder() {
        let col = SourceColumn::new("AGE", "Age in Years")
            .with_data_type("integer")
            .with_mandatory(true)
            .with_length(3);

        assert_eq!(col.id, "AGE");
        assert_eq!(col.label, "Age in Years");
        assert_eq!(col.data_type, Some("integer".to_string()));
        assert!(col.mandatory);
        assert_eq!(col.content_length, Some(3));
    }

    #[test]
    fn test_codelist_lookup() {
        let mut cl = StudyCodelist::new("SEX");
        cl.insert("M", "Male");
        cl.insert("F", "Female");

        assert_eq!(cl.lookup("M"), Some("Male"));
        assert_eq!(cl.lookup("m"), Some("Male")); // case-insensitive
        assert_eq!(cl.lookup("X"), None);
    }

    #[test]
    fn test_codelist_numeric_lookup() {
        let mut cl = StudyCodelist::new("RACE");
        cl.insert("1", "Asian");
        cl.insert("2", "Black");

        assert_eq!(cl.lookup("1"), Some("Asian"));
        assert_eq!(cl.lookup("1.0"), Some("Asian")); // numeric normalization
        assert_eq!(cl.lookup("1.00"), Some("Asian"));
    }

    #[test]
    fn test_study_metadata() {
        let mut meta = StudyMetadata::new();

        meta.add_item(SourceColumn::new("AGE", "Age"));
        meta.add_codelist(StudyCodelist::new("SEX"));

        assert!(meta.get_item("age").is_some()); // case-insensitive
        assert!(meta.get_item("AGE").is_some());
        assert!(meta.get_codelist("sex").is_some());
    }

    #[test]
    fn test_normalize_numeric_key() {
        assert_eq!(normalize_numeric_key("1"), Some("1".to_string()));
        assert_eq!(normalize_numeric_key("1.0"), Some("1".to_string()));
        assert_eq!(normalize_numeric_key("1.50"), Some("1.5".to_string()));
        assert_eq!(normalize_numeric_key("0"), Some("0".to_string()));
        assert_eq!(normalize_numeric_key("0.0"), Some("0".to_string()));
        assert_eq!(normalize_numeric_key("abc"), None);
        assert_eq!(normalize_numeric_key(""), None);
    }
}
