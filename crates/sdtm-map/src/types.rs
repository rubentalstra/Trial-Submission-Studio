//! Column mapping types for source-to-SDTM variable mapping.
//!
//! This module provides types for representing column mapping suggestions
//! and configurations used to transform source data to SDTM format.

use serde::{Deserialize, Serialize};

/// Hints about a source column's characteristics.
///
/// Used by the mapping engine to infer appropriate SDTM variable mappings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnHint {
    /// True if the column contains numeric values.
    pub is_numeric: bool,
    /// Ratio of unique values to total rows (0.0 to 1.0).
    pub unique_ratio: f64,
    /// Ratio of null/missing values to total rows (0.0 to 1.0).
    pub null_ratio: f64,
    /// Optional label/description from source metadata.
    pub label: Option<String>,
}

/// A suggested mapping from source column to SDTM variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSuggestion {
    /// Source column name from input data.
    pub source_column: String,
    /// Target SDTM variable name.
    pub target_variable: String,
    /// Confidence score (0.0 to 1.0) for this mapping.
    pub confidence: f32,
    /// Optional transformation to apply (e.g., "uppercase", "date_iso8601").
    pub transformation: Option<String>,
}

/// Complete mapping configuration for a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfig {
    /// Target domain code (e.g., "AE", "DM").
    pub domain_code: String,
    /// Study identifier.
    pub study_id: String,
    /// Approved column-to-variable mappings.
    pub mappings: Vec<MappingSuggestion>,
    /// Source columns that could not be mapped.
    pub unmapped_columns: Vec<String>,
}
