//! Mapping state for interactive sessions.
//!
//! Provides session-only state management for manual mapping workflows.
//! No persistence - mappings live only for the duration of the session.

use std::collections::{BTreeMap, BTreeSet};

use sdtm_model::{CoreDesignation, Domain};
use serde::{Deserialize, Serialize};

use crate::error::MappingError;
use crate::score::{ColumnHint, ScoringEngine};

/// Status of a variable's mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableStatus {
    /// Mapping accepted by user.
    Accepted,
    /// Engine suggested a mapping (not yet accepted).
    Suggested,
    /// No mapping or suggestion.
    Unmapped,
}

/// Summary of mapping counts.
#[derive(Debug, Clone, Copy, Default)]
pub struct MappingSummary {
    /// Total number of variables in the domain.
    pub total_variables: usize,
    /// Number of variables with accepted mappings.
    pub mapped: usize,
    /// Number of variables with suggestions (not yet accepted).
    pub suggested: usize,
    /// Number of required variables.
    pub required_total: usize,
    /// Number of required variables that are mapped.
    pub required_mapped: usize,
}

/// A single column-to-variable mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    /// Source column name.
    pub source_column: String,
    /// Target SDTM variable name.
    pub target_variable: String,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f32,
}

/// Exported mapping configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfig {
    /// Domain code (e.g., "AE", "DM").
    pub domain_code: String,
    /// Study identifier.
    pub study_id: String,
    /// List of accepted mappings.
    pub mappings: Vec<Mapping>,
}

/// Mapping state for a domain (session-only, no persistence).
///
/// Manages the interactive workflow where users:
/// 1. View auto-generated suggestions
/// 2. Accept suggestions or manually select columns
/// 3. Clear mappings as needed
#[derive(Debug, Clone)]
pub struct MappingState {
    domain: Domain,
    study_id: String,

    /// Scoring engine for this domain.
    scorer: ScoringEngine,

    /// Engine suggestions: variable_name -> (column, score).
    suggestions: BTreeMap<String, (String, f32)>,

    /// User-accepted mappings: variable_name -> (column, confidence).
    accepted: BTreeMap<String, (String, f32)>,

    /// All source columns.
    source_columns: Vec<String>,

    /// Column hints for metadata.
    column_hints: BTreeMap<String, ColumnHint>,
}

impl MappingState {
    /// Create new mapping state and generate suggestions.
    ///
    /// # Arguments
    /// * `domain` - The SDTM domain definition
    /// * `study_id` - Study identifier
    /// * `source_columns` - List of source column names
    /// * `hints` - Optional metadata about source columns
    /// * `min_confidence` - Minimum confidence for auto-suggestions (0.0 to 1.0)
    pub fn new(
        domain: Domain,
        study_id: &str,
        source_columns: &[String],
        hints: BTreeMap<String, ColumnHint>,
        min_confidence: f32,
    ) -> Self {
        let scorer = ScoringEngine::new(domain.clone(), hints.clone());
        let auto_suggestions = scorer.suggest_all(source_columns, min_confidence);

        let suggestions: BTreeMap<_, _> = auto_suggestions
            .into_iter()
            .map(|s| (s.target_variable, (s.source_column, s.score.score)))
            .collect();

        Self {
            domain,
            study_id: study_id.to_string(),
            scorer,
            suggestions,
            accepted: BTreeMap::new(),
            source_columns: source_columns.to_vec(),
            column_hints: hints,
        }
    }

    /// Get the SDTM domain definition.
    pub fn domain(&self) -> &Domain {
        &self.domain
    }

    /// Get the study ID.
    pub fn study_id(&self) -> &str {
        &self.study_id
    }

    /// Get the scoring engine for dropdown sorting.
    pub fn scorer(&self) -> &ScoringEngine {
        &self.scorer
    }

    /// Get column hints.
    pub fn column_hints(&self) -> &BTreeMap<String, ColumnHint> {
        &self.column_hints
    }

    /// Get mapping status for a variable.
    pub fn status(&self, variable_name: &str) -> VariableStatus {
        if self.accepted.contains_key(variable_name) {
            VariableStatus::Accepted
        } else if self.suggestions.contains_key(variable_name) {
            VariableStatus::Suggested
        } else {
            VariableStatus::Unmapped
        }
    }

    /// Get suggestion for a variable.
    ///
    /// Returns the suggested column and its confidence score.
    pub fn suggestion(&self, variable_name: &str) -> Option<(&str, f32)> {
        self.suggestions
            .get(variable_name)
            .map(|(col, conf)| (col.as_str(), *conf))
    }

    /// Get accepted mapping for a variable.
    ///
    /// Returns the accepted column and its confidence score.
    pub fn accepted(&self, variable_name: &str) -> Option<(&str, f32)> {
        self.accepted
            .get(variable_name)
            .map(|(col, conf)| (col.as_str(), *conf))
    }

    /// Get current mapping (accepted or suggested) for a variable.
    ///
    /// Accepted mappings take priority over suggestions.
    pub fn current_mapping(&self, variable_name: &str) -> Option<(&str, f32)> {
        self.accepted(variable_name)
            .or_else(|| self.suggestion(variable_name))
    }

    /// Accept the engine's suggestion for a variable.
    ///
    /// # Errors
    /// Returns `MappingError::VariableNotFound` if no suggestion exists.
    pub fn accept_suggestion(&mut self, variable_name: &str) -> Result<(), MappingError> {
        let (col, conf) = self
            .suggestions
            .get(variable_name)
            .ok_or_else(|| MappingError::VariableNotFound(variable_name.into()))?
            .clone();

        self.accepted.insert(variable_name.to_string(), (col, conf));
        Ok(())
    }

    /// Accept a manual mapping (user selected from dropdown).
    ///
    /// # Errors
    /// - `MappingError::ColumnNotFound` if column doesn't exist
    /// - `MappingError::ColumnAlreadyUsed` if column is mapped to another variable
    pub fn accept_manual(&mut self, variable_name: &str, column: &str) -> Result<(), MappingError> {
        // Validate column exists
        if !self.source_columns.iter().any(|c| c == column) {
            return Err(MappingError::ColumnNotFound(column.into()));
        }

        // Check if column already used by another variable
        for (var, (col, _)) in &self.accepted {
            if col == column && var != variable_name {
                return Err(MappingError::ColumnAlreadyUsed {
                    column: column.into(),
                    variable: var.clone(),
                });
            }
        }

        // Manual mappings get confidence 1.0
        self.accepted
            .insert(variable_name.to_string(), (column.to_string(), 1.0));
        Ok(())
    }

    /// Clear mapping for a variable.
    ///
    /// Returns `true` if a mapping was removed.
    pub fn clear(&mut self, variable_name: &str) -> bool {
        self.accepted.remove(variable_name).is_some()
    }

    /// Get all source columns.
    pub fn source_columns(&self) -> &[String] {
        &self.source_columns
    }

    /// Get available (unmapped) source columns.
    ///
    /// Returns columns that haven't been assigned to any variable yet.
    pub fn available_columns(&self) -> Vec<&str> {
        let used: BTreeSet<&str> = self
            .accepted
            .values()
            .map(|(col, _)| col.as_str())
            .collect();

        self.source_columns
            .iter()
            .map(String::as_str)
            .filter(|c| !used.contains(*c))
            .collect()
    }

    /// Get summary statistics.
    pub fn summary(&self) -> MappingSummary {
        let required_vars: Vec<_> = self
            .domain
            .variables
            .iter()
            .filter(|v| v.core == Some(CoreDesignation::Required))
            .collect();

        let required_mapped = required_vars
            .iter()
            .filter(|v| self.accepted.contains_key(&v.name))
            .count();

        // Count suggestions that haven't been accepted yet
        let pending_suggestions = self
            .suggestions
            .keys()
            .filter(|var| !self.accepted.contains_key(*var))
            .count();

        MappingSummary {
            total_variables: self.domain.variables.len(),
            mapped: self.accepted.len(),
            suggested: pending_suggestions,
            required_total: required_vars.len(),
            required_mapped,
        }
    }

    /// Export to MappingConfig for downstream use.
    pub fn to_config(&self) -> MappingConfig {
        MappingConfig {
            domain_code: self.domain.name.clone(),
            study_id: self.study_id.clone(),
            mappings: self
                .accepted
                .iter()
                .map(|(var, (col, conf))| Mapping {
                    source_column: col.clone(),
                    target_variable: var.clone(),
                    confidence: *conf,
                })
                .collect(),
        }
    }

    /// Get all accepted mappings.
    pub fn all_accepted(&self) -> &BTreeMap<String, (String, f32)> {
        &self.accepted
    }

    /// Get all suggestions.
    pub fn all_suggestions(&self) -> &BTreeMap<String, (String, f32)> {
        &self.suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdtm_model::{Variable, VariableType};

    fn make_variable(name: &str, core: Option<CoreDesignation>) -> Variable {
        Variable {
            name: name.to_string(),
            label: None,
            data_type: VariableType::Char,
            length: None,
            role: None,
            core,
            codelist_code: None,
            described_value_domain: None,
            order: None,
        }
    }

    fn make_domain(variables: Vec<Variable>) -> Domain {
        Domain {
            name: "TEST".to_string(),
            label: Some("Test Domain".to_string()),
            class: None,
            structure: None,
            dataset_name: None,
            variables,
        }
    }

    #[test]
    fn test_new_generates_suggestions() {
        let domain = make_domain(vec![
            make_variable("USUBJID", Some(CoreDesignation::Required)),
            make_variable("AETERM", None),
        ]);

        let columns = vec!["USUBJID".to_string(), "AETERM".to_string()];
        let state = MappingState::new(domain, "STUDY01", &columns, BTreeMap::new(), 0.5);

        // Should have suggestions for matching columns
        assert!(state.suggestion("USUBJID").is_some());
        assert!(state.suggestion("AETERM").is_some());
    }

    #[test]
    fn test_accept_suggestion() {
        let domain = make_domain(vec![make_variable("USUBJID", None)]);
        let columns = vec!["USUBJID".to_string()];
        let mut state = MappingState::new(domain, "STUDY01", &columns, BTreeMap::new(), 0.5);

        assert_eq!(state.status("USUBJID"), VariableStatus::Suggested);

        state.accept_suggestion("USUBJID").unwrap();

        assert_eq!(state.status("USUBJID"), VariableStatus::Accepted);
    }

    #[test]
    fn test_accept_manual() {
        let domain = make_domain(vec![make_variable("USUBJID", None)]);
        let columns = vec!["SUBJECT_ID".to_string()];
        let mut state = MappingState::new(domain, "STUDY01", &columns, BTreeMap::new(), 0.5);

        state.accept_manual("USUBJID", "SUBJECT_ID").unwrap();

        assert_eq!(state.status("USUBJID"), VariableStatus::Accepted);
        let (col, conf) = state.accepted("USUBJID").unwrap();
        assert_eq!(col, "SUBJECT_ID");
        assert_eq!(conf, 1.0); // Manual mappings get 1.0
    }

    #[test]
    fn test_clear_mapping() {
        let domain = make_domain(vec![make_variable("USUBJID", None)]);
        let columns = vec!["USUBJID".to_string()];
        let mut state = MappingState::new(domain, "STUDY01", &columns, BTreeMap::new(), 0.5);

        state.accept_suggestion("USUBJID").unwrap();
        assert_eq!(state.status("USUBJID"), VariableStatus::Accepted);

        state.clear("USUBJID");
        // Should fall back to suggested since the suggestion still exists
        assert_eq!(state.status("USUBJID"), VariableStatus::Suggested);
    }

    #[test]
    fn test_available_columns() {
        let domain = make_domain(vec![
            make_variable("USUBJID", None),
            make_variable("AETERM", None),
        ]);
        let columns = vec![
            "USUBJID".to_string(),
            "AETERM".to_string(),
            "EXTRA".to_string(),
        ];
        let mut state = MappingState::new(domain, "STUDY01", &columns, BTreeMap::new(), 0.5);

        state.accept_suggestion("USUBJID").unwrap();

        let available = state.available_columns();
        assert!(!available.contains(&"USUBJID"));
        assert!(available.contains(&"AETERM"));
        assert!(available.contains(&"EXTRA"));
    }

    #[test]
    fn test_summary() {
        let domain = make_domain(vec![
            make_variable("USUBJID", Some(CoreDesignation::Required)),
            make_variable("AETERM", Some(CoreDesignation::Required)),
            make_variable("AESEQ", None),
        ]);
        let columns = vec!["USUBJID".to_string(), "AETERM".to_string()];
        let mut state = MappingState::new(domain, "STUDY01", &columns, BTreeMap::new(), 0.5);

        state.accept_suggestion("USUBJID").unwrap();

        let summary = state.summary();
        assert_eq!(summary.total_variables, 3);
        assert_eq!(summary.mapped, 1);
        assert_eq!(summary.required_total, 2);
        assert_eq!(summary.required_mapped, 1);
    }

    #[test]
    fn test_to_config() {
        let domain = make_domain(vec![make_variable("USUBJID", None)]);
        let columns = vec!["USUBJID".to_string()];
        let mut state = MappingState::new(domain, "STUDY01", &columns, BTreeMap::new(), 0.5);

        state.accept_suggestion("USUBJID").unwrap();

        let config = state.to_config();
        assert_eq!(config.domain_code, "TEST");
        assert_eq!(config.study_id, "STUDY01");
        assert_eq!(config.mappings.len(), 1);
        assert_eq!(config.mappings[0].target_variable, "USUBJID");
    }

    #[test]
    fn test_column_already_used_error() {
        let domain = make_domain(vec![
            make_variable("USUBJID", None),
            make_variable("SUBJID", None),
        ]);
        let columns = vec!["SUBJECT".to_string()];
        let mut state = MappingState::new(domain, "STUDY01", &columns, BTreeMap::new(), 0.0);

        state.accept_manual("USUBJID", "SUBJECT").unwrap();

        // Try to map same column to different variable
        let result = state.accept_manual("SUBJID", "SUBJECT");
        assert!(matches!(
            result,
            Err(MappingError::ColumnAlreadyUsed { .. })
        ));
    }
}
