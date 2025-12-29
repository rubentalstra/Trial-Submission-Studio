#![allow(missing_docs)]

use sdtm_model::ct::{Codelist, Term};

#[test]
fn test_codelist_validation() {
    let mut sex = Codelist::new("C66731".to_string(), "Sex".to_string(), false);
    sex.add_term(Term {
        code: "C16576".to_string(),
        submission_value: "F".to_string(),
        synonyms: vec!["Female".to_string()],
        definition: None,
        preferred_term: Some("Female".to_string()),
    });
    sex.add_term(Term {
        code: "C20197".to_string(),
        submission_value: "M".to_string(),
        synonyms: vec!["Male".to_string()],
        definition: None,
        preferred_term: Some("Male".to_string()),
    });
    sex.add_term(Term {
        code: "C17998".to_string(),
        submission_value: "U".to_string(),
        synonyms: vec!["U".to_string(), "UNK".to_string(), "Unknown".to_string()],
        definition: None,
        preferred_term: Some("Unknown".to_string()),
    });

    // Valid submission values
    assert!(sex.is_valid("F"));
    assert!(sex.is_valid("f")); // Case-insensitive
    assert!(sex.is_valid("M"));
    assert!(sex.is_valid("U"));

    // Valid synonyms
    assert!(sex.is_valid("Female"));
    assert!(sex.is_valid("FEMALE"));
    assert!(sex.is_valid("Male"));
    assert!(sex.is_valid("UNK"));
    assert!(sex.is_valid("Unknown"));

    // Invalid
    assert!(!sex.is_valid("X"));
    assert!(!sex.is_valid("OTHER"));
}

#[test]
fn test_codelist_normalization() {
    let mut sex = Codelist::new("C66731".to_string(), "Sex".to_string(), false);
    sex.add_term(Term {
        code: "C17998".to_string(),
        submission_value: "U".to_string(),
        synonyms: vec!["UNK".to_string(), "Unknown".to_string()],
        definition: None,
        preferred_term: None,
    });

    // Normalize synonyms to submission value
    assert_eq!(sex.normalize("UNK"), "U");
    assert_eq!(sex.normalize("Unknown"), "U");
    assert_eq!(sex.normalize("u"), "U");

    // Unknown value stays as-is
    assert_eq!(sex.normalize("OTHER"), "OTHER");
}

#[test]
fn test_extensible_codelist() {
    let extensible = Codelist::new("C99999".to_string(), "Test".to_string(), true);
    assert!(extensible.extensible);
    assert!(!extensible.is_valid("ANYTHING")); // Not valid, but extensible allows warning-only
}
