//! Mapping service for the GUI
//!
//! Provides column-to-variable mapping functionality using sdtm-map,
//! with support for incremental acceptance/rejection of suggestions.

use anyhow::{Context, Result};
use polars::prelude::DataFrame;
use sdtm_map::{ConfidenceLevel, ConfidenceThresholds, MappingEngine, MappingResult};
use sdtm_model::{ColumnHint, Domain, MappingConfig, MappingSuggestion};
use std::collections::BTreeMap;

/// State of a mapping operation for a single domain
#[derive(Debug, Clone)]
pub struct MappingState {
    /// Domain code
    pub domain_code: String,
    /// Study ID
    pub study_id: String,
    /// Accepted mappings
    pub accepted: Vec<MappingSuggestion>,
    /// Pending suggestions (not yet accepted or rejected)
    pub pending: Vec<MappingSuggestion>,
    /// Rejected/unmapped columns
    pub unmapped: Vec<String>,
}

impl MappingState {
    /// Create a new mapping state from a MappingResult
    pub fn from_result(domain_code: &str, study_id: &str, result: MappingResult) -> Self {
        Self {
            domain_code: domain_code.to_string(),
            study_id: study_id.to_string(),
            accepted: Vec::new(),
            pending: result.mappings,
            unmapped: result.unmapped_columns,
        }
    }

    /// Get all mappings grouped by confidence level
    pub fn pending_by_level(&self) -> BTreeMap<ConfidenceLevel, Vec<&MappingSuggestion>> {
        let thresholds = ConfidenceThresholds::default();
        let mut groups: BTreeMap<ConfidenceLevel, Vec<&MappingSuggestion>> = BTreeMap::new();

        for mapping in &self.pending {
            if let Some(level) = thresholds.categorize(mapping.confidence) {
                groups.entry(level).or_default().push(mapping);
            }
        }

        groups
    }

    /// Accept a pending mapping by source column name
    pub fn accept(&mut self, source_column: &str) -> bool {
        if let Some(pos) = self
            .pending
            .iter()
            .position(|m| m.source_column == source_column)
        {
            let mapping = self.pending.remove(pos);
            self.accepted.push(mapping);
            true
        } else {
            false
        }
    }

    /// Accept all pending mappings at or above a confidence level
    pub fn accept_all_above(&mut self, min_level: ConfidenceLevel) {
        let thresholds = ConfidenceThresholds::default();
        let (to_accept, remaining): (Vec<_>, Vec<_>) =
            self.pending.drain(..).partition(|m| {
                thresholds
                    .categorize(m.confidence)
                    .is_some_and(|level| level >= min_level)
            });

        self.accepted.extend(to_accept);
        self.pending = remaining;
    }

    /// Reject a pending mapping by source column name
    pub fn reject(&mut self, source_column: &str) -> bool {
        if let Some(pos) = self
            .pending
            .iter()
            .position(|m| m.source_column == source_column)
        {
            let mapping = self.pending.remove(pos);
            self.unmapped.push(mapping.source_column);
            true
        } else {
            false
        }
    }

    /// Manually map an unmapped column to a variable
    pub fn set_manual_mapping(&mut self, source_column: &str, target_variable: &str) -> bool {
        // Remove from unmapped if present
        if let Some(pos) = self.unmapped.iter().position(|c| c == source_column) {
            self.unmapped.remove(pos);
        }

        // Check if already accepted
        if self
            .accepted
            .iter()
            .any(|m| m.source_column == source_column)
        {
            return false;
        }

        // Add as manually accepted mapping
        self.accepted.push(MappingSuggestion {
            source_column: source_column.to_string(),
            target_variable: target_variable.to_string(),
            confidence: 1.0, // Manual mappings are always 100% confident
            transformation: None,
        });

        true
    }

    /// Convert to MappingConfig for storage/export
    pub fn to_config(&self) -> MappingConfig {
        MappingConfig {
            domain_code: self.domain_code.clone(),
            study_id: self.study_id.clone(),
            mappings: self.accepted.clone(),
            unmapped_columns: self.unmapped.clone(),
        }
    }

    /// Get count summary
    pub fn summary(&self) -> MappingSummary {
        MappingSummary {
            accepted: self.accepted.len(),
            pending: self.pending.len(),
            unmapped: self.unmapped.len(),
        }
    }
}

/// Summary of mapping counts
#[derive(Debug, Clone, Copy)]
pub struct MappingSummary {
    pub accepted: usize,
    pub pending: usize,
    pub unmapped: usize,
}

/// Service for generating and managing column mappings
pub struct MappingService;

impl MappingService {
    /// Generate mapping suggestions for a domain
    pub fn generate_suggestions(
        domain: &Domain,
        source_columns: &[String],
        column_hints: BTreeMap<String, ColumnHint>,
        min_confidence: f32,
    ) -> MappingResult {
        let engine = MappingEngine::new(domain.clone(), min_confidence, column_hints);
        engine.suggest(source_columns)
    }

    /// Generate mapping state for interactive editing
    pub fn create_mapping_state(
        domain: &Domain,
        study_id: &str,
        source_columns: &[String],
        column_hints: BTreeMap<String, ColumnHint>,
    ) -> MappingState {
        let result = Self::generate_suggestions(domain, source_columns, column_hints, 0.6);
        MappingState::from_result(&domain.code, study_id, result)
    }

    /// Extract column hints from a DataFrame
    pub fn extract_column_hints(df: &DataFrame) -> BTreeMap<String, ColumnHint> {
        let mut hints = BTreeMap::new();
        let row_count = df.height();

        for name in df.get_column_names() {
            if let Ok(col) = df.column(name) {
                let null_count = col.null_count();
                let null_ratio = if row_count > 0 {
                    null_count as f64 / row_count as f64
                } else {
                    0.0
                };

                let is_numeric = matches!(
                    col.dtype(),
                    polars::datatypes::DataType::Int8
                        | polars::datatypes::DataType::Int16
                        | polars::datatypes::DataType::Int32
                        | polars::datatypes::DataType::Int64
                        | polars::datatypes::DataType::UInt8
                        | polars::datatypes::DataType::UInt16
                        | polars::datatypes::DataType::UInt32
                        | polars::datatypes::DataType::UInt64
                        | polars::datatypes::DataType::Float32
                        | polars::datatypes::DataType::Float64
                );

                // Calculate unique ratio
                let unique_ratio = if row_count > 0 {
                    if let Ok(unique) = col.n_unique() {
                        unique as f64 / row_count as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                hints.insert(
                    name.to_string(),
                    ColumnHint {
                        is_numeric,
                        null_ratio,
                        unique_ratio,
                        label: None,
                    },
                );
            }
        }

        hints
    }

    /// Preview a mapping by extracting sample values
    pub fn preview_mapping(
        df: &DataFrame,
        source_column: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        let col = df
            .column(source_column)
            .with_context(|| format!("Column '{}' not found", source_column))?;

        let str_col = col
            .str()
            .with_context(|| format!("Column '{}' is not a string column", source_column))?;

        let mut samples = Vec::new();
        for i in 0..limit.min(df.height()) {
            if let Some(val) = str_col.get(i) {
                samples.push(val.to_string());
            }
        }

        Ok(samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use sdtm_model::{Variable, VariableType};

    fn make_variable(name: &str, label: &str) -> Variable {
        Variable {
            name: name.to_string(),
            label: Some(label.to_string()),
            data_type: VariableType::Char,
            length: None,
            role: None,
            core: None,
            codelist_code: None,
            order: None,
        }
    }

    fn test_domain() -> Domain {
        Domain {
            code: "DM".to_string(),
            description: Some("Demographics".to_string()),
            class_name: Some("SPECIAL PURPOSE".to_string()),
            dataset_class: None,
            label: Some("Demographics".to_string()),
            structure: None,
            dataset_name: None,
            variables: vec![
                make_variable("STUDYID", "Study Identifier"),
                make_variable("USUBJID", "Unique Subject Identifier"),
                make_variable("AGE", "Age"),
            ],
        }
    }

    #[test]
    fn test_generate_suggestions() {
        let domain = test_domain();
        let columns = vec!["STUDY".to_string(), "SUBJECT".to_string(), "AGE".to_string()];

        let result = MappingService::generate_suggestions(&domain, &columns, BTreeMap::new(), 0.6);

        assert!(!result.mappings.is_empty());
    }

    #[test]
    fn test_mapping_state_accept() {
        let domain = test_domain();
        let columns = vec!["AGE".to_string()];

        let mut state = MappingService::create_mapping_state(&domain, "STUDY01", &columns, BTreeMap::new());

        assert!(!state.pending.is_empty());
        let first_col = state.pending[0].source_column.clone();

        state.accept(&first_col);

        assert_eq!(state.accepted.len(), 1);
        assert!(state
            .pending
            .iter()
            .all(|m| m.source_column != first_col));
    }

    #[test]
    fn test_extract_column_hints() {
        let df = DataFrame::new(vec![
            Series::new("NAME".into(), vec!["Alice", "Bob", "Charlie"]).into(),
            Series::new("AGE".into(), vec![25i64, 30, 35]).into(),
        ])
        .unwrap();

        let hints = MappingService::extract_column_hints(&df);

        assert!(hints.contains_key("NAME"));
        assert!(hints.contains_key("AGE"));
        assert!(!hints["NAME"].is_numeric);
        assert!(hints["AGE"].is_numeric);
    }
}
