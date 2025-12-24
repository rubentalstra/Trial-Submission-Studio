use std::collections::{BTreeMap, BTreeSet};

use rapidfuzz::distance::jaro_winkler::similarity as jaro_similarity;

use sdtm_model::{ColumnHint, Domain, MappingConfig, MappingSuggestion};

use crate::patterns::build_variable_patterns;
use crate::utils::{normalize_text, safe_column_name};

const SEQ_UNIQUENESS_MIN: f64 = 0.5;
const REQUIRED_NULL_RATIO_MAX: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct MappingResult {
    pub mappings: Vec<MappingSuggestion>,
    pub unmapped_columns: Vec<String>,
}

pub struct MappingEngine {
    domain: Domain,
    min_confidence: f32,
    column_hints: BTreeMap<String, ColumnHint>,
    variable_patterns: BTreeMap<String, Vec<String>>,
}

impl MappingEngine {
    pub fn new(
        domain: Domain,
        min_confidence: f32,
        column_hints: BTreeMap<String, ColumnHint>,
    ) -> Self {
        let variable_patterns = build_variable_patterns(&domain);
        Self {
            domain,
            min_confidence,
            column_hints,
            variable_patterns,
        }
    }

    pub fn suggest(&self, columns: &[String]) -> MappingResult {
        let mut suggestions = Vec::new();
        let mut unmapped = Vec::new();
        let mut assigned_targets = BTreeSet::new();
        let mut alias_mappings: BTreeMap<String, MappingSuggestion> = BTreeMap::new();
        let mut alias_collisions = BTreeSet::new();

        for column in columns {
            if let Some(alias_target) = self.alias_override(column) {
                if assigned_targets.contains(&alias_target) {
                    alias_collisions.insert(column.clone());
                    continue;
                }
                assigned_targets.insert(alias_target.clone());
                alias_mappings.insert(
                    column.clone(),
                    MappingSuggestion {
                        source_column: safe_column_name(column),
                        target_variable: alias_target,
                        confidence: 1.0,
                        transformation: None,
                    },
                );
            }
        }

        for column in columns {
            if let Some(mapping) = alias_mappings.get(column) {
                suggestions.push(mapping.clone());
                continue;
            }
            if alias_collisions.contains(column) {
                unmapped.push(column.clone());
                continue;
            }
            if let Some((candidate, confidence)) = self.best_match(column) {
                if confidence < self.min_confidence {
                    unmapped.push(column.clone());
                    continue;
                }
                if assigned_targets.contains(&candidate) {
                    unmapped.push(column.clone());
                    continue;
                }
                assigned_targets.insert(candidate.clone());
                suggestions.push(MappingSuggestion {
                    source_column: safe_column_name(column),
                    target_variable: candidate,
                    confidence,
                    transformation: None,
                });
            } else {
                unmapped.push(column.clone());
            }
        }

        MappingResult {
            mappings: suggestions,
            unmapped_columns: unmapped,
        }
    }

    pub fn to_config(&self, study_id: &str, result: MappingResult) -> MappingConfig {
        MappingConfig {
            domain_code: self.domain.code.clone(),
            study_id: study_id.to_string(),
            mappings: result.mappings,
            unmapped_columns: result.unmapped_columns,
        }
    }

    fn alias_override(&self, column: &str) -> Option<String> {
        let normalized = normalize_text(column);
        for (target_var, patterns) in &self.variable_patterns {
            for pattern in patterns {
                if &normalized == pattern {
                    return Some(target_var.clone());
                }
            }
        }
        None
    }

    fn best_match(&self, column: &str) -> Option<(String, f32)> {
        let normalized = normalize_text(column);
        let mut best: Option<(String, f32)> = None;
        for variable in &self.domain.variables {
            let score_raw = jaro_similarity(column.to_uppercase().chars(), variable.name.chars());
            let score_norm =
                jaro_similarity(normalized.chars(), variable.name.to_lowercase().chars());
            let mut score = score_raw.max(score_norm);
            score = self.apply_hints(column, variable.name.as_str(), score);
            let score_f32 = score as f32;
            if best.as_ref().map(|(_, s)| score_f32 > *s).unwrap_or(true) {
                best = Some((variable.name.clone(), score_f32));
            }
        }
        best
    }

    fn apply_hints(&self, column: &str, variable_name: &str, score: f64) -> f64 {
        let hint = match self.column_hints.get(column) {
            Some(hint) => hint,
            None => return score,
        };
        let mut adjusted = score;
        let is_numeric_var = variable_name.to_uppercase().ends_with("N");
        if is_numeric_var != hint.is_numeric {
            adjusted *= 0.85;
        }
        if variable_name.to_uppercase().ends_with("SEQ") && hint.unique_ratio < SEQ_UNIQUENESS_MIN {
            adjusted *= 0.9;
        }
        if hint.null_ratio > REQUIRED_NULL_RATIO_MAX {
            adjusted *= 0.9;
        }
        adjusted
    }
}
