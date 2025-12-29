//! Controlled Terminology (CT) utilities for SDTM conformance.
//!
//! # SDTMIG v3.4 Chapter 10 Reference
//!
//! SDTM controlled terminology must use CDISC submission values. This module
//! normalizes values to submission values using the CT package and preserves
//! unknown values without inventing replacements.

use sdtm_model::ct::Codelist;

use crate::pipeline_context::CtMatchingMode;

/// Creates a compact key by keeping only uppercase alphanumeric characters.
fn compact_key(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|ch| ch.to_ascii_uppercase())
        .collect()
}

fn resolve_ct_value_strict(ct: &Codelist, trimmed: &str) -> Option<String> {
    if ct.is_valid(trimmed) {
        Some(ct.normalize(trimmed))
    } else {
        None
    }
}

fn resolve_ct_value_lenient(ct: &Codelist, trimmed: &str) -> Option<String> {
    if let Some(value) = resolve_ct_value_strict(ct, trimmed) {
        return Some(value);
    }
    let input_compact = compact_key(trimmed);
    if input_compact.is_empty() {
        return None;
    }
    for submission in ct.submission_values() {
        if compact_key(submission) == input_compact {
            return Some(submission.to_string());
        }
    }
    None
}

/// Resolve a raw value against a codelist based on matching mode.
pub(crate) fn resolve_ct_value(ct: &Codelist, raw: &str, mode: CtMatchingMode) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    match mode {
        CtMatchingMode::Strict => resolve_ct_value_strict(ct, trimmed),
        CtMatchingMode::Lenient => resolve_ct_value_lenient(ct, trimmed),
    }
}

/// Normalize a CT value to its submission value when possible.
pub(crate) fn normalize_ct_value(ct: &Codelist, raw: &str, mode: CtMatchingMode) -> String {
    resolve_ct_value(ct, raw, mode).unwrap_or_else(|| raw.trim().to_string())
}

/// Gets the preferred term for a submission value.
pub(crate) fn preferred_term_for(ct: &Codelist, submission: &str) -> Option<String> {
    let key = submission.to_uppercase();
    ct.terms.get(&key).and_then(|t| t.preferred_term.clone())
}

/// Checks if a value appears to be a yes/no token.
pub(crate) fn is_yes_no_token(value: &str) -> bool {
    matches!(
        value.trim().to_uppercase().as_str(),
        "Y" | "N" | "YES" | "NO" | "TRUE" | "FALSE" | "1" | "0"
    )
}
