//! Mapping engine implementation.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use rapidfuzz::distance::jaro_winkler::similarity as jaro_similarity;

use crate::types::{ColumnHint, MappingConfig, MappingSuggestion};
use sdtm_model::{Domain, Variable};

use crate::patterns::{build_synonym_map, build_variable_patterns, match_synonyms};
use crate::utils::{normalize_text, safe_column_name};

const SEQ_UNIQUENESS_MIN: f64 = 0.5;
const REQUIRED_NULL_RATIO_MAX: f64 = 0.5;
const SEQ_MISMATCH_PENALTY: f64 = 0.4;
const CODE_MISMATCH_PENALTY: f64 = 0.6;
const CODE_EXPECTED_PENALTY: f64 = 0.7;
const TOKEN_NO_OVERLAP_PENALTY: f64 = 0.6;
const TOKEN_GENERIC_ONLY_PENALTY: f64 = 0.55;
const TOKEN_SPECIFIC_BOOST: f64 = 1.05;
/// Boost for synonym-based matches
const SYNONYM_MATCH_BOOST: f64 = 1.15;
/// Boost for label-based matches
const LABEL_MATCH_BOOST: f64 = 1.10;

/// Confidence level categories for mapping quality assessment.
///
/// These levels help categorize mappings by their reliability:
/// - `High`: Near-certain matches that can be used without review
/// - `Medium`: Good matches that should be verified
/// - `Low`: Weak matches requiring manual confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidenceLevel {
    /// Low confidence (≥ low threshold, < medium threshold).
    /// These mappings are uncertain and require manual verification.
    Low,
    /// Medium confidence (≥ medium threshold, < high threshold).
    /// These mappings are reasonable but should be reviewed.
    Medium,
    /// High confidence (≥ high threshold).
    /// These mappings are near-certain and typically correct.
    High,
}

impl ConfidenceLevel {
    /// Returns a human-readable description of the confidence level.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::High => "high confidence - likely correct",
            Self::Medium => "medium confidence - should review",
            Self::Low => "low confidence - needs verification",
        }
    }
}

/// Configurable thresholds for categorizing mapping confidence.
///
/// The thresholds define boundaries between confidence levels:
/// - Below `low`: rejected (not included in results)
/// - `low` to `medium`: [`ConfidenceLevel::Low`]
/// - `medium` to `high`: [`ConfidenceLevel::Medium`]
/// - At or above `high`: [`ConfidenceLevel::High`]
#[derive(Debug, Clone, Copy)]
pub struct ConfidenceThresholds {
    /// Minimum confidence for high-quality matches (default: 0.95).
    pub high: f32,
    /// Minimum confidence for medium-quality matches (default: 0.80).
    pub medium: f32,
    /// Minimum confidence to include in results (default: 0.60).
    pub low: f32,
}

impl Default for ConfidenceThresholds {
    fn default() -> Self {
        Self {
            high: 0.95,
            medium: 0.80,
            low: 0.60,
        }
    }
}

impl ConfidenceThresholds {
    /// Creates thresholds with strict boundaries for high-quality mapping.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            high: 0.98,
            medium: 0.90,
            low: 0.75,
        }
    }

    /// Creates thresholds with relaxed boundaries for exploratory mapping.
    #[must_use]
    pub fn relaxed() -> Self {
        Self {
            high: 0.90,
            medium: 0.70,
            low: 0.50,
        }
    }

    /// Categorizes a confidence score into a confidence level.
    ///
    /// Returns `None` if the score is below the low threshold.
    #[must_use]
    pub fn categorize(&self, confidence: f32) -> Option<ConfidenceLevel> {
        if confidence >= self.high {
            Some(ConfidenceLevel::High)
        } else if confidence >= self.medium {
            Some(ConfidenceLevel::Medium)
        } else if confidence >= self.low {
            Some(ConfidenceLevel::Low)
        } else {
            None
        }
    }
}

/// Result of a mapping operation.
#[derive(Debug, Clone)]
pub struct MappingResult {
    /// Successfully mapped column-to-variable suggestions.
    pub mappings: Vec<MappingSuggestion>,
    /// Columns that could not be mapped above the confidence threshold.
    pub unmapped_columns: Vec<String>,
}

impl MappingResult {
    /// Returns the count of mappings at each confidence level.
    ///
    /// Uses default thresholds. For custom thresholds, use [`Self::count_by_level_with`].
    #[must_use]
    pub fn count_by_level(&self) -> BTreeMap<ConfidenceLevel, usize> {
        self.count_by_level_with(&ConfidenceThresholds::default())
    }

    /// Returns the count of mappings at each confidence level using custom thresholds.
    #[must_use]
    pub fn count_by_level_with(
        &self,
        thresholds: &ConfidenceThresholds,
    ) -> BTreeMap<ConfidenceLevel, usize> {
        let mut counts = BTreeMap::new();
        for mapping in &self.mappings {
            if let Some(level) = thresholds.categorize(mapping.confidence) {
                *counts.entry(level).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Filters mappings to only those at or above the specified confidence level.
    ///
    /// Uses default thresholds. For custom thresholds, use [`Self::filter_by_level_with`].
    #[must_use]
    pub fn filter_by_level(&self, min_level: ConfidenceLevel) -> Vec<&MappingSuggestion> {
        self.filter_by_level_with(min_level, &ConfidenceThresholds::default())
    }

    /// Filters mappings to only those at or above the specified confidence level
    /// using custom thresholds.
    #[must_use]
    pub fn filter_by_level_with(
        &self,
        min_level: ConfidenceLevel,
        thresholds: &ConfidenceThresholds,
    ) -> Vec<&MappingSuggestion> {
        self.mappings
            .iter()
            .filter(|m| {
                thresholds
                    .categorize(m.confidence)
                    .is_some_and(|level| level >= min_level)
            })
            .collect()
    }

    /// Returns mappings grouped by their confidence level.
    ///
    /// Uses default thresholds.
    #[must_use]
    pub fn group_by_level(&self) -> BTreeMap<ConfidenceLevel, Vec<&MappingSuggestion>> {
        self.group_by_level_with(&ConfidenceThresholds::default())
    }

    /// Returns mappings grouped by their confidence level using custom thresholds.
    #[must_use]
    pub fn group_by_level_with(
        &self,
        thresholds: &ConfidenceThresholds,
    ) -> BTreeMap<ConfidenceLevel, Vec<&MappingSuggestion>> {
        let mut groups: BTreeMap<ConfidenceLevel, Vec<&MappingSuggestion>> = BTreeMap::new();
        for mapping in &self.mappings {
            if let Some(level) = thresholds.categorize(mapping.confidence) {
                groups.entry(level).or_default().push(mapping);
            }
        }
        groups
    }

    /// Returns the minimum confidence score among all mappings, if any.
    #[must_use]
    pub fn min_confidence(&self) -> Option<f32> {
        self.mappings
            .iter()
            .map(|m| m.confidence)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
    }

    /// Returns the maximum confidence score among all mappings, if any.
    #[must_use]
    pub fn max_confidence(&self) -> Option<f32> {
        self.mappings
            .iter()
            .map(|m| m.confidence)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
    }

    /// Returns the mean confidence score among all mappings, if any.
    #[must_use]
    pub fn mean_confidence(&self) -> Option<f32> {
        if self.mappings.is_empty() {
            return None;
        }
        let sum: f32 = self.mappings.iter().map(|m| m.confidence).sum();
        Some(sum / self.mappings.len() as f32)
    }

    /// Returns true if all mappings are at high confidence level.
    #[must_use]
    pub fn all_high_confidence(&self) -> bool {
        self.all_high_confidence_with(&ConfidenceThresholds::default())
    }

    /// Returns true if all mappings are at high confidence level using custom thresholds.
    #[must_use]
    pub fn all_high_confidence_with(&self, thresholds: &ConfidenceThresholds) -> bool {
        !self.mappings.is_empty()
            && self
                .mappings
                .iter()
                .all(|m| thresholds.categorize(m.confidence) == Some(ConfidenceLevel::High))
    }
}

/// Engine for mapping source columns to SDTM domain variables.
///
/// The engine uses fuzzy string matching combined with metadata hints
/// to suggest the best mapping between source data columns and target
/// SDTM variables. It produces confidence scores for each mapping,
/// allowing downstream processing to filter or prioritize results.
///
/// # Example
///
/// ```ignore
/// use sdtm_map::MappingEngine;
/// use std::collections::BTreeMap;
///
/// let engine = MappingEngine::new(domain, 0.6, BTreeMap::new());
/// let result = engine.suggest(&["STUDYID".to_string(), "AGE".to_string()]);
/// ```
pub struct MappingEngine {
    domain: Domain,
    min_confidence: f32,
    column_hints: BTreeMap<String, ColumnHint>,
    variable_patterns: BTreeMap<String, Vec<String>>,
    synonym_map: BTreeMap<String, Vec<String>>,
}

struct Candidate {
    source_column: String,
    target_variable: String,
    confidence: f32,
}

impl MappingEngine {
    /// Creates a new mapping engine for a specific domain.
    ///
    /// # Arguments
    ///
    /// * `domain` - The SDTM domain definition containing target variables
    /// * `min_confidence` - Minimum confidence score (0.0-1.0) for a mapping to be included
    /// * `column_hints` - Optional metadata about source columns (type, uniqueness, etc.)
    pub fn new(
        domain: Domain,
        min_confidence: f32,
        column_hints: BTreeMap<String, ColumnHint>,
    ) -> Self {
        let variable_patterns = build_variable_patterns(&domain);
        let synonym_map = build_synonym_map(&domain);
        Self {
            domain,
            min_confidence,
            column_hints,
            variable_patterns,
            synonym_map,
        }
    }

    /// Suggests mappings for a list of source column names.
    ///
    /// Returns a [`MappingResult`] containing:
    /// - Suggested mappings with confidence scores
    /// - Columns that could not be mapped above the minimum confidence threshold
    ///
    /// The engine performs one-to-one matching: each source column maps to at most
    /// one target variable, and each target variable is assigned at most once.
    pub fn suggest(&self, columns: &[String]) -> MappingResult {
        let mut suggestions = Vec::new();
        let mut assigned_targets = BTreeSet::new();
        let mut assigned_columns = BTreeSet::new();
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
                assigned_columns.insert(column.clone());
                continue;
            }
        }

        let mut column_tokens: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
        for column in columns {
            if assigned_columns.contains(column) || alias_collisions.contains(column) {
                continue;
            }
            let hint = self.column_hints.get(column);
            let tokens = column_token_set(column, hint);
            column_tokens.insert(column.as_str(), tokens);
        }

        let mut variable_tokens: Vec<BTreeSet<String>> = Vec::new();
        for variable in &self.domain.variables {
            variable_tokens.push(variable_token_set(variable));
        }

        let mut candidates: Vec<Candidate> = Vec::new();
        for column in columns {
            if assigned_columns.contains(column) || alias_collisions.contains(column) {
                continue;
            }
            let hint = self.column_hints.get(column);
            let col_tokens = column_tokens
                .get(column.as_str())
                .cloned()
                .unwrap_or_default();
            for (idx, variable) in self.domain.variables.iter().enumerate() {
                let var_tokens = &variable_tokens[idx];
                let confidence =
                    self.score_candidate(column, variable, hint, &col_tokens, var_tokens);
                candidates.push(Candidate {
                    source_column: column.clone(),
                    target_variable: variable.name.clone(),
                    confidence,
                });
            }
        }

        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(Ordering::Equal)
        });

        for candidate in candidates {
            if candidate.confidence < self.min_confidence {
                break;
            }
            if assigned_targets.contains(&candidate.target_variable)
                || assigned_columns.contains(&candidate.source_column)
            {
                continue;
            }
            assigned_targets.insert(candidate.target_variable.clone());
            assigned_columns.insert(candidate.source_column.clone());
            suggestions.push(MappingSuggestion {
                source_column: safe_column_name(&candidate.source_column),
                target_variable: candidate.target_variable,
                confidence: candidate.confidence,
                transformation: None,
            });
        }

        let mut unmapped = Vec::new();
        for column in columns {
            if alias_collisions.contains(column) {
                unmapped.push(column.clone());
                continue;
            }
            if !assigned_columns.contains(column) {
                unmapped.push(column.clone());
            }
        }

        MappingResult {
            mappings: suggestions,
            unmapped_columns: unmapped,
        }
    }

    /// Converts a mapping result into a [`MappingConfig`] for persistence or further processing.
    ///
    /// # Arguments
    ///
    /// * `study_id` - The study identifier to include in the config
    /// * `result` - The mapping result to convert
    pub fn to_config(&self, study_id: &str, result: MappingResult) -> MappingConfig {
        MappingConfig {
            domain_code: self.domain.name.clone(),
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
        if let Some(target) = self.date_alias_override(column) {
            return Some(target);
        }
        if let Some(target) = self.seq_alias_override(column) {
            return Some(target);
        }
        None
    }

    fn date_alias_override(&self, column: &str) -> Option<String> {
        let upper = column.trim().to_uppercase();
        if upper.ends_with("DTC") {
            return None;
        }
        for suffix in ["DATE", "DAT", "DT"] {
            if upper.ends_with(suffix) && upper.len() > suffix.len() {
                let prefix = &upper[..upper.len() - suffix.len()];
                let candidate = format!("{prefix}DTC");
                if self
                    .domain
                    .variables
                    .iter()
                    .any(|var| var.name.eq_ignore_ascii_case(&candidate))
                {
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn seq_alias_override(&self, column: &str) -> Option<String> {
        let compact = normalize_text(column).replace(' ', "").to_uppercase();
        if !compact.ends_with("SEQ") {
            return None;
        }
        let seq_vars: Vec<&str> = self
            .domain
            .variables
            .iter()
            .map(|var| var.name.as_str())
            .filter(|name| name.to_uppercase().ends_with("SEQ"))
            .collect();
        if seq_vars.len() == 1 {
            return Some(seq_vars[0].to_string());
        }
        None
    }

    fn score_candidate(
        &self,
        column: &str,
        variable: &Variable,
        hint: Option<&ColumnHint>,
        column_tokens: &BTreeSet<String>,
        variable_tokens: &BTreeSet<String>,
    ) -> f32 {
        let normalized = normalize_text(column);
        let score_raw = jaro_similarity(column.to_uppercase().chars(), variable.name.chars());
        let score_norm = jaro_similarity(normalized.chars(), variable.name.to_lowercase().chars());
        let mut score = score_raw.max(score_norm);

        // Check for synonym match and apply boost
        let label = hint.and_then(|h| h.label.as_deref());
        let synonym_matches = match_synonyms(column, label, &self.synonym_map);
        if synonym_matches
            .iter()
            .any(|t| t.eq_ignore_ascii_case(&variable.name))
        {
            score *= SYNONYM_MATCH_BOOST;
        }

        // Check for label similarity with variable label
        if let Some(hint) = hint {
            if let Some(col_label) = &hint.label
                && let Some(var_label) = &variable.label
            {
                let label_score = jaro_similarity(
                    normalize_text(col_label).chars(),
                    normalize_text(var_label).chars(),
                );
                if label_score > 0.85 {
                    score *= LABEL_MATCH_BOOST;
                }
            }
            score = self.apply_hints(
                column,
                variable,
                hint,
                column_tokens,
                variable_tokens,
                score,
            );
        }
        score as f32
    }

    fn apply_hints(
        &self,
        column: &str,
        variable: &Variable,
        hint: &ColumnHint,
        column_tokens: &BTreeSet<String>,
        variable_tokens: &BTreeSet<String>,
        score: f64,
    ) -> f64 {
        let mut adjusted = score;
        let col_upper = column.trim().to_uppercase();
        let var_upper = variable.name.to_uppercase();
        if col_upper.ends_with("SEQ") {
            if var_upper.ends_with("SEQ") {
                adjusted *= 1.05;
            } else {
                adjusted *= SEQ_MISMATCH_PENALTY;
            }
        }
        if col_upper.ends_with("CD") && !var_upper.ends_with("CD") {
            adjusted *= CODE_MISMATCH_PENALTY;
        }
        if var_upper.ends_with("CD") && !col_upper.ends_with("CD") {
            adjusted *= CODE_EXPECTED_PENALTY;
        }
        let is_numeric_var = var_upper.ends_with('N');
        if is_numeric_var != hint.is_numeric {
            adjusted *= 0.85;
        }
        if var_upper.ends_with("SEQ") && hint.unique_ratio < SEQ_UNIQUENESS_MIN {
            adjusted *= 0.9;
        }
        if hint.null_ratio > REQUIRED_NULL_RATIO_MAX {
            adjusted *= 0.9;
        }
        adjusted = overlap_adjustment(column_tokens, variable_tokens, adjusted);
        adjusted
    }
}

fn overlap_adjustment(
    column_tokens: &BTreeSet<String>,
    variable_tokens: &BTreeSet<String>,
    score: f64,
) -> f64 {
    let overlap: BTreeSet<String> = column_tokens
        .intersection(variable_tokens)
        .cloned()
        .collect();
    if overlap.is_empty() {
        return score * TOKEN_NO_OVERLAP_PENALTY;
    }
    let mut specific = false;
    for token in &overlap {
        if !is_generic_token(token) {
            specific = true;
            break;
        }
    }
    if !specific && overlap.len() <= 1 {
        return score * TOKEN_GENERIC_ONLY_PENALTY;
    }
    score * TOKEN_SPECIFIC_BOOST
}

fn column_token_set(column: &str, hint: Option<&ColumnHint>) -> BTreeSet<String> {
    let mut tokens = token_set(column);
    if let Some(hint) = hint
        && let Some(label) = hint.label.as_deref()
    {
        tokens.extend(token_set(label));
    }
    tokens
}

fn variable_token_set(variable: &Variable) -> BTreeSet<String> {
    let mut tokens = token_set(&variable.name);
    if let Some(label) = variable.label.as_deref() {
        tokens.extend(token_set(label));
    }
    tokens
}

fn token_set(raw: &str) -> BTreeSet<String> {
    let mut normalized = String::new();
    let mut prev_lower = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            if prev_lower && ch.is_ascii_uppercase() {
                normalized.push(' ');
            }
            normalized.push(ch);
            prev_lower = ch.is_ascii_lowercase();
        } else {
            normalized.push(' ');
            prev_lower = false;
        }
    }
    let mut tokens = BTreeSet::new();
    for raw_token in normalized.split_whitespace() {
        let token = raw_token.to_ascii_lowercase();
        for part in split_suffixes(&token) {
            if let Some(normalized) = normalize_token(&part)
                && !normalized.is_empty()
            {
                tokens.insert(normalized);
            }
        }
    }
    tokens
}

fn split_suffixes(token: &str) -> Vec<String> {
    const SUFFIXES: [&str; 12] = [
        "dtc",
        "date",
        "dat",
        "dt",
        "seq",
        "sequence",
        "id",
        "identifier",
        "ident",
        "cd",
        "code",
        "name",
    ];
    for suffix in SUFFIXES {
        if token.len() > suffix.len() && token.ends_with(suffix) {
            let base = &token[..token.len() - suffix.len()];
            return vec![base.to_string(), suffix.to_string()];
        }
    }
    vec![token.to_string()]
}

fn normalize_token(token: &str) -> Option<String> {
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    if is_stopword(token) {
        return None;
    }
    let mapped = match token {
        "subj" | "subject" | "subjid" | "subjectid" | "usubjid" => "subject",
        "code" | "cd" | "identifier" | "ident" | "id" => "id",
        "seq" | "sequence" => "seq",
        "dtc" | "dt" | "dat" | "date" | "datetime" | "time" => "date",
        "nam" | "name" => "name",
        "inv" | "investigator" => "investigator",
        _ => token,
    };
    if is_stopword(mapped) {
        return None;
    }
    Some(mapped.to_string())
}

fn is_generic_token(token: &str) -> bool {
    matches!(token, "id" | "seq" | "date" | "name" | "number" | "num")
}

fn is_stopword(token: &str) -> bool {
    matches!(
        token,
        "of" | "and"
            | "the"
            | "to"
            | "for"
            | "in"
            | "on"
            | "at"
            | "with"
            | "by"
            | "from"
            | "or"
            | "a"
            | "an"
    )
}
