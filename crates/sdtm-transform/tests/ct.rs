//! Tests for controlled terminology normalization.

use sdtm_model::ct::{Codelist, Term};
use sdtm_transform::normalization::{normalize_ct_value, NormalizationOptions};

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
