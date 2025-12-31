//! Fuzzy matching and scoring for column-to-variable mapping.
//!
//! Uses Jaro-Winkler similarity as the base algorithm with optional
//! adjustments for label matching, suffix patterns, and type compatibility.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use rapidfuzz::distance::jaro_winkler;
use sdtm_model::{Domain, Variable};
use serde::{Deserialize, Serialize};

/// Hints about a source column's characteristics.
///
/// Used to improve scoring accuracy based on column metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColumnHint {
    /// Whether the column contains numeric data.
    pub is_numeric: bool,
    /// Ratio of unique values (0.0 to 1.0).
    pub unique_ratio: f64,
    /// Ratio of null/missing values (0.0 to 1.0).
    pub null_ratio: f64,
    /// Optional column label from source metadata.
    pub label: Option<String>,
}

/// Score for a single column-variable pair.
#[derive(Debug, Clone)]
pub struct ColumnScore {
    /// Final confidence score (0.0 to 1.0, may slightly exceed 1.0 with boosts).
    pub score: f32,
    /// Breakdown of score components for explainability.
    pub explanation: Vec<ScoreComponent>,
}

impl ColumnScore {
    /// Human-readable explanation of the score.
    pub fn explain(&self) -> String {
        self.explanation
            .iter()
            .map(|c| format!("{}: {:.0}%", c.name, c.value * 100.0))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// A component contributing to the final score.
#[derive(Debug, Clone)]
pub struct ScoreComponent {
    /// Component name (e.g., "Name similarity").
    pub name: &'static str,
    /// Component value (can be negative for penalties).
    pub value: f32,
    /// Human-readable description.
    pub description: String,
}

/// A suggested mapping from column to variable.
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Source column name.
    pub source_column: String,
    /// Target SDTM variable name.
    pub target_variable: String,
    /// Scoring details.
    pub score: ColumnScore,
}

/// Engine for scoring column-to-variable matches.
///
/// Uses Jaro-Winkler similarity as the base algorithm with adjustments for:
/// - Label similarity (boost if labels match well)
/// - Suffix patterns (SEQ, CD, etc.)
/// - Type compatibility (numeric vs character)
#[derive(Debug, Clone)]
pub struct ScoringEngine {
    domain: Domain,
    hints: BTreeMap<String, ColumnHint>,
}

impl ScoringEngine {
    /// Create a new scoring engine for a domain.
    pub fn new(domain: Domain, hints: BTreeMap<String, ColumnHint>) -> Self {
        Self { domain, hints }
    }

    /// Get the domain this engine is scoring against.
    pub fn domain(&self) -> &Domain {
        &self.domain
    }

    /// Score a single column against a specific variable.
    ///
    /// Returns `None` if the variable doesn't exist in the domain.
    pub fn score(&self, column: &str, variable_name: &str) -> Option<ColumnScore> {
        let variable = self
            .domain
            .variables
            .iter()
            .find(|v| v.name.eq_ignore_ascii_case(variable_name))?;

        Some(self.compute_score(column, variable))
    }

    /// Score all columns against a specific variable.
    ///
    /// Returns columns sorted by score (highest first).
    pub fn score_all_for_variable(
        &self,
        variable_name: &str,
        columns: &[String],
    ) -> Vec<(String, ColumnScore)> {
        let Some(variable) = self
            .domain
            .variables
            .iter()
            .find(|v| v.name.eq_ignore_ascii_case(variable_name))
        else {
            return Vec::new();
        };

        let mut scores: Vec<_> = columns
            .iter()
            .map(|col| (col.clone(), self.compute_score(col, variable)))
            .collect();

        scores.sort_by(|a, b| b.1.score.partial_cmp(&a.1.score).unwrap_or(Ordering::Equal));
        scores
    }

    /// Suggest best one-to-one mappings for all variables.
    ///
    /// Uses greedy assignment by descending score. Each column and variable
    /// is assigned at most once.
    pub fn suggest_all(&self, columns: &[String], min_confidence: f32) -> Vec<Suggestion> {
        // Score all column-variable pairs
        let mut candidates: Vec<(String, String, ColumnScore)> = Vec::new();

        for variable in &self.domain.variables {
            for column in columns {
                let score = self.compute_score(column, variable);
                if score.score >= min_confidence {
                    candidates.push((column.clone(), variable.name.clone(), score));
                }
            }
        }

        // Sort by score descending
        candidates.sort_by(|a, b| b.2.score.partial_cmp(&a.2.score).unwrap_or(Ordering::Equal));

        // Greedy one-to-one assignment
        let mut assigned_columns: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        let mut assigned_variables: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        let mut suggestions = Vec::new();

        for (column, variable, score) in candidates {
            if assigned_columns.contains(&column) || assigned_variables.contains(&variable) {
                continue;
            }

            assigned_columns.insert(column.clone());
            assigned_variables.insert(variable.clone());

            suggestions.push(Suggestion {
                source_column: column,
                target_variable: variable,
                score,
            });
        }

        suggestions
    }

    fn compute_score(&self, column: &str, variable: &Variable) -> ColumnScore {
        let mut components = Vec::new();

        // 1. Base: Jaro-Winkler similarity on normalized names
        let normalized_col = normalize(column);
        let normalized_var = normalize(&variable.name);

        let base = jaro_winkler::similarity(normalized_col.chars(), normalized_var.chars()) as f32;

        components.push(ScoreComponent {
            name: "Name similarity",
            value: base,
            description: format!("'{}' vs '{}'", column, variable.name),
        });

        let mut score = base;

        // 2. Label similarity boost (+10%)
        if let Some(hint) = self.hints.get(column) {
            if let (Some(col_label), Some(var_label)) = (&hint.label, &variable.label) {
                let label_sim = jaro_winkler::similarity(
                    normalize(col_label).chars(),
                    normalize(var_label).chars(),
                ) as f32;

                if label_sim > 0.85 {
                    score *= 1.10;
                    components.push(ScoreComponent {
                        name: "Label match",
                        value: 0.10,
                        description: format!("Labels match {:.0}%", label_sim * 100.0),
                    });
                }
            }
        }

        // 3. Suffix matching adjustments
        score = self.apply_suffix_adjustments(column, variable, score, &mut components);

        // 4. Type mismatch penalty (-15%)
        if let Some(hint) = self.hints.get(column) {
            let var_is_numeric = variable.name.ends_with('N');
            if var_is_numeric != hint.is_numeric {
                score *= 0.85;
                components.push(ScoreComponent {
                    name: "Type mismatch",
                    value: -0.15,
                    description: if var_is_numeric {
                        "Variable expects numeric, column is text".into()
                    } else {
                        "Variable expects text, column is numeric".into()
                    },
                });
            }
        }

        ColumnScore {
            score,
            explanation: components,
        }
    }

    fn apply_suffix_adjustments(
        &self,
        column: &str,
        variable: &Variable,
        mut score: f32,
        components: &mut Vec<ScoreComponent>,
    ) -> f32 {
        let col_upper = column.to_uppercase();
        let var_upper = variable.name.to_uppercase();

        // SEQ suffix matching
        if col_upper.ends_with("SEQ") {
            if var_upper.ends_with("SEQ") {
                score *= 1.05;
                components.push(ScoreComponent {
                    name: "SEQ match",
                    value: 0.05,
                    description: "Both have SEQ suffix".into(),
                });
            } else {
                score *= 0.6;
                components.push(ScoreComponent {
                    name: "SEQ mismatch",
                    value: -0.4,
                    description: "Column has SEQ but variable doesn't".into(),
                });
            }
        } else if var_upper.ends_with("SEQ") {
            score *= 0.6;
            components.push(ScoreComponent {
                name: "SEQ mismatch",
                value: -0.4,
                description: "Variable has SEQ but column doesn't".into(),
            });
        }

        // CD (code) suffix matching
        if col_upper.ends_with("CD") && !var_upper.ends_with("CD") {
            score *= 0.7;
            components.push(ScoreComponent {
                name: "CD mismatch",
                value: -0.3,
                description: "Column has CD suffix but variable doesn't".into(),
            });
        }
        if var_upper.ends_with("CD") && !col_upper.ends_with("CD") {
            score *= 0.8;
            components.push(ScoreComponent {
                name: "CD expected",
                value: -0.2,
                description: "Variable expects CD suffix".into(),
            });
        }

        score
    }
}

/// Normalize a string for comparison.
///
/// - Trims whitespace
/// - Converts to lowercase
/// - Replaces separators with spaces
fn normalize(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .replace(['_', '-', '.'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdtm_model::VariableType;

    fn make_variable(name: &str, label: Option<&str>) -> Variable {
        Variable {
            name: name.to_string(),
            label: label.map(String::from),
            data_type: VariableType::Char,
            length: None,
            role: None,
            core: None,
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
    fn test_exact_match_high_score() {
        let domain = make_domain(vec![make_variable("USUBJID", Some("Unique Subject ID"))]);
        let engine = ScoringEngine::new(domain, BTreeMap::new());

        let score = engine.score("USUBJID", "USUBJID").unwrap();
        assert!(score.score > 0.95, "Exact match should score > 0.95");
    }

    #[test]
    fn test_similar_names() {
        let domain = make_domain(vec![make_variable("USUBJID", Some("Unique Subject ID"))]);
        let engine = ScoringEngine::new(domain, BTreeMap::new());

        let score = engine.score("SUBJID", "USUBJID").unwrap();
        assert!(
            score.score > 0.8,
            "Similar names should score > 0.8, got {}",
            score.score
        );
    }

    #[test]
    fn test_seq_mismatch_penalty() {
        let domain = make_domain(vec![make_variable("AETERM", Some("AE Term"))]);
        let engine = ScoringEngine::new(domain, BTreeMap::new());

        let score = engine.score("AESEQ", "AETERM").unwrap();
        assert!(
            score.score < 0.7,
            "SEQ mismatch should reduce score, got {}",
            score.score
        );
    }

    #[test]
    fn test_suggest_all_one_to_one() {
        let domain = make_domain(vec![
            make_variable("USUBJID", None),
            make_variable("AETERM", None),
        ]);
        let engine = ScoringEngine::new(domain, BTreeMap::new());

        let columns = vec!["USUBJID".to_string(), "AETERM".to_string()];
        let suggestions = engine.suggest_all(&columns, 0.5);

        assert_eq!(suggestions.len(), 2);
        // Each column should map to its matching variable
        let mapped: std::collections::BTreeSet<_> =
            suggestions.iter().map(|s| &s.source_column).collect();
        assert!(mapped.contains(&"USUBJID".to_string()));
        assert!(mapped.contains(&"AETERM".to_string()));
    }

    #[test]
    fn test_explainability() {
        let domain = make_domain(vec![make_variable("USUBJID", Some("Unique Subject ID"))]);
        let engine = ScoringEngine::new(domain, BTreeMap::new());

        let score = engine.score("USUBJID", "USUBJID").unwrap();
        let explanation = score.explain();
        assert!(
            explanation.contains("Name similarity"),
            "Explanation should contain name similarity"
        );
    }
}
