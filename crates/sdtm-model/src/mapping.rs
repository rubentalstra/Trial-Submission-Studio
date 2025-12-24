use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnHint {
    pub is_numeric: bool,
    pub unique_ratio: f64,
    pub null_ratio: f64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSuggestion {
    pub source_column: String,
    pub target_variable: String,
    pub confidence: f32,
    pub transformation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfig {
    pub domain_code: String,
    pub study_id: String,
    pub mappings: Vec<MappingSuggestion>,
    pub unmapped_columns: Vec<String>,
}
