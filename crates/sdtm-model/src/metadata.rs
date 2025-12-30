//! Study-level metadata models.
//!
//! This module defines structures for representing study-specific metadata,
//! including source column definitions and study-specific codelists.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// Definition of a source column from study metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceColumn {
    /// Column identifier/name.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Data type (e.g., "text", "integer").
    pub data_type: Option<String>,
    /// Whether the column is mandatory.
    pub mandatory: bool,
    /// Name of the format/codelist associated with this column.
    pub format_name: Option<String>,
    /// Maximum content length.
    pub content_length: Option<usize>,
}

/// A study-specific codelist (format).
///
/// Maps coded values to decoded text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyCodelist {
    /// Name of the format/codelist.
    pub format_name: String,
    /// Map of code values to text.
    values: BTreeMap<String, String>,
    /// Uppercase lookup map.
    values_upper: BTreeMap<String, String>,
    /// Numeric lookup map.
    values_numeric: BTreeMap<String, String>,
}

impl StudyCodelist {
    pub fn new(format_name: String) -> Self {
        Self {
            format_name,
            values: BTreeMap::new(),
            values_upper: BTreeMap::new(),
            values_numeric: BTreeMap::new(),
        }
    }

    pub fn insert_value(&mut self, code_value: &str, code_text: &str) {
        let trimmed = code_value.trim();
        let text = code_text.trim();
        if trimmed.is_empty() || text.is_empty() {
            return;
        }
        self.values.insert(trimmed.to_string(), text.to_string());
        self.values_upper
            .insert(trimmed.to_uppercase(), text.to_string());
        if let Some(key) = normalize_numeric_key(trimmed) {
            self.values_numeric.insert(key, text.to_string());
        }
    }

    pub fn lookup_text(&self, raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(text) = self.values.get(trimmed) {
            return Some(text.clone());
        }
        let upper = trimmed.to_uppercase();
        if let Some(text) = self.values_upper.get(&upper) {
            return Some(text.clone());
        }
        if let Some(key) = normalize_numeric_key(trimmed)
            && let Some(text) = self.values_numeric.get(&key)
        {
            return Some(text.clone());
        }
        None
    }
}

/// Collection of study metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StudyMetadata {
    /// Source column definitions keyed by column ID.
    pub items: BTreeMap<String, SourceColumn>,
    /// Study-specific codelists keyed by format name.
    pub codelists: BTreeMap<String, StudyCodelist>,
}

impl StudyMetadata {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.codelists.is_empty()
    }
}

fn normalize_numeric_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = trimmed.parse::<f64>().ok()?;
    let mut text = format!("{parsed}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    if text.is_empty() { None } else { Some(text) }
}
