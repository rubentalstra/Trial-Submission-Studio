//! Controlled Terminology (CT) normalization logic.

use sdtm_model::ct::Codelist;
use sdtm_model::options::{CtMatchingMode, NormalizationOptions};

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
pub fn resolve_ct_value(
    ct: &Codelist,
    raw: &str,
    options: &NormalizationOptions,
) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 1. Custom Mapping
    if !options.custom_maps.is_empty() {
        // Try exact match in custom map
        if let Some(mapped) = options.custom_maps.get(trimmed) {
            return Some(mapped.clone());
        }
        // Try uppercase match
        if let Some(mapped) = options.custom_maps.get(&trimmed.to_uppercase()) {
            return Some(mapped.clone());
        }
    }

    // 2. Standard Matching (Strict/Lenient)
    let resolved = match options.matching_mode {
        CtMatchingMode::Strict => resolve_ct_value_strict(ct, trimmed),
        CtMatchingMode::Lenient => resolve_ct_value_lenient(ct, trimmed),
    };

    if resolved.is_some() {
        return resolved;
    }

    // 3. Fallback Logic
    // Only if codelist is non-extensible (otherwise we should keep original as warning)
    if !ct.extensible {
        // Check for "OTHER" fallback
        if options.enable_other_fallback {
            // If the value is clearly an "other" value, or if we are just falling back everything
            // For now, we rely on the flag. If the flag is true, we try to map to OTHER.
            // But we should check if "OTHER" exists in the codelist.
            if ct.terms.contains_key("OTHER") {
                return Some("OTHER".to_string());
            }
        }

        // Check for "UNKNOWN" fallback
        if options.enable_unknown_fallback {
            // Simple heuristic for unknown values
            let upper = trimmed.to_uppercase();
            if upper == "UNKNOWN" || upper == "UNK" || upper == "?" || upper == "NOT REPORTED" {
                if ct.terms.contains_key("UNKNOWN") {
                    return Some("UNKNOWN".to_string());
                }
            }
        }
    }

    None
}

/// Normalize a CT value to its submission value when possible.
pub fn normalize_ct_value(ct: &Codelist, raw: &str, options: &NormalizationOptions) -> String {
    resolve_ct_value(ct, raw, options).unwrap_or_else(|| raw.trim().to_string())
}

/// Gets the preferred term for a submission value.
pub fn preferred_term_for(ct: &Codelist, submission: &str) -> Option<String> {
    let key = submission.to_uppercase();
    ct.terms.get(&key).and_then(|t| t.preferred_term.clone())
}

/// Checks if a value appears to be a yes/no token.
pub fn is_yes_no_token(value: &str) -> bool {
    matches!(
        value.trim().to_uppercase().as_str(),
        "Y" | "N" | "YES" | "NO" | "TRUE" | "FALSE" | "1" | "0"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdtm_model::ct::{Codelist, Term};

    fn create_race_codelist() -> Codelist {
        let mut ct = Codelist::new(
            "C74457".to_string(),
            "Race".to_string(),
            false, // Non-extensible
        );

        ct.add_term(Term {
            code: "C123".to_string(),
            submission_value: "WHITE".to_string(),
            preferred_term: None,
            synonyms: vec![],
            definition: Some("White race".to_string()),
        });

        ct.add_term(Term {
            code: "C456".to_string(),
            submission_value: "OTHER".to_string(),
            preferred_term: None,
            synonyms: vec![],
            definition: Some("Other race".to_string()),
        });

        ct
    }

    #[test]
    fn test_race_normalization_fallback() {
        let ct = create_race_codelist();
        let options = NormalizationOptions::new().with_other_fallback(true);

        // Valid value
        assert_eq!(normalize_ct_value(&ct, "WHITE", &options), "WHITE");

        // Invalid value mapping to OTHER
        assert_eq!(normalize_ct_value(&ct, "Caucasian", &options), "OTHER");
        assert_eq!(normalize_ct_value(&ct, "Arabic", &options), "OTHER");

        // Without fallback
        let strict_options = NormalizationOptions::new().with_other_fallback(false);
        assert_eq!(
            normalize_ct_value(&ct, "Caucasian", &strict_options),
            "Caucasian"
        );
    }
}
