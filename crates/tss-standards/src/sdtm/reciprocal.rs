//! RELSUB reciprocal relationship lookups.
//!
//! Per SDTM-IG v3.4 Section 8.7, RELSUB relationships MUST be bidirectional.
//! This module provides lookup functions for reciprocal SREL terms.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Lookup table for reciprocal SREL terms.
///
/// Per SDTM-IG v3.4:
/// - If A is MOTHER to B, then B is CHILD to A
/// - If A is FATHER to B, then B is CHILD to A
/// - Twin relationships are symmetric (TWIN to TWIN)
/// - Sibling relationships are symmetric (SIBLING to SIBLING)
static RECIPROCAL_SREL: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Parent-child relationships (biological)
    map.insert("MOTHER, BIOLOGICAL", "CHILD, BIOLOGICAL");
    map.insert("FATHER, BIOLOGICAL", "CHILD, BIOLOGICAL");
    // Note: CHILD, BIOLOGICAL reciprocal depends on parent's sex - handled specially

    // Parent-child relationships (adoptive)
    map.insert("MOTHER, ADOPTIVE", "CHILD, ADOPTIVE");
    map.insert("FATHER, ADOPTIVE", "CHILD, ADOPTIVE");

    // Parent-child relationships (foster)
    map.insert("MOTHER, FOSTER", "CHILD, FOSTER");
    map.insert("FATHER, FOSTER", "CHILD, FOSTER");

    // Parent-child relationships (step)
    map.insert("MOTHER, STEP", "CHILD, STEP");
    map.insert("FATHER, STEP", "CHILD, STEP");

    // Symmetric relationships (twin types)
    map.insert("TWIN, DIZYGOTIC", "TWIN, DIZYGOTIC");
    map.insert("TWIN, MONOZYGOTIC", "TWIN, MONOZYGOTIC");
    map.insert("TWIN, UNKNOWN ZYGOSITY", "TWIN, UNKNOWN ZYGOSITY");

    // Symmetric relationships (sibling)
    map.insert("SIBLING", "SIBLING");
    map.insert("SIBLING, BIOLOGICAL", "SIBLING, BIOLOGICAL");
    map.insert("SIBLING, HALF", "SIBLING, HALF");
    map.insert("SIBLING, STEP", "SIBLING, STEP");
    map.insert("SIBLING, ADOPTIVE", "SIBLING, ADOPTIVE");

    // Grandparent relationships
    map.insert("GRANDMOTHER, BIOLOGICAL", "GRANDCHILD, BIOLOGICAL");
    map.insert("GRANDFATHER, BIOLOGICAL", "GRANDCHILD, BIOLOGICAL");
    map.insert("GRANDMOTHER, ADOPTIVE", "GRANDCHILD, ADOPTIVE");
    map.insert("GRANDFATHER, ADOPTIVE", "GRANDCHILD, ADOPTIVE");

    // Spouse relationships (symmetric)
    map.insert("SPOUSE", "SPOUSE");
    map.insert("HUSBAND", "WIFE");
    map.insert("WIFE", "HUSBAND");

    // Other family relationships
    map.insert("AUNT, BIOLOGICAL", "NEPHEW/NIECE, BIOLOGICAL");
    map.insert("UNCLE, BIOLOGICAL", "NEPHEW/NIECE, BIOLOGICAL");
    map.insert("COUSIN, BIOLOGICAL", "COUSIN, BIOLOGICAL");

    map
});

/// Get the reciprocal SREL term for a given relationship.
///
/// Returns `Some(reciprocal)` if a known reciprocal exists, or `None` if:
/// - The relationship is not in the lookup table
/// - The relationship requires context (e.g., CHILD needs parent's sex)
///
/// # Examples
///
/// ```
/// use tss_standards::sdtm::get_reciprocal_srel;
///
/// assert_eq!(get_reciprocal_srel("MOTHER, BIOLOGICAL"), Some("CHILD, BIOLOGICAL"));
/// assert_eq!(get_reciprocal_srel("TWIN, MONOZYGOTIC"), Some("TWIN, MONOZYGOTIC"));
/// assert_eq!(get_reciprocal_srel("SIBLING"), Some("SIBLING"));
/// ```
pub fn get_reciprocal_srel(srel: &str) -> Option<&'static str> {
    RECIPROCAL_SREL.get(srel.trim()).copied()
}

/// Check if a relationship is symmetric (same term for both directions).
///
/// # Examples
///
/// ```
/// use tss_standards::sdtm::is_symmetric_srel;
///
/// assert!(is_symmetric_srel("TWIN, DIZYGOTIC"));
/// assert!(is_symmetric_srel("SIBLING"));
/// assert!(!is_symmetric_srel("MOTHER, BIOLOGICAL"));
/// ```
pub fn is_symmetric_srel(srel: &str) -> bool {
    RECIPROCAL_SREL
        .get(srel.trim())
        .is_some_and(|&reciprocal| reciprocal == srel.trim())
}

/// Get the reciprocal for a CHILD relationship based on parent's biological sex.
///
/// CHILD relationships are special because the reciprocal depends on
/// the parent's sex (MOTHER vs FATHER).
///
/// # Arguments
///
/// * `child_type` - The child relationship type (e.g., "BIOLOGICAL", "ADOPTIVE")
/// * `parent_sex` - The parent's SEX value ("M" or "F")
///
/// # Returns
///
/// The appropriate parent term (e.g., "MOTHER, BIOLOGICAL" or "FATHER, BIOLOGICAL")
pub fn get_parent_srel_for_child(child_type: &str, parent_sex: &str) -> Option<&'static str> {
    let parent_prefix = match parent_sex.trim().to_uppercase().as_str() {
        "F" => "MOTHER",
        "M" => "FATHER",
        _ => return None,
    };

    // Match the child type to return the full parent term
    match child_type.trim().to_uppercase().as_str() {
        "CHILD, BIOLOGICAL" | "BIOLOGICAL" => Some(if parent_prefix == "MOTHER" {
            "MOTHER, BIOLOGICAL"
        } else {
            "FATHER, BIOLOGICAL"
        }),
        "CHILD, ADOPTIVE" | "ADOPTIVE" => Some(if parent_prefix == "MOTHER" {
            "MOTHER, ADOPTIVE"
        } else {
            "FATHER, ADOPTIVE"
        }),
        "CHILD, FOSTER" | "FOSTER" => Some(if parent_prefix == "MOTHER" {
            "MOTHER, FOSTER"
        } else {
            "FATHER, FOSTER"
        }),
        "CHILD, STEP" | "STEP" => Some(if parent_prefix == "MOTHER" {
            "MOTHER, STEP"
        } else {
            "FATHER, STEP"
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reciprocal_parent_child() {
        assert_eq!(
            get_reciprocal_srel("MOTHER, BIOLOGICAL"),
            Some("CHILD, BIOLOGICAL")
        );
        assert_eq!(
            get_reciprocal_srel("FATHER, BIOLOGICAL"),
            Some("CHILD, BIOLOGICAL")
        );
    }

    #[test]
    fn test_reciprocal_symmetric() {
        assert_eq!(
            get_reciprocal_srel("TWIN, DIZYGOTIC"),
            Some("TWIN, DIZYGOTIC")
        );
        assert_eq!(
            get_reciprocal_srel("TWIN, MONOZYGOTIC"),
            Some("TWIN, MONOZYGOTIC")
        );
        assert_eq!(get_reciprocal_srel("SIBLING"), Some("SIBLING"));
    }

    #[test]
    fn test_is_symmetric() {
        assert!(is_symmetric_srel("TWIN, DIZYGOTIC"));
        assert!(is_symmetric_srel("SIBLING"));
        assert!(is_symmetric_srel("SPOUSE"));
        assert!(!is_symmetric_srel("MOTHER, BIOLOGICAL"));
        assert!(!is_symmetric_srel("HUSBAND"));
    }

    #[test]
    fn test_spouse_reciprocals() {
        assert_eq!(get_reciprocal_srel("HUSBAND"), Some("WIFE"));
        assert_eq!(get_reciprocal_srel("WIFE"), Some("HUSBAND"));
        assert_eq!(get_reciprocal_srel("SPOUSE"), Some("SPOUSE"));
    }

    #[test]
    fn test_parent_for_child() {
        assert_eq!(
            get_parent_srel_for_child("CHILD, BIOLOGICAL", "F"),
            Some("MOTHER, BIOLOGICAL")
        );
        assert_eq!(
            get_parent_srel_for_child("CHILD, BIOLOGICAL", "M"),
            Some("FATHER, BIOLOGICAL")
        );
        assert_eq!(
            get_parent_srel_for_child("BIOLOGICAL", "F"),
            Some("MOTHER, BIOLOGICAL")
        );
    }

    #[test]
    fn test_unknown_relationship() {
        assert_eq!(get_reciprocal_srel("UNKNOWN_REL"), None);
    }
}
