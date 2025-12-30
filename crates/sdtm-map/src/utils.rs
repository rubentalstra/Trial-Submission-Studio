//! Utility functions for mapping operations.

use crate::types::{MappingConfig, MappingSuggestion};

/// Normalizes text for comparison by lowercasing and replacing separators with spaces.
pub fn normalize_text(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .replace(['_', '-', '.', '/', '\\'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Trims whitespace from a column name, preserving the original casing.
pub fn safe_column_name(raw: &str) -> String {
    raw.trim().to_string()
}

/// Merges multiple mapping configs per domain into a single config per domain.
///
/// When the same target variable is mapped by multiple configs, the one with
/// the highest confidence is selected. Ties are broken by source column name.
pub fn merge_mappings(
    configs: &std::collections::BTreeMap<String, Vec<MappingConfig>>,
    study_id: &str,
) -> std::collections::BTreeMap<String, MappingConfig> {
    let mut merged = std::collections::BTreeMap::new();
    for (domain_code, entries) in configs {
        if entries.is_empty() {
            continue;
        }
        merged.insert(
            domain_code.to_uppercase(),
            merge_mapping_configs(domain_code, study_id, entries),
        );
    }
    merged
}

/// Merges multiple mapping configs for a single domain into one.
///
/// For each target variable, keeps the mapping with the highest confidence.
/// All unmapped columns from all configs are collected.
pub fn merge_mapping_configs(
    domain_code: &str,
    study_id: &str,
    configs: &[MappingConfig],
) -> MappingConfig {
    let mut best: std::collections::BTreeMap<String, MappingSuggestion> =
        std::collections::BTreeMap::new();
    let mut unmapped = std::collections::BTreeSet::new();
    for config in configs {
        for suggestion in &config.mappings {
            let key = suggestion.target_variable.to_uppercase();
            match best.get(&key) {
                Some(existing) => {
                    if suggestion.confidence > existing.confidence
                        || (suggestion.confidence == existing.confidence
                            && suggestion.source_column < existing.source_column)
                    {
                        best.insert(key, suggestion.clone());
                    }
                }
                None => {
                    best.insert(key, suggestion.clone());
                }
            }
        }
        for column in &config.unmapped_columns {
            unmapped.insert(column.clone());
        }
    }
    MappingConfig {
        domain_code: domain_code.to_uppercase(),
        study_id: study_id.to_string(),
        mappings: best.into_values().collect(),
        unmapped_columns: unmapped.into_iter().collect(),
    }
}
