//! Mapping state management for interactive mapping workflows.
//!
//! This module provides types for managing the state of a mapping operation,
//! including accepted mappings, suggestions, and summary statistics.

use std::collections::{BTreeMap, HashSet};

use sdtm_model::{ColumnHint, Domain, MappingConfig, MappingSuggestion, Variable};

use crate::engine::{MappingEngine, MappingResult};

/// State of a mapping operation for a single domain.
///
/// This tracks the mapping workflow including suggestions from the engine,
/// accepted mappings, and unmapped columns.
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
    /// Create a new mapping state from engine results.
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

    /// Create a new mapping state by running the mapping engine.
    pub fn from_domain(
        sdtm_domain: Domain,
        study_id: &str,
        source_columns: &[String],
        column_hints: BTreeMap<String, ColumnHint>,
        min_confidence: f32,
    ) -> Self {
        let engine =
            MappingEngine::new(sdtm_domain.clone(), min_confidence, column_hints.clone());
        let result = engine.suggest(source_columns);
        let domain_code = sdtm_domain.code.clone();
        Self::new(&domain_code, study_id, sdtm_domain, result, column_hints)
    }

    /// Get filtered variables based on search text.
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
                    let matches_name = v.name.to_lowercase().contains(&filter);
                    let matches_label = v
                        .label
                        .as_ref()
                        .is_some_and(|l| l.to_lowercase().contains(&filter));
                    let matches_subjid = v.name.eq_ignore_ascii_case("USUBJID")
                        && (filter.contains("subjid")
                            || filter.contains("subject id")
                            || filter.contains("subject"));
                    matches_name || matches_label || matches_subjid
                }
            })
            .collect()
    }

    /// Get the currently selected variable.
    pub fn selected_variable(&self) -> Option<&Variable> {
        self.selected_variable_idx
            .and_then(|idx| self.sdtm_domain.variables.get(idx))
    }

    /// Get suggestion for a specific variable.
    pub fn get_suggestion_for(&self, variable_name: &str) -> Option<&MappingSuggestion> {
        self.suggestions
            .iter()
            .find(|s| s.target_variable.eq_ignore_ascii_case(variable_name))
    }

    /// Get accepted mapping for a variable.
    pub fn get_accepted_for(&self, variable_name: &str) -> Option<(&str, f32)> {
        self.accepted
            .get(variable_name)
            .map(|(col, conf)| (col.as_str(), *conf))
    }

    /// Get mapping status for a variable.
    pub fn variable_status(&self, variable_name: &str) -> VariableMappingStatus {
        if self.accepted.contains_key(variable_name) {
            VariableMappingStatus::Accepted
        } else if self.get_suggestion_for(variable_name).is_some() {
            VariableMappingStatus::Suggested
        } else {
            VariableMappingStatus::Unmapped
        }
    }

    /// Accept the suggestion for a variable.
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

    /// Accept a manual mapping for a variable.
    pub fn accept_manual(&mut self, variable_name: &str, source_column: &str) {
        self.accepted
            .insert(variable_name.to_string(), (source_column.to_string(), 1.0));
        // Remove from unmapped if present
        self.unmapped_columns.retain(|c| c != source_column);
    }

    /// Clear the mapping for a variable.
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

    /// Get all source columns (mapped and unmapped).
    pub fn all_source_columns(&self) -> Vec<&str> {
        self.column_hints.keys().map(String::as_str).collect()
    }

    /// Check if a source column is already used.
    pub fn is_column_used(&self, column: &str) -> bool {
        self.accepted.values().any(|(c, _)| c == column)
    }

    /// Get available (unused) source columns.
    pub fn available_columns(&self) -> Vec<&str> {
        let used: HashSet<&str> = self.accepted.values().map(|(c, _)| c.as_str()).collect();
        self.all_source_columns()
            .into_iter()
            .filter(|c| !used.contains(*c))
            .collect()
    }

    /// Get summary counts.
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

    /// Convert to final MappingConfig.
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

/// Status of a variable's mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableMappingStatus {
    /// Has an accepted mapping.
    Accepted,
    /// Has a suggestion but not yet accepted.
    Suggested,
    /// No mapping or suggestion.
    Unmapped,
}

/// Summary of mapping counts.
#[derive(Debug, Clone, Copy)]
pub struct MappingSummary {
    /// Total number of variables in the domain.
    pub total_variables: usize,
    /// Number of accepted mappings.
    pub mapped: usize,
    /// Number of pending suggestions.
    pub suggested: usize,
    /// Total required variables.
    pub required_total: usize,
    /// Required variables that are mapped.
    pub required_mapped: usize,
}
