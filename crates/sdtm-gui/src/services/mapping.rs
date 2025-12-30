//! Mapping service for the GUI
//!
//! Provides column-to-variable mapping functionality using sdtm-map,
//! with support for variable-centric mapping workflow.

// Fields/methods used for state serialization and future features
#![allow(dead_code)]

use polars::prelude::DataFrame;
use sdtm_map::{ConfidenceLevel, ConfidenceThresholds, MappingEngine, MappingResult};
use sdtm_model::{ColumnHint, Domain, MappingConfig, MappingSuggestion, Variable};
use std::collections::BTreeMap;

/// State of a mapping operation for a single domain
#[derive(Debug, Clone)]
pub struct MappingState {
    /// Domain code
    pub domain_code: String,
    /// Study ID
    pub study_id: String,
    /// SDTM domain definition (for variable metadata)
    pub sdtm_domain: Domain,
    /// All mapping suggestions from the engine
    pub suggestions: Vec<MappingSuggestion>,
    /// Accepted mappings (variable_name -> source_column, confidence)
    pub accepted: BTreeMap<String, (String, f32)>,
    /// Source columns that couldn't be mapped
    pub unmapped_columns: Vec<String>,
    /// UI state: currently selected variable index
    pub selected_variable_idx: Option<usize>,
    /// UI state: search filter text
    pub search_filter: String,
    /// Column hints from source data
    pub column_hints: BTreeMap<String, ColumnHint>,
}

impl MappingState {
    /// Create a new mapping state
    pub fn new(
        domain_code: &str,
        study_id: &str,
        sdtm_domain: Domain,
        result: MappingResult,
        column_hints: BTreeMap<String, ColumnHint>,
    ) -> Self {
        Self {
            domain_code: domain_code.to_string(),
            study_id: study_id.to_string(),
            sdtm_domain,
            suggestions: result.mappings,
            accepted: BTreeMap::new(),
            unmapped_columns: result.unmapped_columns,
            selected_variable_idx: None,
            search_filter: String::new(),
            column_hints,
        }
    }

    /// Get filtered variables based on search text
    pub fn filtered_variables(&self) -> Vec<(usize, &Variable)> {
        let filter = self.search_filter.to_lowercase();
        self.sdtm_domain
            .variables
            .iter()
            .enumerate()
            .filter(|(_, v)| {
                if filter.is_empty() {
                    true
                } else {
                    v.name.to_lowercase().contains(&filter)
                        || v.label
                            .as_ref()
                            .is_some_and(|l| l.to_lowercase().contains(&filter))
                }
            })
            .collect()
    }

    /// Get the currently selected variable
    pub fn selected_variable(&self) -> Option<&Variable> {
        self.selected_variable_idx
            .and_then(|idx| self.sdtm_domain.variables.get(idx))
    }

    /// Get suggestion for a specific variable
    pub fn get_suggestion_for(&self, variable_name: &str) -> Option<&MappingSuggestion> {
        self.suggestions
            .iter()
            .find(|s| s.target_variable.eq_ignore_ascii_case(variable_name))
    }

    /// Get accepted mapping for a variable
    pub fn get_accepted_for(&self, variable_name: &str) -> Option<(&str, f32)> {
        self.accepted
            .get(variable_name)
            .map(|(col, conf)| (col.as_str(), *conf))
    }

    /// Get mapping status for a variable
    pub fn variable_status(&self, variable_name: &str) -> VariableMappingStatus {
        if self.accepted.contains_key(variable_name) {
            VariableMappingStatus::Accepted
        } else if self.get_suggestion_for(variable_name).is_some() {
            VariableMappingStatus::Suggested
        } else {
            VariableMappingStatus::Unmapped
        }
    }

    /// Accept the suggestion for a variable
    pub fn accept_suggestion(&mut self, variable_name: &str) -> bool {
        if let Some(suggestion) = self.get_suggestion_for(variable_name).cloned() {
            self.accepted.insert(
                variable_name.to_string(),
                (suggestion.source_column.clone(), suggestion.confidence),
            );
            true
        } else {
            false
        }
    }

    /// Accept a manual mapping for a variable
    pub fn accept_manual(&mut self, variable_name: &str, source_column: &str) {
        self.accepted.insert(
            variable_name.to_string(),
            (source_column.to_string(), 1.0),
        );
        // Remove from unmapped if present
        self.unmapped_columns.retain(|c| c != source_column);
    }

    /// Clear the mapping for a variable
    pub fn clear_mapping(&mut self, variable_name: &str) -> bool {
        if let Some((source_col, _)) = self.accepted.remove(variable_name) {
            // Add back to unmapped if not suggested elsewhere
            if !self
                .suggestions
                .iter()
                .any(|s| s.source_column == source_col)
            {
                self.unmapped_columns.push(source_col);
            }
            true
        } else {
            false
        }
    }

    /// Accept all high-confidence suggestions
    pub fn accept_all_high(&mut self) {
        let thresholds = ConfidenceThresholds::default();
        for suggestion in &self.suggestions {
            if thresholds.categorize(suggestion.confidence) == Some(ConfidenceLevel::High) {
                if !self.accepted.contains_key(&suggestion.target_variable) {
                    self.accepted.insert(
                        suggestion.target_variable.clone(),
                        (suggestion.source_column.clone(), suggestion.confidence),
                    );
                }
            }
        }
    }

    /// Get all source columns (mapped and unmapped)
    pub fn all_source_columns(&self) -> Vec<&str> {
        let mut cols: Vec<&str> = self.column_hints.keys().map(String::as_str).collect();
        cols.sort();
        cols
    }

    /// Check if a source column is already used
    pub fn is_column_used(&self, column: &str) -> bool {
        self.accepted.values().any(|(c, _)| c == column)
    }

    /// Get available (unused) source columns
    pub fn available_columns(&self) -> Vec<&str> {
        self.all_source_columns()
            .into_iter()
            .filter(|c| !self.is_column_used(c))
            .collect()
    }

    /// Get summary counts
    pub fn summary(&self) -> MappingSummary {
        let required_count = self
            .sdtm_domain
            .variables
            .iter()
            .filter(|v| v.core.as_deref() == Some("Req"))
            .count();
        let required_mapped = self
            .sdtm_domain
            .variables
            .iter()
            .filter(|v| v.core.as_deref() == Some("Req") && self.accepted.contains_key(&v.name))
            .count();

        MappingSummary {
            total_variables: self.sdtm_domain.variables.len(),
            mapped: self.accepted.len(),
            suggested: self
                .suggestions
                .iter()
                .filter(|s| !self.accepted.contains_key(&s.target_variable))
                .count(),
            required_total: required_count,
            required_mapped,
        }
    }

    /// Convert to final MappingConfig
    pub fn to_config(&self) -> MappingConfig {
        let mappings: Vec<MappingSuggestion> = self
            .accepted
            .iter()
            .map(|(var, (col, conf))| MappingSuggestion {
                source_column: col.clone(),
                target_variable: var.clone(),
                confidence: *conf,
                transformation: None,
            })
            .collect();

        MappingConfig {
            domain_code: self.domain_code.clone(),
            study_id: self.study_id.clone(),
            mappings,
            unmapped_columns: self.unmapped_columns.clone(),
        }
    }
}

/// Status of a variable's mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableMappingStatus {
    /// Has an accepted mapping
    Accepted,
    /// Has a suggestion but not yet accepted
    Suggested,
    /// No mapping or suggestion
    Unmapped,
}

impl VariableMappingStatus {
    /// Get status icon
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Accepted => "✓",
            Self::Suggested => "○",
            Self::Unmapped => "—",
        }
    }
}

/// Summary of mapping counts
#[derive(Debug, Clone, Copy)]
pub struct MappingSummary {
    pub total_variables: usize,
    pub mapped: usize,
    pub suggested: usize,
    pub required_total: usize,
    pub required_mapped: usize,
}

/// Service for generating and managing column mappings
pub struct MappingService;

impl MappingService {
    /// Generate mapping suggestions for a domain
    pub fn generate_suggestions(
        domain: &Domain,
        source_columns: &[String],
        column_hints: &BTreeMap<String, ColumnHint>,
        min_confidence: f32,
    ) -> MappingResult {
        let engine = MappingEngine::new(domain.clone(), min_confidence, column_hints.clone());
        engine.suggest(source_columns)
    }

    /// Create a new mapping state
    pub fn create_mapping_state(
        sdtm_domain: Domain,
        study_id: &str,
        source_columns: &[String],
        column_hints: BTreeMap<String, ColumnHint>,
    ) -> MappingState {
        let result =
            Self::generate_suggestions(&sdtm_domain, source_columns, &column_hints, 0.6);
        let domain_code = sdtm_domain.code.clone();
        MappingState::new(&domain_code, study_id, sdtm_domain, result, column_hints)
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

    /// Get sample values from a column
    pub fn get_sample_values(df: &DataFrame, column: &str, limit: usize) -> Vec<String> {
        let Ok(col) = df.column(column) else {
            return Vec::new();
        };

        let mut samples = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Try to get unique sample values
        if let Ok(str_col) = col.str() {
            for i in 0..df.height().min(limit * 3) {
                if let Some(val) = str_col.get(i) {
                    if !val.is_empty() && seen.insert(val.to_string()) {
                        samples.push(val.to_string());
                        if samples.len() >= limit {
                            break;
                        }
                    }
                }
            }
        } else {
            // For non-string columns, format as string using Display (not Debug)
            for i in 0..df.height().min(limit) {
                if let Ok(val) = col.get(i) {
                    // Use Display formatting which gives clean output without type info
                    let formatted = format!("{}", val);
                    if formatted != "null" && !formatted.is_empty() && seen.insert(formatted.clone()) {
                        samples.push(formatted);
                        if samples.len() >= limit {
                            break;
                        }
                    }
                }
            }
        }

        samples
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use sdtm_model::VariableType;

    fn make_variable(name: &str, label: &str, core: Option<&str>) -> Variable {
        Variable {
            name: name.to_string(),
            label: Some(label.to_string()),
            data_type: VariableType::Char,
            length: None,
            role: None,
            core: core.map(String::from),
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
                make_variable("STUDYID", "Study Identifier", Some("Req")),
                make_variable("USUBJID", "Unique Subject Identifier", Some("Req")),
                make_variable("AGE", "Age", Some("Exp")),
            ],
        }
    }

    #[test]
    fn test_mapping_state_accept_suggestion() {
        let domain = test_domain();
        let columns = vec!["AGE".to_string()];
        let hints = BTreeMap::new();

        let mut state =
            MappingService::create_mapping_state(domain, "STUDY01", &columns, hints);

        // Should have suggestion for AGE
        assert!(state.get_suggestion_for("AGE").is_some());

        // Accept it
        state.accept_suggestion("AGE");
        assert!(state.get_accepted_for("AGE").is_some());
        assert_eq!(state.variable_status("AGE"), VariableMappingStatus::Accepted);
    }

    #[test]
    fn test_mapping_state_clear() {
        let domain = test_domain();
        let columns = vec!["AGE".to_string()];
        let hints = BTreeMap::new();

        let mut state =
            MappingService::create_mapping_state(domain, "STUDY01", &columns, hints);

        state.accept_suggestion("AGE");
        assert!(state.get_accepted_for("AGE").is_some());

        state.clear_mapping("AGE");
        assert!(state.get_accepted_for("AGE").is_none());
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
