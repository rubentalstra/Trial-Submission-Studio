//! Controlled Terminology (CT) normalization per SDTM standards.
//!
//! CT normalization behavior differs based on codelist extensibility:
//! - **Non-extensible**: Only CDISC values allowed. Invalid = ERROR (compliance violation)
//! - **Extensible**: Sponsor values allowed. Invalid = INFO (valid sponsor extension)

use sdtm_model::ct::Codelist;

/// Result of CT normalization.
#[derive(Debug, Clone, PartialEq)]
pub struct CtNormalizationResult {
    /// The normalized (or original) value.
    pub value: String,

    /// Whether the value was found in the codelist.
    pub found: bool,

    /// True if this is a compliance violation (non-extensible and not found).
    pub is_error: bool,
}

impl CtNormalizationResult {
    /// Create a result for a found value.
    pub fn found(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            found: true,
            is_error: false,
        }
    }

    /// Create a result for a not-found value in an extensible codelist.
    pub fn not_found_extensible(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            found: false,
            is_error: false,
        }
    }

    /// Create a result for a not-found value in a non-extensible codelist.
    pub fn not_found_non_extensible(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            found: false,
            is_error: true,
        }
    }
}

/// Normalize a value using controlled terminology codelist.
///
/// # Behavior
///
/// 1. If value is found (term or synonym): Returns canonical submission value
/// 2. If value not found:
///    - **Non-extensible codelist**: Preserves original + is_error=true (compliance violation)
///    - **Extensible codelist**: Preserves exact case + is_error=false (valid sponsor extension)
///
/// # Arguments
/// * `value` - The value to normalize
/// * `codelist` - The codelist to normalize against
///
/// # Examples
///
/// ```ignore
/// // SEX (C66731) is non-extensible
/// let sex_codelist = load_codelist("C66731");
/// let result = normalize_ct_value("male", &sex_codelist);
/// assert_eq!(result.value, "M"); // Normalized via synonym
/// assert!(result.found);
///
/// let result = normalize_ct_value("X", &sex_codelist);
/// assert_eq!(result.value, "X"); // Preserved original
/// assert!(!result.found);
/// assert!(result.is_error); // Non-extensible: compliance violation
/// ```
pub fn normalize_ct_value(value: &str, codelist: &Codelist) -> CtNormalizationResult {
    let trimmed = value.trim();

    // Empty values pass through
    if trimmed.is_empty() {
        return CtNormalizationResult::found(String::new());
    }

    // Try to normalize (case-insensitive lookup)
    if codelist.is_valid(trimmed) {
        // Found - return canonical value
        let normalized = codelist.normalize(trimmed);
        return CtNormalizationResult::found(normalized);
    }

    // Not found - behavior depends on extensibility
    if codelist.extensible {
        // Extensible: preserve exact case, log as info (valid sponsor extension)
        tracing::info!(
            codelist = %codelist.code,
            codelist_name = %codelist.name,
            value = %trimmed,
            "CT value not in codelist (extensible - valid sponsor extension)"
        );
        CtNormalizationResult::not_found_extensible(trimmed)
    } else {
        // Non-extensible: preserve original, log as error (compliance violation)
        tracing::error!(
            codelist = %codelist.code,
            codelist_name = %codelist.name,
            value = %trimmed,
            "CT value not in codelist (non-extensible - SDTM compliance violation)"
        );
        CtNormalizationResult::not_found_non_extensible(trimmed)
    }
}

/// Normalize a value without a codelist available.
///
/// When the codelist cannot be found in the registry, we preserve
/// the original value and log a warning.
pub fn normalize_without_codelist(value: &str, codelist_code: &str) -> CtNormalizationResult {
    let trimmed = value.trim();

    tracing::warn!(
        codelist = %codelist_code,
        value = %trimmed,
        "Codelist not found in registry - preserving original value"
    );

    // Cannot determine if extensible, so don't mark as error
    CtNormalizationResult {
        value: trimmed.to_string(),
        found: false,
        is_error: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdtm_model::ct::Term;

    fn create_test_codelist(extensible: bool) -> Codelist {
        let mut codelist = Codelist::new("C66731".to_string(), "Sex".to_string(), extensible);

        codelist.add_term(Term {
            code: "C16576".to_string(),
            submission_value: "F".to_string(),
            synonyms: vec!["FEMALE".to_string()],
            definition: Some("Female".to_string()),
            preferred_term: Some("Female".to_string()),
        });

        codelist.add_term(Term {
            code: "C20197".to_string(),
            submission_value: "M".to_string(),
            synonyms: vec!["MALE".to_string()],
            definition: Some("Male".to_string()),
            preferred_term: Some("Male".to_string()),
        });

        codelist.add_term(Term {
            code: "C38046".to_string(),
            submission_value: "U".to_string(),
            synonyms: vec!["UNKNOWN".to_string(), "UNK".to_string()],
            definition: Some("Unknown".to_string()),
            preferred_term: Some("Unknown".to_string()),
        });

        codelist
    }

    #[test]
    fn test_found_exact_match() {
        let codelist = create_test_codelist(false);
        let result = normalize_ct_value("M", &codelist);
        assert_eq!(result.value, "M");
        assert!(result.found);
        assert!(!result.is_error);
    }

    #[test]
    fn test_found_case_insensitive() {
        let codelist = create_test_codelist(false);
        let result = normalize_ct_value("m", &codelist);
        assert_eq!(result.value, "M");
        assert!(result.found);
        assert!(!result.is_error);
    }

    #[test]
    fn test_found_via_synonym() {
        let codelist = create_test_codelist(false);
        let result = normalize_ct_value("male", &codelist);
        assert_eq!(result.value, "M");
        assert!(result.found);
        assert!(!result.is_error);
    }

    #[test]
    fn test_not_found_non_extensible() {
        let codelist = create_test_codelist(false);
        let result = normalize_ct_value("X", &codelist);
        assert_eq!(result.value, "X"); // Preserved
        assert!(!result.found);
        assert!(result.is_error); // Non-extensible: compliance violation
    }

    #[test]
    fn test_not_found_extensible() {
        let codelist = create_test_codelist(true);
        let result = normalize_ct_value("CustomValue", &codelist);
        assert_eq!(result.value, "CustomValue"); // Preserved exact case
        assert!(!result.found);
        assert!(!result.is_error); // Extensible: valid sponsor extension
    }

    #[test]
    fn test_empty_value() {
        let codelist = create_test_codelist(false);
        let result = normalize_ct_value("", &codelist);
        assert_eq!(result.value, "");
        assert!(result.found);
        assert!(!result.is_error);
    }

    #[test]
    fn test_whitespace_trimming() {
        let codelist = create_test_codelist(false);
        let result = normalize_ct_value("  M  ", &codelist);
        assert_eq!(result.value, "M");
        assert!(result.found);
        assert!(!result.is_error);
    }
}
