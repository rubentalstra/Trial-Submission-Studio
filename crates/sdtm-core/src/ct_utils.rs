//! Controlled Terminology (CT) utilities for SDTM conformance.
//!
//! This module provides a unified API for CT normalization, resolution, and validation.
//!
//! # SDTMIG v3.4 Chapter 10 Reference
//!
//! Per Section 10.1, CDISC Controlled Terminology defines permissible values for
//! SDTM variables. CT normalization must:
//! - Use exact submission values from the codelist
//! - Apply synonym mappings when defined in the CT package
//! - For extensible codelists, allow non-standard values with proper documentation
//!
//! # Normalization Modes
//!
//! - **Strict**: Only exact matches or defined synonym mappings are allowed.
//!   Values not in CT return `None`, triggering validation errors.
//! - **Lenient**: Falls back to fuzzy matching for better data ingestion.
//!   Use only during mapping/preview, not for final outputs.

use sdtm_model::ct::Codelist;

// =============================================================================
// Core Types
// =============================================================================

/// Result of CT resolution with provenance information.
#[derive(Debug, Clone, PartialEq)]
pub enum CtResolution {
    /// Exact match to a submission value in the codelist.
    ExactMatch(String),
    /// Matched via a defined synonym mapping.
    SynonymMatch {
        submission_value: String,
        matched_synonym: String,
    },
    /// Matched via compact key comparison (alphanumeric normalization).
    CompactMatch(String),
    /// No match found - value is not in CT.
    NoMatch,
    /// Empty input value.
    Empty,
}

impl CtResolution {
    /// Returns the resolved submission value if found.
    pub fn submission_value(&self) -> Option<&str> {
        match self {
            Self::ExactMatch(v) => Some(v),
            Self::SynonymMatch {
                submission_value, ..
            } => Some(submission_value),
            Self::CompactMatch(v) => Some(v),
            Self::NoMatch | Self::Empty => None,
        }
    }
}

// =============================================================================
// Core Resolution Functions
// =============================================================================

/// Creates a compact key from a string by keeping only uppercase alphanumeric characters.
///
/// This is used for fuzzy matching when exact matching fails.
pub fn compact_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_uppercase())
        .collect()
}

/// Resolves a raw value against a controlled terminology codelist.
///
/// This is the primary CT resolution function. It attempts to match the input
/// in the following order:
///
/// 1. Exact submission value match or synonym lookup
/// 2. Compact key match against submission values
/// 3. Compact key match against preferred terms
///
/// # Arguments
///
/// * `ct` - The controlled terminology codelist to match against
/// * `raw` - The raw input value to resolve
///
/// # Returns
///
/// A `CtResolution` indicating the match type and resolved value.
pub fn resolve_ct_value(ct: &Codelist, raw: &str) -> CtResolution {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return CtResolution::Empty;
    }

    let key = trimmed.to_uppercase();

    // 1. Check if it's a valid value (exact or synonym)
    if ct.is_valid(trimmed) {
        let normalized = ct.normalize(trimmed);
        // Determine if this was exact or synonym match
        if key == normalized.to_uppercase() {
            return CtResolution::ExactMatch(normalized);
        } else {
            return CtResolution::SynonymMatch {
                submission_value: normalized,
                matched_synonym: trimmed.to_string(),
            };
        }
    }

    // 2. Check compact key match against submission values
    let input_compact = compact_key(trimmed);
    for submission in ct.submission_values() {
        if compact_key(submission) == input_compact {
            return CtResolution::CompactMatch(submission.to_string());
        }
    }

    // 3. Check compact key match against preferred terms
    for term in ct.terms.values() {
        if let Some(ref preferred) = term.preferred_term
            && compact_key(preferred) == input_compact
        {
            return CtResolution::CompactMatch(term.submission_value.clone());
        }
    }

    CtResolution::NoMatch
}

/// Resolves a CT value in strict mode (exact or synonym match only).
///
/// Returns `Some(submission_value)` only for exact matches or defined synonyms.
/// This is the preferred function for validation and final output generation.
pub fn resolve_ct_strict(ct: &Codelist, raw: &str) -> Option<String> {
    match resolve_ct_value(ct, raw) {
        CtResolution::ExactMatch(v)
        | CtResolution::SynonymMatch {
            submission_value: v,
            ..
        } => Some(v),
        _ => None,
    }
}

/// Resolves a CT value with lenient matching (includes compact key matching).
///
/// Returns `Some(submission_value)` for any successful match including fuzzy matches.
/// Use this for data ingestion and mapping suggestions, not for final outputs.
pub fn resolve_ct_lenient(ct: &Codelist, raw: &str) -> Option<String> {
    resolve_ct_value(ct, raw)
        .submission_value()
        .map(String::from)
}

// =============================================================================
// Normalization Functions
// =============================================================================

/// Normalizes a value against CT, returning the original if no match found.
///
/// This function:
/// 1. Attempts to resolve the value against CT
/// 2. If matched (strict), returns the submission value
/// 3. If no match, returns the original trimmed value
///
/// Use this when you want to normalize known values but preserve unknown ones.
pub fn normalize_ct_value(ct: &Codelist, raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Use strict matching for normalization
    resolve_ct_strict(ct, trimmed).unwrap_or_else(|| trimmed.to_string())
}

/// Normalizes a value against CT, only returning normalized value if it's in CT.
///
/// This function uses LENIENT matching:
/// 1. Attempts to resolve the value against CT (lenient - includes compact key matching)
/// 2. If the resolved value is a valid submission value, returns it
/// 3. Otherwise, returns the original trimmed value
///
/// This prevents normalizing to invalid values but allows fuzzy matching.
/// For strict-mode processing, use `normalize_ct_value_strict` instead.
pub fn normalize_ct_value_safe(ct: &Codelist, raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    match resolve_ct_value(ct, trimmed) {
        CtResolution::ExactMatch(v)
        | CtResolution::SynonymMatch {
            submission_value: v,
            ..
        }
        | CtResolution::CompactMatch(v)
            if ct.is_valid(&v) =>
        {
            v
        }
        _ => trimmed.to_string(),
    }
}

/// Normalizes a value against CT using STRICT matching only.
///
/// This function:
/// 1. Attempts to resolve the value against CT (strict - exact or synonym match only)
/// 2. If matched, returns the submission value
/// 3. Otherwise, returns the original trimmed value
///
/// Use this in strict mode when only exact matches or defined synonyms should
/// be normalized. Fuzzy/compact key matching is not used.
pub fn normalize_ct_value_strict(ct: &Codelist, raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    resolve_ct_strict(ct, trimmed).unwrap_or_else(|| trimmed.to_string())
}

// =============================================================================
// Lookup Functions
// =============================================================================

/// Gets the preferred term for a submission value.
pub fn preferred_term_for(ct: &Codelist, submission: &str) -> Option<String> {
    let key = submission.to_uppercase();
    ct.terms.get(&key).and_then(|t| t.preferred_term.clone())
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Checks if a value appears to be a yes/no token.
pub fn is_yes_no_token(value: &str) -> bool {
    matches!(
        value.trim().to_uppercase().as_str(),
        "Y" | "N" | "YES" | "NO" | "TRUE" | "FALSE" | "1" | "0"
    )
}

// =============================================================================
// Fuzzy Matching (for mapping suggestions only)
// =============================================================================

/// Computes the Levenshtein edit distance between two strings.
fn edit_distance(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    let a_len = a.len();
    let b_len = b.len();
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }
    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0usize; b_len + 1];
    for (i, a_ch) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, b_ch) in b.chars().enumerate() {
            let cost = if a_ch == b_ch { 0 } else { 1 };
            let insert = curr[j] + 1;
            let delete = prev[j + 1] + 1;
            let replace = prev[j] + cost;
            curr[j + 1] = insert.min(delete).min(replace);
        }
        prev.clone_from_slice(&curr);
    }
    prev[b_len]
}

/// Resolves a CT value from a hint using fuzzy matching.
///
/// This function is intended for mapping suggestions and column matching,
/// NOT for final CT normalization. It uses:
/// 1. Standard resolution (exact/synonym/compact)
/// 2. Substring matching
/// 3. Edit distance matching (threshold: 1)
///
/// # Warning
///
/// Do not use this for final output generation. Use `resolve_ct_strict` instead.
pub fn resolve_ct_value_from_hint(ct: &Codelist, hint: &str) -> Option<String> {
    // First try standard resolution
    if let Some(value) = resolve_ct_lenient(ct, hint) {
        return Some(value);
    }

    let hint_compact = compact_key(hint);
    if hint_compact.len() < 3 {
        return None;
    }

    // Try substring matching
    let mut matches: Vec<String> = Vec::new();
    for submission in ct.submission_values() {
        let compact = compact_key(submission);
        if compact.len() >= 3
            && (hint_compact.contains(&compact) || compact.contains(&hint_compact))
        {
            matches.push(submission.to_string());
        }
    }
    for term in ct.terms.values() {
        if let Some(ref preferred) = term.preferred_term {
            let compact = compact_key(preferred);
            if compact.len() >= 3
                && (hint_compact.contains(&compact) || compact.contains(&hint_compact))
            {
                matches.push(term.submission_value.clone());
            }
        }
        for synonym in &term.synonyms {
            let compact = compact_key(synonym);
            if compact.len() >= 3
                && (hint_compact.contains(&compact) || compact.contains(&hint_compact))
            {
                matches.push(term.submission_value.clone());
            }
        }
    }

    matches.sort();
    matches.dedup();
    if matches.len() == 1 {
        return Some(matches.remove(0));
    }

    // Try edit distance as last resort
    let mut best_dist = usize::MAX;
    let mut best_val: Option<String> = None;
    let mut best_count = 0usize;
    for submission in ct.submission_values() {
        let dist = edit_distance(&hint_compact, &compact_key(submission));
        if dist < best_dist {
            best_dist = dist;
            best_val = Some(submission.to_string());
            best_count = 1;
        } else if dist == best_dist {
            best_count += 1;
        }
    }
    if best_dist <= 1 && best_count == 1 {
        best_val
    } else {
        None
    }
}
