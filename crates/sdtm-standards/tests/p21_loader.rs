//! Tests for Pinnacle 21 rule loader.

use sdtm_standards::{P21Category, load_default_p21_rules};

#[test]
fn test_load_default_p21_rules() {
    let registry = load_default_p21_rules().expect("load P21 rules");

    // Should have loaded rules
    assert!(!registry.is_empty(), "Registry should not be empty");

    // Check some known rules exist
    assert!(registry.get("CT2001").is_some(), "CT2001 should exist");
    assert!(registry.get("CT2002").is_some(), "CT2002 should exist");
    assert!(registry.get("SD0002").is_some(), "SD0002 should exist");
    assert!(registry.get("SD0003").is_some(), "SD0003 should exist");
    assert!(registry.get("SD0005").is_some(), "SD0005 should exist");
}

#[test]
fn test_p21_rule_categories() {
    let registry = load_default_p21_rules().expect("load P21 rules");

    // CT rules should be Terminology
    if let Some(ct2001) = registry.get("CT2001") {
        assert_eq!(ct2001.category, P21Category::Terminology);
    }

    // SD0002 should be Presence
    if let Some(sd0002) = registry.get("SD0002") {
        assert_eq!(sd0002.category, P21Category::Presence);
    }

    // SD0003 should be Format
    if let Some(sd0003) = registry.get("SD0003") {
        assert_eq!(sd0003.category, P21Category::Format);
    }
}

#[test]
fn test_p21_rule_messages() {
    let registry = load_default_p21_rules().expect("load P21 rules");

    // Check CT2001 message
    if let Some(ct2001) = registry.get("CT2001") {
        assert!(
            ct2001.message.contains("non-extensible"),
            "CT2001 message should mention non-extensible: {}",
            ct2001.message
        );
    }

    // Check SD0002 message
    if let Some(sd0002) = registry.get("SD0002") {
        assert!(
            sd0002.message.contains("Required") || sd0002.message.contains("Null"),
            "SD0002 message should mention Required or Null: {}",
            sd0002.message
        );
    }
}
