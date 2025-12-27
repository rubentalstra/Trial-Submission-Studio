use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use rapidfuzz::distance::jaro_winkler::similarity as jaro_similarity;

use sdtm_model::{ColumnHint, Domain, MappingConfig, MappingSuggestion, Variable};

use crate::patterns::build_variable_patterns;
use crate::utils::{normalize_text, safe_column_name};

const SEQ_UNIQUENESS_MIN: f64 = 0.5;
const REQUIRED_NULL_RATIO_MAX: f64 = 0.5;
const SEQ_MISMATCH_PENALTY: f64 = 0.4;
const CODE_MISMATCH_PENALTY: f64 = 0.6;
const CODE_EXPECTED_PENALTY: f64 = 0.7;
const TOKEN_NO_OVERLAP_PENALTY: f64 = 0.6;
const TOKEN_GENERIC_ONLY_PENALTY: f64 = 0.55;
const TOKEN_SPECIFIC_BOOST: f64 = 1.05;

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

struct Candidate {
    source_column: String,
    target_variable: String,
    confidence: f32,
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
        if let Some(hint) = hint {
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
        && let Some(label) = hint.label.as_deref() {
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
                && !normalized.is_empty() {
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
