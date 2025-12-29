#![allow(missing_docs)]

use sdtm_map::{ConfidenceLevel, ConfidenceThresholds, MappingEngine, MappingResult};
use sdtm_model::{ColumnHint, Domain, MappingSuggestion};
use sdtm_standards::load_default_sdtm_ig_domains;
use std::collections::BTreeMap;

fn sample_domain() -> Domain {
    let domains = load_default_sdtm_ig_domains().expect("standards");
    domains
        .into_iter()
        .find(|domain| domain.code == "DM")
        .expect("DM domain")
}

/// Create a MappingResult with specific confidence values for testing.
fn result_with_confidences(confidences: &[f32]) -> MappingResult {
    let mappings = confidences
        .iter()
        .enumerate()
        .map(|(i, &conf)| MappingSuggestion {
            source_column: format!("COL{i}"),
            target_variable: format!("VAR{i}"),
            confidence: conf,
            transformation: None,
        })
        .collect();
    MappingResult {
        mappings,
        unmapped_columns: vec![],
    }
}

#[test]
fn suggests_mappings_and_unmapped() {
    let domain = sample_domain();
    let mut hints = BTreeMap::new();
    hints.insert(
        "AGE".to_string(),
        ColumnHint {
            is_numeric: true,
            unique_ratio: 1.0,
            null_ratio: 0.0,
            label: None,
        },
    );
    let engine = MappingEngine::new(domain, 0.95, hints);
    let columns = vec![
        "STUDYID".to_string(),
        "AGE".to_string(),
        "UNLIKELY_COLUMN".to_string(),
    ];
    let result = engine.suggest(&columns);

    assert!(
        result
            .mappings
            .iter()
            .any(|m| m.target_variable == "STUDYID")
    );
    assert!(result.mappings.iter().any(|m| m.target_variable == "AGE"));
    assert!(
        result
            .unmapped_columns
            .contains(&"UNLIKELY_COLUMN".to_string())
    );
}

// ============================================================================
// Confidence Threshold Tests
// ============================================================================

#[test]
fn confidence_level_ordering() {
    // Verify that confidence levels are ordered correctly
    assert!(ConfidenceLevel::Low < ConfidenceLevel::Medium);
    assert!(ConfidenceLevel::Medium < ConfidenceLevel::High);
    assert!(ConfidenceLevel::Low < ConfidenceLevel::High);
}

#[test]
fn confidence_thresholds_default() {
    let thresholds = ConfidenceThresholds::default();
    assert_eq!(thresholds.high, 0.95);
    assert_eq!(thresholds.medium, 0.80);
    assert_eq!(thresholds.low, 0.60);
}

#[test]
fn confidence_thresholds_strict() {
    let thresholds = ConfidenceThresholds::strict();
    assert!(thresholds.low > ConfidenceThresholds::default().low);
    assert!(thresholds.medium > ConfidenceThresholds::default().medium);
    assert!(thresholds.high > ConfidenceThresholds::default().high);
}

#[test]
fn confidence_thresholds_relaxed() {
    let thresholds = ConfidenceThresholds::relaxed();
    assert!(thresholds.low < ConfidenceThresholds::default().low);
    assert!(thresholds.medium < ConfidenceThresholds::default().medium);
    assert!(thresholds.high < ConfidenceThresholds::default().high);
}

#[test]
fn categorize_confidence_levels() {
    let thresholds = ConfidenceThresholds::default();

    // High confidence (≥ 0.95)
    assert_eq!(thresholds.categorize(1.0), Some(ConfidenceLevel::High));
    assert_eq!(thresholds.categorize(0.99), Some(ConfidenceLevel::High));
    assert_eq!(thresholds.categorize(0.95), Some(ConfidenceLevel::High));

    // Medium confidence (≥ 0.80, < 0.95)
    assert_eq!(thresholds.categorize(0.94), Some(ConfidenceLevel::Medium));
    assert_eq!(thresholds.categorize(0.85), Some(ConfidenceLevel::Medium));
    assert_eq!(thresholds.categorize(0.80), Some(ConfidenceLevel::Medium));

    // Low confidence (≥ 0.60, < 0.80)
    assert_eq!(thresholds.categorize(0.79), Some(ConfidenceLevel::Low));
    assert_eq!(thresholds.categorize(0.70), Some(ConfidenceLevel::Low));
    assert_eq!(thresholds.categorize(0.60), Some(ConfidenceLevel::Low));

    // Below threshold (< 0.60)
    assert_eq!(thresholds.categorize(0.59), None);
    assert_eq!(thresholds.categorize(0.0), None);
}

#[test]
fn categorize_boundary_values() {
    let thresholds = ConfidenceThresholds::default();

    // Test exact boundary values
    assert_eq!(thresholds.categorize(0.95), Some(ConfidenceLevel::High));
    assert_eq!(
        thresholds.categorize(0.9499999),
        Some(ConfidenceLevel::Medium)
    );
    assert_eq!(thresholds.categorize(0.80), Some(ConfidenceLevel::Medium));
    assert_eq!(thresholds.categorize(0.7999999), Some(ConfidenceLevel::Low));
    assert_eq!(thresholds.categorize(0.60), Some(ConfidenceLevel::Low));
    assert_eq!(thresholds.categorize(0.5999999), None);
}

#[test]
fn count_by_level_empty() {
    let result = result_with_confidences(&[]);
    let counts = result.count_by_level();
    assert!(counts.is_empty());
}

#[test]
fn count_by_level_mixed() {
    let result = result_with_confidences(&[0.99, 0.97, 0.85, 0.75, 0.65]);
    let counts = result.count_by_level();

    assert_eq!(counts.get(&ConfidenceLevel::High), Some(&2)); // 0.99, 0.97
    assert_eq!(counts.get(&ConfidenceLevel::Medium), Some(&1)); // 0.85
    assert_eq!(counts.get(&ConfidenceLevel::Low), Some(&2)); // 0.75, 0.65
}

#[test]
fn filter_by_level_high() {
    let result = result_with_confidences(&[0.99, 0.85, 0.65]);
    let high = result.filter_by_level(ConfidenceLevel::High);

    assert_eq!(high.len(), 1);
    assert_eq!(high[0].confidence, 0.99);
}

#[test]
fn filter_by_level_medium_includes_high() {
    let result = result_with_confidences(&[0.99, 0.85, 0.65]);
    let medium_and_up = result.filter_by_level(ConfidenceLevel::Medium);

    assert_eq!(medium_and_up.len(), 2);
    assert!(medium_and_up.iter().any(|m| m.confidence == 0.99));
    assert!(medium_and_up.iter().any(|m| m.confidence == 0.85));
}

#[test]
fn filter_by_level_low_includes_all() {
    let result = result_with_confidences(&[0.99, 0.85, 0.65]);
    let all = result.filter_by_level(ConfidenceLevel::Low);

    assert_eq!(all.len(), 3);
}

#[test]
fn group_by_level() {
    let result = result_with_confidences(&[0.99, 0.97, 0.85, 0.65]);
    let groups = result.group_by_level();

    assert_eq!(groups.get(&ConfidenceLevel::High).map(Vec::len), Some(2));
    assert_eq!(groups.get(&ConfidenceLevel::Medium).map(Vec::len), Some(1));
    assert_eq!(groups.get(&ConfidenceLevel::Low).map(Vec::len), Some(1));
}

#[test]
fn min_max_mean_confidence() {
    let result = result_with_confidences(&[0.90, 0.80, 0.70]);

    assert_eq!(result.min_confidence(), Some(0.70));
    assert_eq!(result.max_confidence(), Some(0.90));
    assert_eq!(result.mean_confidence(), Some(0.80));
}

#[test]
fn min_max_mean_confidence_empty() {
    let result = result_with_confidences(&[]);

    assert_eq!(result.min_confidence(), None);
    assert_eq!(result.max_confidence(), None);
    assert_eq!(result.mean_confidence(), None);
}

#[test]
fn all_high_confidence_true() {
    let result = result_with_confidences(&[0.99, 0.97, 0.95]);
    assert!(result.all_high_confidence());
}

#[test]
fn all_high_confidence_false_with_medium() {
    let result = result_with_confidences(&[0.99, 0.85]);
    assert!(!result.all_high_confidence());
}

#[test]
fn all_high_confidence_empty() {
    let result = result_with_confidences(&[]);
    assert!(!result.all_high_confidence()); // Empty is not "all high"
}

#[test]
fn custom_thresholds_affect_categorization() {
    let strict = ConfidenceThresholds::strict();
    let relaxed = ConfidenceThresholds::relaxed();
    let default = ConfidenceThresholds::default();

    // A score of 0.85 is Medium under default, Low under strict, High under relaxed
    // strict: high=0.98, medium=0.90, low=0.75
    // default: high=0.95, medium=0.80, low=0.60
    // relaxed: high=0.90, medium=0.70, low=0.50
    assert_eq!(default.categorize(0.85), Some(ConfidenceLevel::Medium)); // 0.80 <= 0.85 < 0.95
    assert_eq!(strict.categorize(0.85), Some(ConfidenceLevel::Low)); // 0.75 <= 0.85 < 0.90
    assert_eq!(relaxed.categorize(0.85), Some(ConfidenceLevel::Medium)); // 0.70 <= 0.85 < 0.90

    // A score of 0.92 is Medium under default, Medium under strict, High under relaxed
    assert_eq!(default.categorize(0.92), Some(ConfidenceLevel::Medium));
    assert_eq!(strict.categorize(0.92), Some(ConfidenceLevel::Medium));
    assert_eq!(relaxed.categorize(0.92), Some(ConfidenceLevel::High));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn no_matches_returns_all_unmapped() {
    let domain = sample_domain();
    let engine = MappingEngine::new(domain, 0.95, BTreeMap::new());
    let columns = vec!["ZZZZZ_UNLIKELY".to_string(), "QQQQQ_RANDOM".to_string()];
    let result = engine.suggest(&columns);

    assert!(result.mappings.is_empty());
    assert_eq!(result.unmapped_columns.len(), 2);
}

#[test]
fn empty_columns_returns_empty_result() {
    let domain = sample_domain();
    let engine = MappingEngine::new(domain, 0.95, BTreeMap::new());
    let result = engine.suggest(&[]);

    assert!(result.mappings.is_empty());
    assert!(result.unmapped_columns.is_empty());
}

#[test]
fn exact_match_has_high_confidence() {
    let domain = sample_domain();
    let engine = MappingEngine::new(domain, 0.0, BTreeMap::new()); // Accept all
    let columns = vec!["STUDYID".to_string()];
    let result = engine.suggest(&columns);

    assert_eq!(result.mappings.len(), 1);
    assert_eq!(result.mappings[0].target_variable, "STUDYID");
    assert!(result.mappings[0].confidence >= 0.95);
}

#[test]
fn confidence_level_description() {
    assert!(!ConfidenceLevel::High.description().is_empty());
    assert!(!ConfidenceLevel::Medium.description().is_empty());
    assert!(!ConfidenceLevel::Low.description().is_empty());
}
